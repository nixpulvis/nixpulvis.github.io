//! Building a site from its pages.
//!
//! [`Site::load`] reads every page's front matter, and the site's
//! [`Theme`] builds it with [`Theme::build`]:
//!
//! 1. Pages with template tags are rendered by [`Theme::template`] first,
//!    after the others, so they can list them with their word counts.
//! 2. Every page's Markdown is rendered to HTML by [`Markdown`].
//! 3. The page is wrapped in its layout, by [`Theme::layout`], and then in
//!    the document around it, by [`Theme::document`].
//! 4. The Markdown, wrapped in its layout but not the document, is
//!    published next to the HTML, like `/posts/foo.md`.
//!
//! Everything else in the crate that's part of the site, like images and
//! CSS, is copied over as it is.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde::de::{DeserializeOwned, IgnoredAny};

use crate::highlight::Highlighter;
use crate::markdown::Markdown;
use crate::page::{FrontMatter, Page};
use crate::publish::Yaml;
use crate::text::Text;
use crate::tikz::Tikz;
use crate::{Config, Source};

/// A site and its pages. `E` is the site's own front matter, see
/// [`FrontMatter`].
pub struct Site<E = IgnoredAny> {
    pub config: &'static Config,
    pub options: Options,
    /// When the build started, which dates undated pages.
    pub time: DateTime<Local>,
    /// Every page, those in collections sorted by date, then by path.
    pub pages: Vec<Page<E>>,
}

/// How to build a site.
#[derive(Debug, Clone)]
pub struct Options {
    /// The top of the site's crate, with the files copied into the site.
    pub source: PathBuf,
    /// Where the site is written, cleared first.
    pub destination: PathBuf,
    /// What every path on the site starts with, for previews under a
    /// path, like `/pr-preview/pr-83`.
    pub baseurl: String,
    /// Build pages marked `published: false` too.
    pub unpublished: bool,
}

/// What a site makes of its pages: its templates and layouts, and any
/// files of its own. A site implements it on a type of its own holding its
/// [`Site`], and anything else its templates need, and builds with
/// [`Theme::build`].
pub trait Theme: Sized {
    /// The site's own front matter fields, see [`FrontMatter`].
    type Extra: DeserializeOwned + Default;

    fn site(&self) -> &Site<Self::Extra>;

    fn site_mut(&mut self) -> &mut Site<Self::Extra>;

    /// The page's Markdown with its template tags rendered, for a page
    /// with them, front matter and all. The build script makes an askama
    /// template for each such page, so this is usually its generated
    /// `render`.
    fn template(&self, page: &Page<Self::Extra>) -> Option<Result<String, String>>;

    /// `content`, the page's HTML or its Markdown to publish, wrapped in
    /// its layout, like a title and date above it. Unchanged by default.
    fn layout(&self, page: &Page<Self::Extra>, content: &str) -> Result<String, String> {
        let _ = page;
        Ok(content.to_owned())
    }

    /// The page's HTML, already in its layout, as a whole document, with
    /// the site's header and footer. Unchanged by default.
    fn document(&self, page: &Page<Self::Extra>, content: &str) -> Result<String, String> {
        let _ = page;
        Ok(content.to_owned())
    }

    /// The front matter the page's Markdown is published with.
    fn front_matter(&self, page: &Page<Self::Extra>) -> Yaml {
        let title = page.title();
        let date = page.date.map(|d| d.format("%Y-%m-%d").to_string());
        Yaml::new()
            .string("title", (!title.is_empty()).then_some(title.as_str()))
            .string("date", date.as_deref())
            .string("description", page.front.description.as_deref())
            .flag("draft", page.front.draft)
    }

    /// More files to write, from the top of the site, like a feed or an
    /// index of the pages.
    fn files(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        Ok(Vec::new())
    }

    /// Renders every page and writes the site.
    fn build(&mut self) -> Result<(), String> {
        self.render()?;
        self.write()
    }

    /// Renders the templates, then the Markdown, of every page.
    #[doc(hidden)]
    fn render(&mut self) -> Result<(), String> {
        let site = self.site();
        let highlighter = Highlighter::default();
        let tikz = Tikz::new(
            site.options.source.join(".site-cache/tikz"),
            site.config.tikz_preamble,
        );
        let markdown = Markdown {
            highlighter: &highlighter,
            tikz: &tikz,
        };
        let render = |page: &mut Page<Self::Extra>| -> Result<(), String> {
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
        for page in self.site_mut().pages.iter_mut() {
            if !page.is_template() {
                render(page)?;
            }
        }
        let mut rendered = Vec::new();
        for (i, page) in self.site().pages.iter().enumerate() {
            if let Some(text) = self.template(page) {
                let text = text.map_err(|e| format!("{}: {e}", page.source.path))?;
                rendered.push((i, FrontMatter::split(&text).1.to_owned()));
            }
        }
        for (i, text) in rendered {
            let page = &mut self.site_mut().pages[i];
            page.markdown = text;
            render(page)?;
        }
        Ok(())
    }

    /// Writes the site into the destination, which is cleared first, all
    /// but what the config says to keep.
    #[doc(hidden)]
    fn write(&self) -> Result<(), String> {
        let site = self.site();
        site.clear()?;
        site.copy_files(&site.options.source)?;
        for page in &site.pages {
            let out = if page.is_markdown() {
                let content = self.layout(page, &page.content)?;
                match page.front.layout.as_deref() {
                    None | Some("null") => content,
                    Some(_) => self.document(page, &content)?,
                }
            } else {
                page.markdown.clone()
            };
            site.write_file(&page.output_path(), site.with_baseurl(&out))?;
            if let Some(url) = page.markdown_url() {
                let body = self.layout(page, &page.markdown)?;
                let published = self.front_matter(page).with_body(&body);
                site.write_file(&url, site.with_baseurl(&published))?;
            }
        }
        for (path, contents) in self.files()? {
            site.write_file(&path, contents)?;
        }
        // The site is already built, so stop GitHub Pages from running
        // Jekyll on it.
        site.write_file(".nojekyll", "")
    }
}

impl<E: DeserializeOwned + Default> Site<E> {
    /// Reads every page in `sources`, the build script's `SOURCES`, ready
    /// to build.
    pub fn load(
        config: &'static Config,
        sources: &[&'static Source],
        options: Options,
    ) -> Result<Self, String> {
        let time = Local::now();
        let mut pages = Vec::new();
        for source in sources {
            let page = Page::read(config, source)?;
            if page.front.published || options.unpublished {
                pages.push(page);
            }
        }
        // Undated pages sort as if written now.
        let today = time.date_naive();
        pages.sort_by(|a, b| {
            let key = |p: &Page<E>| (p.date.unwrap_or(today), p.source.path);
            key(a).cmp(&key(b))
        });
        Ok(Self {
            config,
            options,
            time,
            pages,
        })
    }
}

impl<E> Site<E> {
    /// The pages in the collection `label`, by date.
    pub fn collection(&self, label: &str) -> Vec<&Page<E>> {
        self.pages
            .iter()
            .filter(|p| p.collection.is_some_and(|c| c.label == label))
            .collect()
    }

    /// The pages in the collection `label`, newest first.
    pub fn newest_first(&self, label: &str) -> Vec<&Page<E>> {
        let mut pages = self.collection(label);
        pages.reverse();
        pages
    }

    /// `path`, from the site's root, with the base URL in front.
    pub fn relative_url(&self, path: &str) -> String {
        format!("{}{path}", self.options.baseurl)
    }

    /// `path`, from the site's root, as a full URL.
    pub fn absolute_url(&self, path: &str) -> String {
        format!("{}{}{path}", self.config.url, self.options.baseurl)
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

    /// The page's date, at midnight where the site's built, for feeds.
    pub fn xml_date(&self, page: &Page<E>) -> String {
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

    /// Writes `contents` to `path`, from the top of the destination.
    pub fn write_file(&self, path: &str, contents: impl AsRef<[u8]>) -> Result<(), String> {
        let file = self.options.destination.join(path.trim_start_matches('/'));
        fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(&file, contents).map_err(|e| format!("{}: {e}", file.display()))
    }

    /// Empties the destination, all but what the config says to keep.
    fn clear(&self) -> Result<(), String> {
        let Ok(entries) = fs::read_dir(&self.options.destination) else {
            return Ok(());
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if self
                .config
                .keep
                .iter()
                .any(|keep| entry.file_name() == *keep)
            {
                continue;
            }
            let removed = if path.is_dir() {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };
            removed.map_err(|e| format!("{}: {e}", path.display()))?;
        }
        Ok(())
    }

    /// Copies every file in `dir` that's part of the site, but isn't a
    /// page, into the same place in the destination.
    fn copy_files(&self, dir: &Path) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let relative = path
                .strip_prefix(&self.options.source)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if self.config.ignored(&relative)
                || self.pages.iter().any(|p| p.source.path == relative)
            {
                continue;
            }
            if path.is_dir() {
                self.copy_files(&path)?;
            } else {
                let to = self.options.destination.join(&relative);
                fs::create_dir_all(to.parent().unwrap()).map_err(|e| e.to_string())?;
                fs::copy(&path, &to).map_err(|e| format!("{relative}: {e}"))?;
            }
        }
        Ok(())
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
