//! The site's [`Theme`] for [cratesite], which builds `_site` from the
//! pages compiled into this crate.
//!
//! cratesite reads the pages, renders their Markdown and writes the site.
//! What's left is this site's own:
//!
//! - Pages with template tags, like [`ramblings`](crate::pages::ramblings), are
//!   askama templates, which can use the [`Site`] and its [`Data`].
//! - Pages are wrapped in their [`Layout`], and then [`DefaultLayout`], with
//!   the header and nav.
//! - The fortunes get their `strfile` databases, and `site.json`, the
//!   [`Index`], lists the pages for the terminal view.

use std::ops::Deref;

use serde::Deserialize;

mod data;
mod fortunes;
mod layout;

pub use data::{Data, Reference, Tags};
pub use fortunes::Fortunes;
pub use layout::{DefaultLayout, Layout};

use crate::CONFIG;
use crate::index::{Entry, Index};
use cratesite::{Name, Options, Theme, Yaml};

/// A page of this site.
pub type Page = cratesite::Page<Extra>;

/// This site's own front matter, beside what cratesite reads.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Extra {
    /// Moved off the Ramblings list, to its archive.
    pub archived: bool,
    /// Left out of every list.
    pub hidden: bool,
    /// A project's status, like `complete`.
    pub status: Option<String>,
    /// A project's links, listed under its title.
    pub references: Vec<String>,
    /// An image for link previews.
    pub image: Option<String>,
    /// A stylesheet just for this page.
    pub style: Option<String>,
    /// Scripts just for this page.
    pub scripts: Vec<String>,
}

/// The site, its pages and its data.
pub struct Site {
    site: cratesite::Site<Extra>,
    pub data: Data,
    pub fortunes: Vec<Fortunes>,
}

impl Site {
    pub const TITLE: &str = CONFIG.title;
    pub const URL: &str = CONFIG.url;
    pub const DESCRIPTION: &str = CONFIG.description;
    pub const REPOSITORY: &str = "https://github.com/nixpulvis/nixpulvis.github.io";

    /// Reads every page and data file, ready to [build](Theme::build).
    pub fn load(options: Options) -> Result<Self, String> {
        Ok(Self {
            data: Data::load()?,
            fortunes: Fortunes::read(&options.source.join("fortunes"))?,
            site: cratesite::Site::load(&CONFIG, crate::pages::SOURCES, options)?,
        })
    }

    /// The index published as `site.json`, of the nav and every page in a
    /// collection.
    pub fn index(&self) -> Index {
        let mut pages = Vec::new();
        for collection in CONFIG.collections {
            for page in self.collection(collection.label) {
                pages.push(Entry {
                    title: page.title(),
                    date: page.date.map(|d| d.format("%Y-%m-%d").to_string()),
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

    /// The ramblings and mathematics, newest first, as they're listed on
    /// the Ramblings page and in the feed.
    pub fn ramblings(&self) -> Vec<&Page> {
        let mut pages = self.collection("ramblings");
        pages.extend(self.collection("mathematics"));
        let today = self.time.date_naive();
        pages.sort_by_key(|p| std::cmp::Reverse(p.date.unwrap_or(today)));
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
}

impl Deref for Site {
    type Target = cratesite::Site<Extra>;

    fn deref(&self) -> &Self::Target {
        &self.site
    }
}

impl Theme for Site {
    type Extra = Extra;

    fn site(&self) -> &cratesite::Site<Extra> {
        &self.site
    }

    fn site_mut(&mut self) -> &mut cratesite::Site<Extra> {
        &mut self.site
    }

    fn template(&self, page: &Page) -> Option<Result<String, String>> {
        templates::render(page.source.path, self, page).map(|r| r.map_err(|e| e.to_string()))
    }

    fn layout(&self, page: &Page, content: &str) -> Result<String, String> {
        Layout::named(page.front.layout.as_deref()).wrap(self, page, content)
    }

    fn document(&self, page: &Page, content: &str) -> Result<String, String> {
        use askama::Template as _;
        DefaultLayout {
            site: self,
            page,
            content,
        }
        .render()
        .map_err(|e| e.to_string())
    }

    fn front_matter(&self, page: &Page) -> Yaml {
        let title = page.title();
        let date = page.date.map(|d| d.format("%Y-%m-%d").to_string());
        // Only the layouts that show a word count.
        let words = matches!(page.front.layout.as_deref(), Some("rambling" | "article"))
            .then(|| page.words());
        Yaml::new()
            .string("title", (!title.is_empty()).then_some(title.as_str()))
            .string("date", date.as_deref())
            .string("description", page.front.description.as_deref())
            .flag("draft", page.front.draft)
            .string("status", page.front.extra.status.as_deref())
            .list("references", &page.front.extra.references)
            .number("words", words)
            .string("collection", page.collection.map(|c| c.label))
    }

    fn files(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        let mut files: Vec<(String, Vec<u8>)> = self
            .fortunes
            .iter()
            .map(|year| (year.dat(), year.strfile()))
            .collect();
        let index = serde_json::to_string_pretty(&self.index()).map_err(|e| e.to_string())?;
        files.push(("site.json".to_owned(), index.into_bytes()));
        Ok(files)
    }
}

/// What the templates ask of a page, beyond what cratesite gives it.
pub trait Post {
    /// `drafted` or `published`.
    fn verb(&self) -> &'static str;

    /// The status as a CSS class, like `partially-complete`.
    fn status_class(&self) -> String;
}

impl Post for Page {
    fn verb(&self) -> &'static str {
        if self.front.draft {
            "drafted"
        } else {
            "published"
        }
    }

    fn status_class(&self) -> String {
        self.front
            .extra
            .status
            .as_deref()
            .map(Name::slug)
            .unwrap_or_default()
    }
}

/// The templates for pages with template tags, made by cratesite's build
/// script.
mod templates {
    use askama::Template;

    use super::{Page, Post as _, Site};

    include!(concat!(env!("OUT_DIR"), "/templates.rs"));
}
