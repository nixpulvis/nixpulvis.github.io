//! The layouts pages are wrapped in, from the askama templates in
//! `templates/`.
//!
//! A page names its layout in its front matter. Every layout but
//! [`DefaultLayout`] wraps the page's content in a title and the like, and is
//! written in Markdown, so the published Markdown gets it too. Then
//! [`DefaultLayout`] wraps that in the site's header, nav and footer.

use askama::Template;

use super::text::Text;
use super::{Page, Site};

/// A page's layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Only the site's header and footer.
    Default,
    /// The title and word count, for mathematics.
    Article,
    /// The title and date, for notes.
    Note,
    /// The title as the page's heading, for index pages like Ramblings.
    Page,
    /// The title, status and links, for projects.
    Project,
    /// The title, word count and date, for ramblings.
    Rambling,
    /// Nothing at all, for pages like `feed.xml`.
    None,
}

impl Layout {
    /// The layout called `name` in a page's front matter.
    pub fn named(name: Option<&str>) -> Self {
        match name {
            Some("default") => Self::Default,
            Some("article") => Self::Article,
            Some("note") => Self::Note,
            Some("page") => Self::Page,
            Some("project") => Self::Project,
            Some("rambling") => Self::Rambling,
            _ => Self::None,
        }
    }

    /// `content`, the page's HTML or Markdown, wrapped in the layout, but
    /// not in [`DefaultLayout`].
    pub fn wrap(self, site: &Site, page: &Page, content: &str) -> Result<String, String> {
        let wrapped = match self {
            Self::Default | Self::None => return Ok(content.to_owned()),
            Self::Article => Article {
                site,
                page,
                content,
            }
            .render(),
            Self::Note => Note {
                site,
                page,
                content,
            }
            .render(),
            Self::Page => PageLayout {
                site,
                page,
                content,
            }
            .render(),
            Self::Project => Project {
                site,
                page,
                content,
            }
            .render(),
            Self::Rambling => Rambling {
                site,
                page,
                content,
            }
            .render(),
        };
        wrapped.map_err(|e| format!("{}: {e}", page.source.path))
    }
}

/// The site's header, nav and footer around every page.
#[derive(Template)]
#[template(path = "default.html", escape = "none")]
pub struct DefaultLayout<'a> {
    pub site: &'a Site,
    pub page: &'a Page,
    /// The page's HTML, in its own layout.
    pub content: &'a str,
}

impl DefaultLayout<'_> {
    /// The page's title, or the name of its file.
    fn title(&self) -> String {
        let title = self.page.title();
        if title.is_empty() {
            self.page
                .source
                .path
                .rsplit('/')
                .next()
                .unwrap_or("")
                .to_owned()
        } else {
            title
        }
    }

    /// A summary for search engines and link previews: the page's
    /// description, or its first paragraph, or the site's description.
    fn description(&self) -> String {
        let excerpt = Text::normalize_whitespace(&Text::strip_html(&self.page.excerpt));
        let excerpt = excerpt.trim();
        let description = match &self.page.front.description {
            Some(description) => description.as_str(),
            None if !excerpt.is_empty() => excerpt,
            None => Site::DESCRIPTION,
        };
        let description = Text::normalize_whitespace(&Text::strip_html(description));
        Text::escape_once(&Text::truncate(&description, 160))
    }

    /// Whether MathJax is needed, for `\(` or `\[` math.
    fn math(&self) -> bool {
        self.content.contains("\\(") || self.content.contains("\\[")
    }

    /// A link to open an issue about this page.
    fn issue_url(&self) -> String {
        format!(
            "{}/issues/new?body={}",
            Site::REPOSITORY,
            Text::url_encode(&self.site.absolute_url(&self.page.url))
        )
    }
}

macro_rules! layout {
    ($name:ident, $path:literal) => {
        #[derive(Template)]
        #[template(path = $path, escape = "none")]
        struct $name<'a> {
            #[allow(dead_code)]
            site: &'a Site,
            page: &'a Page,
            content: &'a str,
        }
    };
}

layout!(Article, "article.md");
layout!(Note, "note.md");
layout!(PageLayout, "page.md");
layout!(Project, "project.md");
layout!(Rambling, "rambling.md");
