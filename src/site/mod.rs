//! The static site generator, which builds `_site` from the pages compiled
//! into this crate.
//!
//! [`Site::load`] reads every page's front matter, and [`Site::build`]
//! renders them and writes the site:
//!
//! 1. Pages with template tags, like [`ramblings`](crate::ramblings), are
//!    rendered with askama first, from templates `build.rs` declares.
//! 2. Every page's Markdown is rendered to HTML by [`Markdown`].
//! 3. The page is wrapped in its [`Layout`], and then the site's default
//!    one, with the header and nav.
//! 4. The Markdown, wrapped in its layout but not the default one, is
//!    published next to the HTML, like `/ramblings/foo.md`, for the
//!    terminal view. So is `site.json`, the [`Index`].
//!
//! Everything else in the repository that's part of the site, like images
//! and CSS, is copied over as it is. The fortunes get their `strfile`
//! databases, and TikZ figures are drawn to SVG.

use std::fs;
use std::path::{Path, PathBuf};

use askama::Template as _;
use chrono::{DateTime, Local};

mod data;
mod fortunes;
mod highlight;
mod layout;
mod markdown;
mod page;
mod publish;
mod serve;
mod text;
mod tikz;

pub use data::{Data, Reference, Tags};
pub use fortunes::Fortunes;
pub use highlight::Highlighter;
pub use layout::{DefaultLayout, Layout};
pub use markdown::Markdown;
pub use page::{FrontMatter, Page};
pub use serve::Server;
pub use text::Text;
pub use tikz::Tikz;

use crate::collection::{Collection, Files};
use crate::index::{Entry, Index};

/// The site, its pages and its data.
pub struct Site {
    pub options: Options,
    /// When the build started, which dates undated pages and shows in the
    /// footer.
    pub time: DateTime<Local>,
    /// Every page, those in collections sorted by date, then by path.
    pub pages: Vec<Page>,
    pub data: Data,
    pub fortunes: Vec<Fortunes>,
}

/// How to build the site.
#[derive(Debug, Clone)]
pub struct Options {
    /// The top of the repository, with the files copied into the site.
    pub source: PathBuf,
    /// Where the site is written, cleared first.
    pub destination: PathBuf,
    /// What every path on the site starts with, for previews under a
    /// path, like `/pr-preview/pr-83`.
    pub baseurl: String,
    /// Build pages marked `published: false` too.
    pub unpublished: bool,
}

impl Site {
    pub const TITLE: &str = "nixpulvis";
    pub const URL: &str = "https://nixpulvis.com";
    pub const REPOSITORY: &str = "https://github.com/nixpulvis/nixpulvis.github.io";
    pub const DESCRIPTION: &str = "Projects, ramblings and notes by Nathan Lilienthal.";
    /// What a build leaves in the destination: the terminal view and the
    /// docs, which are built separately.
    const KEEP: &[&str] = &["tui", "doc"];

    /// Reads every page and data file, ready to build.
    pub fn load(options: Options) -> Result<Self, String> {
        let time = Local::now();
        let mut pages = Vec::new();
        for source in crate::SOURCES {
            let page = Page::read(source)?;
            if page.front.published || options.unpublished {
                pages.push(page);
            }
        }
        // Undated pages sort as if written now.
        let today = time.date_naive();
        pages.sort_by(|a, b| {
            let key = |p: &Page| (p.date.unwrap_or(today), p.source.path);
            key(a).cmp(&key(b))
        });
        Ok(Self {
            data: Data::load()?,
            fortunes: Fortunes::read(&options.source.join("fortunes"))?,
            options,
            time,
            pages,
        })
    }

    /// Renders every page and writes the site.
    pub fn build(&mut self) -> Result<(), String> {
        self.render()?;
        self.write()
    }

    /// Renders the templates, then the Markdown, of every page.
    fn render(&mut self) -> Result<(), String> {
        let highlighter = Highlighter::default();
        let tikz = Tikz::new(self.options.source.join(".site-cache/tikz"));
        let markdown = Markdown {
            highlighter: &highlighter,
            tikz: &tikz,
        };
        let render = |page: &mut Page| -> Result<(), String> {
            if !page.is_markdown() {
                return Ok(());
            }
            page.content = markdown
                .render(&page.markdown)
                .map_err(|e| format!("{}: {e}", page.source.path))?;
            if let Some(excerpt) = page.excerpt_markdown() {
                page.excerpt = markdown.render(&excerpt)?;
            }
            Ok(())
        };
        // Pages with templates list the others, with their word counts, so
        // they come last.
        for page in self.pages.iter_mut().filter(|p| !p.is_template()) {
            render(page)?;
        }
        let mut rendered = Vec::new();
        for (i, page) in self.pages.iter().enumerate() {
            if let Some(text) = templates::render(page.source.path, self, page) {
                let text = text.map_err(|e| format!("{}: {e}", page.source.path))?;
                rendered.push((i, FrontMatter::split(&text).1.to_owned()));
            }
        }
        for (i, text) in rendered {
            let page = &mut self.pages[i];
            page.markdown = text;
            render(page)?;
        }
        Ok(())
    }

    /// Writes the site into the destination, which is cleared first, all
    /// but [`Site::KEEP`].
    fn write(&self) -> Result<(), String> {
        let dest = &self.options.destination;
        if let Ok(entries) = fs::read_dir(dest) {
            for entry in entries.flatten() {
                let path = entry.path();
                if Self::KEEP.iter().any(|keep| entry.file_name() == *keep) {
                    continue;
                }
                let removed = if path.is_dir() {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_file(&path)
                };
                removed.map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
        self.copy_files(&self.options.source, dest)?;
        for page in &self.pages {
            let out = if page.is_markdown() {
                let content =
                    Layout::named(page.front.layout.as_deref()).wrap(self, page, &page.content)?;
                match page.front.layout.as_deref() {
                    None | Some("null") => content,
                    Some(_) => layout::DefaultLayout {
                        site: self,
                        page,
                        content: &content,
                    }
                    .render()
                    .map_err(|e| e.to_string())?,
                }
            } else {
                page.markdown.clone()
            };
            self.write_file(&page.output_path(), self.with_baseurl(&out))?;
            if let Some(url) = page.markdown_url() {
                let published = publish::Published::new(self, page)?;
                self.write_file(&url, self.with_baseurl(&published.to_string()))?;
            }
        }
        for year in &self.fortunes {
            self.write_file(&year.dat(), year.strfile())?;
        }
        let index = serde_json::to_string_pretty(&self.index()).map_err(|e| e.to_string())?;
        self.write_file("site.json", index)?;
        // The site is already built, so stop Pages from running Jekyll on it.
        self.write_file(".nojekyll", "")?;
        Ok(())
    }

    /// The index published as `site.json`, of the nav and every page in a
    /// collection.
    pub fn index(&self) -> Index {
        let mut pages = Vec::new();
        for collection in Collection::ALL {
            for page in self.collection(collection.label) {
                let front = publish::Published::front_matter(self, page);
                pages.push(Entry {
                    title: page.title(),
                    date: front.date,
                    collection: Some(collection.label.to_owned()),
                    draft: page.front.draft,
                    url: page.url.clone(),
                });
            }
        }
        Index {
            title: Self::TITLE.to_owned(),
            url: Self::URL.to_owned(),
            baseurl: self.options.baseurl.clone(),
            nav: self.data.nav.clone(),
            pages,
        }
    }

    /// Copies every file that's part of the site, but isn't a page, from
    /// `dir` into the same place under `dest`.
    fn copy_files(&self, dir: &Path, dest: &Path) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let relative = path
                .strip_prefix(&self.options.source)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if Files::ignored(&relative) || self.pages.iter().any(|p| p.source.path == relative) {
                continue;
            }
            if path.is_dir() {
                self.copy_files(&path, dest)?;
            } else if !relative.ends_with(".dat") {
                let to = dest.join(&relative);
                fs::create_dir_all(to.parent().unwrap()).map_err(|e| e.to_string())?;
                fs::copy(&path, &to).map_err(|e| format!("{relative}: {e}"))?;
            }
        }
        Ok(())
    }

    fn write_file(&self, path: &str, contents: impl AsRef<[u8]>) -> Result<(), String> {
        let file = self.options.destination.join(path.trim_start_matches('/'));
        fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(&file, contents).map_err(|e| format!("{}: {e}", file.display()))
    }

    /// The pages in the collection `label`, by date.
    pub fn collection(&self, label: &str) -> Vec<&Page> {
        self.pages
            .iter()
            .filter(|p| p.collection.is_some_and(|c| c.label == label))
            .collect()
    }

    /// The ramblings and mathematics, newest first, as they're listed on
    /// the Ramblings page and in the feed.
    pub fn ramblings(&self) -> Vec<&Page> {
        let mut pages = self.collection("ramblings");
        pages.extend(self.collection("mathematics"));
        let today = self.time.date_naive();
        pages.sort_by_key(|p| std::cmp::Reverse(p.date.unwrap_or(today)));
        pages
    }

    /// The pages in the collection `label`, newest first.
    pub fn newest_first(&self, label: &str) -> Vec<&Page> {
        let mut pages = self.collection(label);
        pages.reverse();
        pages
    }

    /// One fortune from every year's, picked at random each build.
    pub fn random_fortune(&self) -> &str {
        let all: Vec<&String> = self.fortunes.iter().flat_map(|y| &y.entries).collect();
        if all.is_empty() {
            return "";
        }
        let i = self.time.timestamp_subsec_nanos() as usize % all.len();
        all[i]
    }

    /// Every fortune, as a JSON array.
    pub fn fortunes_json(&self) -> String {
        let all: Vec<&String> = self.fortunes.iter().flat_map(|y| &y.entries).collect();
        serde_json::to_string(&all).unwrap_or_default()
    }

    /// `path`, from the site's root, with the base URL in front.
    pub fn relative_url(&self, path: &str) -> String {
        format!("{}{path}", self.options.baseurl)
    }

    /// `path`, from the site's root, as a full URL.
    pub fn absolute_url(&self, path: &str) -> String {
        format!("{}{}{path}", Self::URL, self.options.baseurl)
    }

    /// The build time, like `2026-10-30T17:03:50+00:00`.
    pub fn xml_time(&self) -> String {
        self.time
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
    }

    /// The build time, for adding to URLs so browsers fetch new copies.
    pub fn build_id(&self) -> i64 {
        self.time.timestamp()
    }

    /// The page's date, at midnight where the site's built, for the feed.
    pub fn xml_date(&self, page: &Page) -> String {
        let date = page.date.unwrap_or(self.time.date_naive());
        match date
            .and_hms_opt(0, 0, 0)
            .and_then(|d| d.and_local_timezone(Local).single())
        {
            Some(time) => time.to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
            None => date.to_string(),
        }
    }

    /// `text` escaped for HTML or XML.
    pub fn escape(&self, text: &str) -> String {
        Text::escape(text)
    }

    /// The text with the base URL put in front of every link and source
    /// from the site's root, like `href="/about"`, Markdown links like
    /// `](/about)` included. Code blocks are left alone.
    fn with_baseurl(&self, text: &str) -> String {
        let base = &self.options.baseurl;
        if base.is_empty() {
            return text.to_owned();
        }
        let mut out = String::with_capacity(text.len());
        let mut fence: Option<&str> = None;
        for line in text.split_inclusive('\n') {
            let trimmed = line.trim_start();
            if let Some(marker) = fence {
                if trimmed.starts_with(marker) {
                    fence = None;
                }
                out.push_str(line);
                continue;
            }
            if let Some(marker) = ["```", "~~~"].into_iter().find(|m| trimmed.starts_with(m)) {
                fence = Some(marker);
                out.push_str(line);
                continue;
            }
            let mut line = line.to_owned();
            for prefix in ["href=\"", "src=\"", "data-music-xml=\"", "](", "]: "] {
                line = Self::prefix_paths(&line, prefix, base);
            }
            out.push_str(&line);
        }
        out
    }

    /// Puts `base` in front of every path from the root after `prefix`.
    fn prefix_paths(text: &str, prefix: &str, base: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(i) = rest.find(prefix) {
            let after = i + prefix.len();
            out.push_str(&rest[..after]);
            rest = &rest[after..];
            if rest.starts_with('/') && !rest.starts_with("//") && !rest.starts_with(base) {
                out.push_str(base);
            }
        }
        out.push_str(rest);
        out
    }
}

/// The templates for pages with template tags, made by `build.rs`.
mod templates {
    use askama::Template;

    use super::{Page, Site};

    include!(concat!(env!("OUT_DIR"), "/templates.rs"));
}
