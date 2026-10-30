//! Each page's Markdown, published next to its HTML for the terminal view
//! and anyone else who'd rather read it, like `/ramblings/foo.md`.
//!
//! It's the Markdown after its template tags are rendered, wrapped in the
//! page's layout, but not the site's default one with the header and nav.
//! Its front matter keeps what a reader might want, and adds the word
//! count and collection.

use std::fmt;

use super::layout::Layout;
use super::{Page, Site};

/// A page's published Markdown.
pub struct Published {
    front: PublishedFrontMatter,
    body: String,
}

/// The front matter a page is published with.
#[derive(Debug, Default)]
pub struct PublishedFrontMatter {
    pub title: Option<String>,
    /// Like `2026-10-19`, only for pages with a date of their own.
    pub date: Option<String>,
    pub description: Option<String>,
    pub draft: bool,
    pub status: Option<String>,
    pub references: Vec<String>,
    /// For layouts that show a word count.
    pub words: Option<usize>,
    pub collection: Option<String>,
}

impl Published {
    pub fn new(site: &Site, page: &Page) -> Result<Self, String> {
        let layout = Layout::named(page.front.layout.as_deref());
        Ok(Self {
            front: Self::front_matter(site, page),
            body: layout.wrap(site, page, &page.markdown)?,
        })
    }

    pub fn front_matter(_site: &Site, page: &Page) -> PublishedFrontMatter {
        let title = page.title();
        PublishedFrontMatter {
            title: (!title.is_empty()).then_some(title),
            date: page.date.map(|d| d.format("%Y-%m-%d").to_string()),
            description: page.front.description.clone(),
            draft: page.front.draft,
            status: page.front.status.clone(),
            references: page.front.references.clone(),
            words: matches!(page.front.layout.as_deref(), Some("rambling" | "article"))
                .then(|| page.words()),
            collection: page.collection.map(|c| c.label.to_owned()),
        }
    }
}

impl fmt::Display for Published {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let front = &self.front;
        let mut yaml = String::new();
        let mut field = |key: &str, value: Option<String>| {
            if let Some(value) = value {
                yaml.push_str(&format!("{key}: {value}\n"));
            }
        };
        field("title", front.title.as_deref().map(Yaml::string));
        field("date", front.date.as_deref().map(Yaml::string));
        field(
            "description",
            front.description.as_deref().map(Yaml::string),
        );
        field("draft", front.draft.then(|| "true".to_owned()));
        field("status", front.status.as_deref().map(Yaml::string));
        if !front.references.is_empty() {
            yaml.push_str("references:\n");
            for reference in &front.references {
                yaml.push_str(&format!("- {}\n", Yaml::string(reference)));
            }
        }
        let mut field = |key: &str, value: Option<String>| {
            if let Some(value) = value {
                yaml.push_str(&format!("{key}: {value}\n"));
            }
        };
        field("words", front.words.map(|w| w.to_string()));
        field("collection", front.collection.as_deref().map(Yaml::string));
        if yaml.is_empty() {
            f.write_str(&self.body)
        } else {
            write!(f, "---\n{yaml}---\n\n{}", self.body)
        }
    }
}

/// Writing YAML the way Ruby's Psych does, which published it before.
struct Yaml;

impl Yaml {
    /// The string plain, or in single quotes where plain would read as
    /// something else.
    fn string(s: &str) -> String {
        let looks_like_other = s.is_empty()
            || s.parse::<f64>().is_ok()
            || ["true", "false", "yes", "no", "on", "off", "null", "~"]
                .contains(&s.to_ascii_lowercase().as_str())
            || (s.len() >= 10
                && s.as_bytes()[4] == b'-'
                && s[..4].bytes().all(|b| b.is_ascii_digit()));
        let special = s.contains(": ")
            || s.contains(" #")
            || s.ends_with(':')
            || s.starts_with(char::is_whitespace)
            || s.ends_with(char::is_whitespace)
            || s.starts_with([
                '-', '?', ':', ',', '[', ']', '{', '}', '#', '&', '*', '!', '|', '>', '\'', '"',
                '%', '@', '`',
            ]);
        if looks_like_other || special {
            format!("'{}'", s.replace('\'', "''"))
        } else {
            s.to_owned()
        }
    }
}
