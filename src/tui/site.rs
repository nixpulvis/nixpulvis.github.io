//! The site's index, `site.json`, which the generator writes as [`crate::index::Index`].

use serde::Deserialize;

/// What the app needs from the index: the nav, and which pages are drafts.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct Site {
    pub nav: Vec<Nav>,
    pages: Vec<Listing>,
}

impl Site {
    /// Reads the index, leaving out the nav's pages that only work in a
    /// browser.
    pub(crate) fn parse(json: &str) -> Result<Self, String> {
        let mut site: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        site.nav.retain(|nav| nav.tui);
        Ok(site)
    }

    /// Whether the page at `path`, from the site's root, is a draft.
    pub(crate) fn is_draft(&self, path: &str) -> bool {
        self.pages
            .iter()
            .any(|p| p.draft && crate::tui::link::same_page(&p.url, path))
    }
}

/// A page in the site's nav.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Nav {
    pub title: String,
    pub url: String,
    /// Shown in place of the title, like `λ`.
    pub icon: Option<String>,
    /// False for pages that only work in a browser.
    #[serde(default = "yes")]
    tui: bool,
}

/// A page the index lists.
#[derive(Debug, Clone, Deserialize)]
struct Listing {
    url: String,
    #[serde(default)]
    draft: bool,
}

fn yes() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_index() {
        let site = Site::parse(
            r#"{
                "title": "t", "url": "",
                "nav": [{"title": "A", "url": "/a"}, {"title": "B", "url": "/b", "tui": false}],
                "pages": [{"title": "D", "url": "/d/", "draft": true}, {"title": "P", "url": "/p"}]
            }"#,
        )
        .unwrap();
        assert_eq!(site.nav.len(), 1);
        assert!(site.is_draft("/d"));
        assert!(!site.is_draft("/p"));
        assert!(Site::parse("{").is_err());
    }
}
