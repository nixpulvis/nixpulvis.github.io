//! The site's index, published at `/site.json`, which the generator writes
//! and the terminal view reads.

use serde::{Deserialize, Serialize};

/// The nav and every page in a collection, for clients like the terminal
/// view to browse the site without reading its HTML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Index {
    pub title: String,
    /// The site's own address, like `https://nixpulvis.com`.
    pub url: String,
    /// What every path starts with, empty except in previews.
    #[serde(default)]
    pub baseurl: String,
    pub nav: Vec<Nav>,
    pub pages: Vec<Entry>,
}

impl Index {
    /// Whether the page at `path`, from the site's root, is a draft.
    pub fn is_draft(&self, path: &str) -> bool {
        self.pages
            .iter()
            .any(|p| p.draft && p.url.trim_end_matches('/') == path.trim_end_matches('/'))
    }
}

/// An entry in the site's nav, from `_data/nav.yml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nav {
    pub title: String,
    pub url: String,
    /// Shown in place of the title, in the header.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// False for pages that only work in a browser, left out of the
    /// terminal's nav.
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub tui: bool,
}

/// A page in one of the site's collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    /// Like `2026-10-19`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    #[serde(default, skip_serializing_if = "is_no")]
    pub draft: bool,
    pub url: String,
}

fn yes() -> bool {
    true
}

fn is_yes(b: &bool) -> bool {
    *b
}

fn is_no(b: &bool) -> bool {
    !*b
}
