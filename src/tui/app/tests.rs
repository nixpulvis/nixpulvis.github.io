//! The app driven the way a host drives it, against a small made-up site,
//! checking what a reader would see.

use std::collections::HashMap;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;

use super::App;
use crate::tui::colors::FAINT;
use crate::tui::{Effect, Host, Key};

/// The made-up site, by the paths its files are published at.
fn site() -> Vec<(String, String)> {
    [
        (
            "/site.json",
            r#"{
                "nav": [
                    {"title": "Ramblings", "url": "/ramblings"},
                    {"title": "References", "url": "/references"},
                    {"title": "Lambda", "icon": "λ", "url": "/lambda"},
                    {"title": "Math", "url": "/math", "tui": false}
                ],
                "pages": [{"url": "/ramblings/first"}, {"url": "/ramblings/draft", "draft": true}]
            }"#,
        ),
        (
            "/index.md",
            "---\ntitle: About\n---\n\nHi, I am Nathan. Read my [ramblings](/ramblings), \
             skip to [the end](/ramblings/first#the-end), see my [resume](/resume.pdf), \
             or find me on [GitHub](https://github.com/nixpulvis).\n",
        ),
        (
            "/ramblings.md",
            "---\ntitle: Ramblings\n---\n\n\
             - [First post](/ramblings/first)\n- [Unfinished](/ramblings/draft)\n",
        ),
        (
            "/ramblings/first.md",
            &format!(
                "---\ntitle: First post\ndate: '2026-10-19'\nwords: 300\n---\n\n\
                 Skip to [the end](#the-end).\n\n{}## The end\n\nThat is all.\n",
                (1..=40)
                    .map(|i| format!("Line {i}.\n\n"))
                    .collect::<String>()
            ),
        ),
        (
            "/ramblings/draft.md",
            "---\ntitle: Unfinished\ndraft: true\n---\n\nSoon.\n",
        ),
        (
            "/references.md",
            "---\ntitle: References\n---\n\nShow [unix](#unix) or [rust](#rust).\n\n<ul>\n\
             <li data-tags=\"unix\">The Unix book</li>\n\
             <li data-tags=\"rust\">The Rust book</li>\n</ul>\n",
        ),
        (
            "/404.md",
            "---\ntitle: Not Found\n---\n\n\
             <pre><code>$ cd <span class=\"missing\">that-page</span></code></pre>\n",
        ),
    ]
    .into_iter()
    .map(|(path, file)| (path.to_owned(), file.to_owned()))
    .collect()
}

/// Plays the host: serves the app's fetches from a set of files, draws it
/// on a fake terminal, and keeps everything else it asks for.
struct Reader {
    app: App,
    files: HashMap<String, String>,
    /// The fetches the app is waiting on.
    fetches: Vec<Effect>,
    /// Everything else it's asked for, oldest first.
    asked: Vec<Effect>,
    terminal: Terminal<TestBackend>,
}

impl Reader {
    fn new(host: Host, base: &str, files: Vec<(String, String)>) -> Self {
        Self {
            app: App::new(host, "https://nixpulvis.com", base),
            files: files.into_iter().collect(),
            fetches: Vec::new(),
            asked: Vec::new(),
            terminal: Terminal::new(TestBackend::new(80, 16)).unwrap(),
        }
    }

    /// The made-up site, open on the page at `path`.
    fn open(host: Host, path: &str) -> Self {
        let mut reader = Self::new(host, "/", site());
        reader.start(path);
        reader.serve();
        reader
    }

    fn start(&mut self, path: &str) {
        let effects = self.app.start(path);
        self.take(effects);
    }

    fn key(&mut self, key: Key) {
        let effects = self.app.key(key);
        self.take(effects);
        self.serve();
    }

    fn press(&mut self, keys: &str) {
        for c in keys.chars() {
            self.key(Key::Char(c));
        }
    }

    /// Clicks the first place `text` shows on screen.
    fn click(&mut self, text: &str) {
        let Some((x, y)) = self.find(text) else {
            panic!("{text:?} isn't on screen:\n{}", self.screen());
        };
        let effects = self.app.click(x, y);
        self.take(effects);
        self.serve();
    }

    /// Answers every fetch, and the ones those lead to.
    fn serve(&mut self) {
        self.serve_only(|_| true);
    }

    /// Answers the fetches `which` picks, and the ones those lead to.
    fn serve_only(&mut self, which: impl Fn(&Effect) -> bool) {
        while let Some(i) = self.fetches.iter().position(&which) {
            match self.fetches.remove(i) {
                Effect::FetchSite(path) => {
                    let json = self.get(&path);
                    self.app.site_loaded(json);
                }
                Effect::FetchPage { path, source } => {
                    let markdown = self.get(&source);
                    let effects = self.app.page_loaded(&path, markdown);
                    self.take(effects);
                }
                _ => unreachable!(),
            }
        }
    }

    /// What the app has asked for since last time, other than fetches.
    fn asked(&mut self) -> Vec<Effect> {
        std::mem::take(&mut self.asked)
    }

    /// The screen as text, a line a row.
    fn screen(&mut self) -> String {
        self.rows().join("\n")
    }

    /// The color of the first place `text` shows on screen.
    fn color_of(&mut self, text: &str) -> Color {
        let (x, y) = self.find(text).unwrap();
        self.terminal.backend().buffer()[(x, y)].fg
    }

    fn rows(&mut self) -> Vec<String> {
        self.terminal.draw(|frame| self.app.draw(frame)).unwrap();
        let buffer = self.terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect()
            })
            .collect()
    }

    fn find(&mut self, text: &str) -> Option<(u16, u16)> {
        self.rows().iter().enumerate().find_map(|(y, row)| {
            let x = row[..row.find(text)?].chars().count();
            Some((x as u16, y as u16))
        })
    }

    fn take(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::FetchSite(_) | Effect::FetchPage { .. } => self.fetches.push(effect),
                _ => self.asked.push(effect),
            }
        }
    }

    fn get(&self, path: &str) -> Result<String, String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| "404 Not Found".to_owned())
    }
}

#[test]
fn reads_through_the_site() {
    let mut reader = Reader::open(Host::Terminal, "/");
    let screen = reader.screen();
    // Math only works in a browser.
    assert!(screen.contains(" nixpulvis  Ramblings  References  λ  "));
    assert!(screen.contains("Hi, I am Nathan."));
    assert!(screen.contains("1-4 pages · q quit"));

    reader.key(Key::Tab);
    reader.key(Key::Enter);
    reader.click("First post");
    let screen = reader.screen();
    assert!(screen.contains(" First post "));
    assert!(screen.contains(" 300 words published on October 19, 2026 "));

    // Away and back to the same place.
    reader.press("jjj");
    let read = reader.screen();
    reader.press("3");
    assert!(reader.screen().contains("The Unix book"));
    reader.key(Key::Back);
    assert_eq!(reader.screen(), read);

    reader.press("hh");
    assert!(reader.screen().contains("Hi, I am Nathan."));
    // Nothing before the first page.
    reader.press("h");
    assert!(reader.screen().contains("Hi, I am Nathan."));
    assert_eq!(reader.asked(), []);
    reader.press("q");
    assert_eq!(reader.asked(), [Effect::Quit]);
}

#[test]
fn opens_other_sites_elsewhere() {
    let mut reader = Reader::open(Host::Terminal, "/");
    reader.click("GitHub");
    assert_eq!(
        reader.asked(),
        [Effect::Open("https://github.com/nixpulvis".into())]
    );
    assert!(
        reader
            .screen()
            .contains("Opened https://github.com/nixpulvis")
    );
    // Files on the site that aren't pages too.
    reader.click("resume");
    assert_eq!(
        reader.asked(),
        [Effect::Open("https://nixpulvis.com/resume.pdf".into())]
    );
}

#[test]
fn jumps_to_sections() {
    let mut reader = Reader::open(Host::Terminal, "/ramblings/first");
    assert!(!reader.screen().contains("That is all."));
    reader.click("the end");
    assert!(reader.screen().contains("That is all."));

    // A tag's link shows only the items with that tag.
    let mut reader = Reader::open(Host::Terminal, "/references");
    reader.click("unix");
    let screen = reader.screen();
    assert!(screen.contains("The Unix book"));
    assert!(!screen.contains("The Rust book"));
}

#[test]
fn starts_where_the_address_says() {
    let mut reader = Reader::open(Host::Terminal, "/ramblings/first#the-end");
    assert!(reader.screen().contains("That is all."));
    // Old addresses.
    let mut reader = Reader::open(Host::Terminal, "/ramblings/first.html");
    assert!(reader.screen().contains(" First post "));
    // jekyll serve's address for ramblings.html, beside ramblings/.
    let mut reader = Reader::open(Host::Terminal, "/ramblings/");
    assert!(reader.screen().contains("First post"));
}

#[test]
fn missing_pages_show_the_404() {
    let mut reader = Reader::open(Host::Terminal, "/nope");
    let screen = reader.screen();
    assert!(screen.contains(" Not Found "));
    assert!(screen.contains("$ cd /nope"));

    let mut reader = Reader::open(Host::Terminal, "/ramblings/nope");
    reader.press("h");
    assert!(reader.screen().contains("$ cd /ramblings/nope"));

    // And when the 404 is missing too.
    let mut files = site();
    files.retain(|(path, _)| path != "/404.md");
    let mut reader = Reader::new(Host::Terminal, "/", files);
    reader.start("/nope");
    reader.serve();
    assert!(
        reader
            .screen()
            .contains("Couldn't load /nope: 404 Not Found")
    );
}

#[test]
fn browsers_load_the_pages() {
    let mut reader = Reader::open(Host::Browser, "/");
    assert!(reader.screen().contains("q leave"));
    reader.click("ramblings");
    assert_eq!(reader.asked(), [Effect::Navigate("/ramblings".into())]);
    assert!(reader.screen().contains("Loading /ramblings…"));
    reader.click("the end");
    assert_eq!(
        reader.asked(),
        [Effect::Navigate("/ramblings/first#the-end".into())]
    );
    reader.click("References");
    assert_eq!(reader.asked(), [Effect::Navigate("/references".into())]);
    reader.press("h");
    assert_eq!(reader.asked(), [Effect::Back]);
    reader.click("×");
    assert_eq!(reader.asked(), [Effect::Quit]);
}

#[test]
fn previews_stay_under_their_base() {
    const BASE: &str = "/pr-preview/pr-1";
    // Jekyll writes the base into the links in a preview.
    let files = site()
        .into_iter()
        .map(|(path, file)| {
            (
                format!("{BASE}{path}"),
                file.replace("](/", &format!("]({BASE}/")),
            )
        })
        .collect();
    let mut reader = Reader::new(Host::Terminal, "/pr-preview/pr-1/", files);
    reader.start("/pr-preview/pr-1/");
    reader.serve();
    assert!(reader.screen().contains("Hi, I am Nathan."));
    reader.press("2");
    reader.click("First post");
    assert!(reader.screen().contains(" First post "));
    reader.press("1");
    assert!(reader.screen().contains("Hi, I am Nathan."));
    reader.start("/pr-preview/pr-1/nope");
    reader.serve();
    assert!(reader.screen().contains("$ cd /pr-preview/pr-1/nope"));
}

#[test]
fn draws_nothing_until_the_page_and_nav_are_in() {
    let blank = |screen: String| screen.chars().all(char::is_whitespace);
    let mut reader = Reader::new(Host::Terminal, "/", site());
    reader.start("/");
    assert!(blank(reader.screen()));
    reader.serve_only(|effect| matches!(effect, Effect::FetchPage { .. }));
    assert!(blank(reader.screen()));
    reader.serve();
    assert!(reader.screen().contains("Hi, I am Nathan."));

    // Without the index, the page shows anyway, and says why there's no
    // nav.
    let mut files = site();
    files.retain(|(path, _)| path != "/site.json");
    let mut reader = Reader::new(Host::Terminal, "/", files);
    reader.start("/");
    reader.serve();
    let screen = reader.screen();
    assert!(screen.contains("Hi, I am Nathan."));
    assert!(screen.contains("Couldn't load the site index"));
}

#[test]
fn drafts_are_faded() {
    let mut reader = Reader::open(Host::Terminal, "/ramblings");
    assert_eq!(reader.color_of("Unfinished"), FAINT);
    assert_ne!(reader.color_of("First post"), FAINT);
    reader.click("Unfinished");
    assert!(reader.screen().contains(" Draft: Unfinished "));
}
