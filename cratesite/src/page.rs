//! A page of a site, read from its [`Source`].

use chrono::NaiveDate;
use serde::Deserialize;
use serde::de::{DeserializeOwned, IgnoredAny};

use crate::text::Text;
use crate::{Collection, Config, Name, Source};

/// A page, with its front matter read and its Markdown rendered. `E` is
/// the site's own front matter, see [`FrontMatter`].
#[derive(Debug)]
pub struct Page<E = IgnoredAny> {
    pub source: &'static Source,
    /// Where it's published, from the site's root, like
    /// `/ramblings/2018-02-05-hello-world` or `/about.html`.
    pub url: String,
    pub collection: Option<&'static Collection>,
    pub front: FrontMatter<E>,
    /// From the front matter or the file's name. Undated pages in a
    /// collection sort as if they were written at build time.
    pub date: Option<NaiveDate>,
    /// The page's Markdown, after its template tags have been rendered, if
    /// it has any.
    pub markdown: String,
    /// The Markdown, rendered to HTML, without any layout.
    pub content: String,
    /// The first paragraph, rendered, for pages in a collection.
    pub excerpt: String,
}

/// What a page says about itself, between the `---` lines at its top.
///
/// The fields here are the ones the generator reads. Anything else a site
/// wants from its pages goes in `extra`, a struct of its own, read from the
/// same front matter.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FrontMatter<E = IgnoredAny> {
    pub title: Option<String>,
    /// The layout the page is wrapped in, or none.
    pub layout: Option<String>,
    /// Like `2019-10-30`, for pages without one in their file's name.
    pub date: Option<String>,
    /// Where it's published, in place of where its path says.
    pub permalink: Option<String>,
    /// Still being written.
    pub draft: bool,
    /// False to leave the page out of the site, unless building with
    /// unpublished pages.
    pub published: bool,
    /// A summary for link previews, in place of the first paragraph.
    pub description: Option<String>,
    /// The site's own fields.
    #[serde(flatten)]
    pub extra: E,
}

impl<E: Default> Default for FrontMatter<E> {
    fn default() -> Self {
        Self {
            title: None,
            layout: None,
            date: None,
            permalink: None,
            draft: false,
            published: true,
            description: None,
            extra: E::default(),
        }
    }
}

impl<E: DeserializeOwned + Default> Page<E> {
    /// Reads the page's front matter, leaving its Markdown to render.
    pub fn read(config: &Config, source: &'static Source) -> Result<Self, String> {
        let (yaml, body) = FrontMatter::split(source.text);
        let front: FrontMatter<E> = if yaml.trim().is_empty() {
            FrontMatter::default()
        } else {
            serde_yaml::from_str(yaml).map_err(|e| format!("{}: {e}", source.path))?
        };
        let name = source.path.rsplit('/').next().unwrap_or(source.path);
        let date = front
            .date
            .as_deref()
            .or_else(|| config.holding(source.path).and(Name::date(name)))
            .and_then(|d| NaiveDate::parse_from_str(d.get(..10)?, "%Y-%m-%d").ok());
        let url = front
            .permalink
            .clone()
            .unwrap_or_else(|| config.permalink(source.path));
        Ok(Self {
            source,
            url,
            collection: config.holding(source.path),
            front,
            date,
            markdown: body.to_owned(),
            content: String::new(),
            excerpt: String::new(),
        })
    }
}

impl<E> Page<E> {
    /// The page's title, or for pages in a collection, one made from the
    /// file's name, like `Hello World`.
    pub fn title(&self) -> String {
        match (&self.front.title, &self.collection) {
            (Some(title), _) => title.clone(),
            (None, Some(_)) => Name::title(self.stem()),
            (None, None) => String::new(),
        }
    }

    /// Whether the page has template tags, rendered before its Markdown.
    pub fn is_template(&self) -> bool {
        let text = self.source.text;
        text.contains("{%") || text.contains("{{")
    }

    /// The title as a slug, for the heading's id.
    pub fn title_slug(&self) -> String {
        Name::slug(&self.title())
    }

    /// The file's name, without its folder or extension.
    pub fn stem(&self) -> &'static str {
        let name = self
            .source
            .path
            .rsplit('/')
            .next()
            .unwrap_or(self.source.path);
        name.rsplit_once('.').map_or(name, |(stem, _)| stem)
    }

    /// Whether the page is Markdown, rendered to HTML.
    pub fn is_markdown(&self) -> bool {
        self.source.path.ends_with(".md")
    }

    /// Where the page's HTML is written, from the top of `_site`.
    pub fn output_path(&self) -> String {
        let url = self.url.trim_start_matches('/');
        if url.is_empty() || url.ends_with('/') {
            format!("{url}index.html")
        } else if !self.is_markdown() || url.rsplit('/').next().unwrap_or(url).contains('.') {
            url.to_owned()
        } else {
            format!("{url}.html")
        }
    }

    /// Where the page's Markdown is published, like `/ramblings/foo.md` or
    /// `/fortunes/index.md`.
    pub fn markdown_url(&self) -> Option<String> {
        if !self.is_markdown() {
            return None;
        }
        let url = if self.url.ends_with('/') {
            format!("{}index", self.url)
        } else {
            self.url.clone()
        };
        Some(format!("{}.md", url.strip_suffix(".html").unwrap_or(&url)))
    }

    /// The date like `October 19, 2026`.
    pub fn long_date(&self) -> String {
        self.date
            .map(|d| d.format("%B %d, %Y").to_string())
            .unwrap_or_default()
    }

    /// How many words the rendered page has.
    pub fn words(&self) -> usize {
        Text::number_of_words(&Text::strip_html(&self.content))
    }

    /// The first paragraph of a page in a collection, as Markdown, with the
    /// definitions of any reference links it uses, the way Jekyll cuts
    /// excerpts.
    pub fn excerpt_markdown(&self) -> Option<String> {
        self.collection?;
        let Some((head, tail)) = self.markdown.split_once("\n\n") else {
            return Some(self.markdown.clone());
        };
        let mut head = head.to_owned();
        let definitions: Vec<&str> = tail
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                line.len() - trimmed.len() <= 3
                    && trimmed.starts_with('[')
                    && trimmed
                        .find("]:")
                        .is_some_and(|end| end > 1 && head.contains(&trimmed[..end + 1]))
            })
            .collect();
        if !definitions.is_empty() {
            head.push_str("\n\n");
            head.push_str(&definitions.join("\n"));
        }
        Some(head)
    }
}

impl FrontMatter {
    /// Splits the YAML between the `---` lines at the top of `text` off the
    /// rest. Without them, it's all the rest.
    pub fn split(text: &str) -> (&str, &str) {
        crate::front::split(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SOURCE: Source = Source {
        path: "_projects/maze-gl.md",
        text: "---\nlayout: project\n---\n\nA [maze][m] game.\n\nMore.\n\n[m]: https://x\n[n]: https://y\n",
    };

    #[test]
    fn reads_pages() {
        let config = Config {
            title: "",
            url: "",
            description: "",
            repository: None,
            collections: &[Collection {
                label: "projects",
                permalink: "/projects/:name",
            }],
            exclude: &[],
            unlisted: &[],
            keep: &[],
            tikz_preamble: "",
        };
        let page: Page = Page::read(&config, &SOURCE).unwrap();
        assert_eq!(page.title(), "Maze Gl");
        assert_eq!(page.url, "/projects/maze-gl");
        assert_eq!(page.output_path(), "projects/maze-gl.html");
        assert_eq!(page.markdown_url().unwrap(), "/projects/maze-gl.md");
        assert_eq!(
            page.excerpt_markdown().unwrap(),
            "\nA [maze][m] game.\n\n[m]: https://x"
        );
    }
}
