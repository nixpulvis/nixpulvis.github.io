//! The app: what's showing, where it's been, and what input does.

use ratatui::layout::Rect;
use ratmark::Theme;
// Like std's, but also works in the browser.
use web_time::Instant;

use crate::tui::link::{self, Target};
use crate::tui::page::Page;
use crate::tui::site::Site;
use crate::tui::{Effect, Host, Key, colors};

mod draw;
#[cfg(test)]
mod tests;

/// How many columns `<` and `>` shift wide code by.
const SHIFT: isize = 8;

pub struct App {
    host: Host,
    /// The site's address, like `https://nixpulvis.com`.
    origin: String,
    /// What every path on the site starts with, without the last `/`, so
    /// empty except in previews.
    base: String,
    site: Option<Site>,
    page: Option<Page>,
    /// The pages before this one, for going back, when the app keeps the
    /// history.
    history: Vec<Visit>,
    loading: Option<Loading>,
    /// Shown at the bottom until another page opens, like a page or the
    /// site's index failing to load.
    message: Option<String>,
    /// Shown at the bottom for a moment, like a link opening elsewhere.
    notice: Option<(String, Instant)>,
    theme: Theme,
    /// Where things were drawn in the last frame, for clicks.
    drawn: Drawn,
}

/// A page visited before, and how far down it was read.
struct Visit {
    path: String,
    scroll: usize,
}

/// The page being fetched, and where to show it from once it's here.
struct Loading {
    path: String,
    anchor: Option<String>,
    scroll: Option<usize>,
    /// Whether this is the site's page for addresses that lead nowhere,
    /// fetched in place of `path`.
    not_found: bool,
}

impl Loading {
    fn new(path: String, anchor: Option<String>) -> Self {
        Self {
            path,
            anchor,
            scroll: None,
            not_found: false,
        }
    }
}

/// Where the nav, the close button and the page's text landed in the last
/// frame.
#[derive(Default)]
struct Drawn {
    nav: Vec<(Rect, String)>,
    close: Rect,
    text: Rect,
}

/// A page in the header, the site's name first.
struct NavItem {
    label: String,
    path: String,
    /// Whether it names its page in words, rather than an icon or the
    /// site's name.
    names_page: bool,
}

impl App {
    /// A new app in `host`, for the site at `origin`, like
    /// `https://nixpulvis.com`, under the path `base`, `/` except in
    /// previews. Call [`App::start`] next.
    pub fn new(host: Host, origin: impl Into<String>, base: &str) -> Self {
        Self {
            host,
            origin: origin.into().trim_end_matches('/').to_owned(),
            base: base.trim_end_matches('/').to_owned(),
            site: None,
            page: None,
            history: Vec::new(),
            loading: None,
            message: None,
            notice: None,
            theme: colors::theme(),
            drawn: Drawn::default(),
        }
    }

    /// Starts on the page at `path`, which may have a `#` section.
    pub fn start(&mut self, path: &str) -> Vec<Effect> {
        let (path, anchor) = link::page_at(path);
        let mut effects = vec![Effect::FetchSite(format!("{}/site.json", self.base))];
        effects.extend(self.load(Loading::new(path, anchor)));
        effects
    }

    /// Takes the site's index, `site.json`, as [`Effect::FetchSite`] asked.
    pub fn site_loaded(&mut self, json: Result<String, String>) {
        let site = json
            .and_then(|json| Site::parse(&json))
            .unwrap_or_else(|e| {
                self.message = Some(format!("Couldn't load the site index: {e}"));
                Site::default()
            });
        self.site = Some(site);
    }

    /// Takes a page's Markdown, as [`Effect::FetchPage`] asked.
    pub fn page_loaded(&mut self, path: &str, markdown: Result<String, String>) -> Vec<Effect> {
        let Some(loading) = self.loading.take_if(|l| l.path == path) else {
            return Vec::new();
        };
        let markdown = match markdown {
            Ok(markdown) => markdown,
            Err(_) if !loading.not_found => {
                // NOTE: When a page shares its name with a folder, like
                // ramblings.html and ramblings/, GitHub Pages serves
                // /ramblings straight from ramblings.html. WEBrick, which
                // jekyll serve runs on, sees the folder first and redirects
                // to /ramblings/, so that's the address the view starts on.
                if let Some(page) = path.strip_suffix('/').filter(|p| !p.is_empty()) {
                    let path = page.to_owned();
                    return self.load(Loading { path, ..loading });
                }
                return self.load(Loading {
                    not_found: true,
                    ..loading
                });
            }
            Err(e) => {
                self.message = Some(format!("Couldn't load {path}: {e}"));
                return Vec::new();
            }
        };
        // The site's 404 names the missing page with a script.
        let markdown = if loading.not_found {
            markdown.replace(">that-page<", &format!(">{path}<"))
        } else {
            markdown
        };
        let mut page = Page::new(path, self.url(path), &markdown);
        page.view.scroll_to(loading.scroll.unwrap_or(0));
        if let Some(anchor) = &loading.anchor {
            page.view.scroll_to_anchor(anchor);
        }
        self.page = Some(page);
        Vec::new()
    }

    pub fn key(&mut self, key: Key) -> Vec<Effect> {
        match key {
            Key::Char('q') | Key::Esc => return vec![Effect::Quit],
            Key::Back | Key::Char('h') => return self.back(),
            Key::Char(c @ '1'..='9') => {
                let i = usize::from(c as u8 - b'1');
                return match self.nav().into_iter().nth(i) {
                    Some(item) => self.open(item.path, None),
                    None => Vec::new(),
                };
            }
            _ => {}
        }
        let Some(view) = self.page.as_mut().map(|p| &mut p.view) else {
            return Vec::new();
        };
        match key {
            Key::Down | Key::Char('j') => _ = view.scroll_by(1),
            Key::Up | Key::Char('k') => _ = view.scroll_by(-1),
            Key::PageDown | Key::Char(' ') | Key::Char('f') => view.page(true),
            Key::PageUp | Key::Char('b') => view.page(false),
            Key::Home | Key::Char('g') => view.scroll_to(0),
            Key::End | Key::Char('G') => view.scroll_to(usize::MAX),
            Key::Char('>') => _ = view.shift_by(SHIFT),
            Key::Char('<') => _ = view.shift_by(-SHIFT),
            Key::Tab => view.focus_next(),
            Key::BackTab => view.focus_prev(),
            Key::Enter => {
                if let Some(url) = view.focused().map(|l| l.url.clone()) {
                    return self.follow(&url);
                }
            }
            _ => {}
        }
        Vec::new()
    }

    /// A click or tap at column `x`, row `y` of the screen.
    pub fn click(&mut self, x: u16, y: u16) -> Vec<Effect> {
        let at = (x, y).into();
        if self.drawn.close.contains(at) {
            return vec![Effect::Quit];
        }
        if let Some((_, path)) = self.drawn.nav.iter().find(|(r, _)| r.contains(at)) {
            return self.open(path.clone(), None);
        }
        let link = self.page.as_ref().and_then(|p| p.view.link_at(x, y));
        match link.map(|l| l.url.clone()) {
            Some(url) => self.follow(&url),
            None => Vec::new(),
        }
    }

    /// Scrolls by `lines`, down when positive. Returns whether it moved,
    /// which it doesn't at either end.
    pub fn scroll_by(&mut self, lines: isize) -> bool {
        self.page
            .as_mut()
            .is_some_and(|page| page.view.scroll_by(lines))
    }

    /// Shifts the code block too wide for the page at column `x`, row `y`
    /// of the screen by `columns`, showing more of the right when positive.
    /// Returns whether it moved, which it doesn't at its end.
    pub fn shift_at(&mut self, x: u16, y: u16, columns: isize) -> bool {
        self.page
            .as_mut()
            .is_some_and(|page| page.view.shift_at(x, y, columns))
    }

    /// Whether the app is still fetching what its first frame needs, the
    /// first page and the site's index for the nav. Until they arrive, or
    /// fail to, it draws only its background, rather than flash a
    /// half-drawn screen.
    fn starting(&self) -> bool {
        self.site.is_none() || self.page.is_none() && self.loading.is_some()
    }

    /// Where the page's text was drawn in the last frame, for hosts that
    /// let it be selected.
    pub fn text_area(&self) -> Rect {
        self.drawn.text
    }

    /// Follows `href`, a link on the current page.
    fn follow(&mut self, href: &str) -> Vec<Effect> {
        let current = self.page.as_ref().map_or("/", |p| p.path.as_str());
        match Target::resolve(href, current, &self.origin) {
            Target::Page { path, anchor } => self.open(path, anchor),
            Target::Anchor(anchor) => {
                if let Some(page) = &mut self.page {
                    page.jump(&anchor);
                }
                Vec::new()
            }
            Target::External(url) => {
                self.notice = Some((format!("Opened {url}"), Instant::now()));
                vec![Effect::Open(url)]
            }
        }
    }

    /// Goes to another page of the site, leaving this one to come back to.
    fn open(&mut self, path: String, anchor: Option<String>) -> Vec<Effect> {
        match self.host {
            Host::Browser => {
                self.message = Some(format!("Loading {path}…"));
                vec![Effect::Navigate(match anchor {
                    Some(anchor) => format!("{path}#{anchor}"),
                    None => path,
                })]
            }
            Host::Terminal => {
                self.message = None;
                if let Some(page) = &self.page {
                    self.history.push(Visit {
                        path: page.path.clone(),
                        scroll: page.view.scroll(),
                    });
                }
                self.load(Loading::new(path, anchor))
            }
        }
    }

    fn back(&mut self) -> Vec<Effect> {
        if self.host == Host::Browser {
            return vec![Effect::Back];
        }
        match self.history.pop() {
            Some(Visit { path, scroll }) => {
                self.message = None;
                self.load(Loading {
                    scroll: Some(scroll),
                    ..Loading::new(path, None)
                })
            }
            None => Vec::new(),
        }
    }

    fn load(&mut self, loading: Loading) -> Vec<Effect> {
        let source = if loading.not_found {
            format!("{}/404.md", self.base)
        } else {
            link::source(&loading.path)
        };
        let path = loading.path.clone();
        self.loading = Some(loading);
        vec![Effect::FetchPage { path, source }]
    }

    /// The header's pages, which the number keys go to.
    fn nav(&self) -> Vec<NavItem> {
        let home = NavItem {
            label: "nixpulvis".to_owned(),
            path: format!("{}/", self.base),
            names_page: false,
        };
        let pages = self
            .site
            .iter()
            .flat_map(|site| &site.nav)
            .map(|nav| NavItem {
                label: nav.icon.clone().unwrap_or_else(|| nav.title.clone()),
                path: format!("{}{}", self.base, nav.url),
                names_page: nav.icon.is_none(),
            });
        std::iter::once(home).chain(pages).collect()
    }

    /// The full address of the site's page at `path`.
    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.origin)
    }
}
