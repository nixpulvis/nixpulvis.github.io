//! A page of the site, read from its published Markdown.

use ratatui::text::Span;
use ratmark::MarkdownState;
use serde::Deserialize;

use crate::markup;

pub(crate) struct Page {
    pub path: String,
    /// Its full address, like `https://nixpulvis.com/ramblings`, for links
    /// that work outside the app.
    pub url: String,
    /// The whole page as the terminal shows it, before any tag filter.
    pub markdown: String,
    pub title: String,
    /// When it was published or drafted, like `October 19, 2026`.
    date: Option<String>,
    pub draft: bool,
    /// The word count, on pages whose layout shows one.
    words: Option<u32>,
    /// A project's status, like `complete`.
    pub status: Option<String>,
    pub view: MarkdownState,
}

impl Page {
    /// The page at `path`, with the full address `url`, from its Markdown,
    /// and what went wrong reading its front matter, if anything. Front
    /// matter that doesn't fit leaves the page with its defaults.
    pub fn new(path: &str, url: String, markdown: &str) -> (Self, Option<String>) {
        let front_matter = ratmark::front_matter::FrontMatter::split(markdown).0;
        let (front, error) = match front_matter.map(|f| f.deserialize::<FrontMatter>()) {
            None => (FrontMatter::default(), None),
            Some(Ok(front)) => (front, None),
            Some(Err(e)) => (
                FrontMatter::default(),
                Some(format!("Couldn't read {path}'s front matter: {e}")),
            ),
        };
        let markdown = markup::adapt(markdown);
        let page = Self {
            path: path.to_owned(),
            view: view(&url, markdown.clone()),
            url,
            markdown,
            title: front.title.unwrap_or_else(|| path.to_owned()),
            date: front.date.map(|d| long_date(&d).unwrap_or(d)),
            draft: front.draft,
            words: front.words,
            status: front.status,
        };
        (page, error)
    }

    /// Goes to the section `anchor` names. On a page of tagged items, like
    /// the references, a tag's link shows only the items with that tag
    /// instead, as the site does with CSS.
    pub fn jump(&mut self, anchor: &str) {
        match markup::filter_tagged(&self.markdown, anchor) {
            Some(markdown) => {
                let scroll = self.view.scroll();
                self.view = view(&self.url, markdown);
                self.view.scroll_to(scroll);
            }
            None => self.view.scroll_to_anchor(anchor),
        }
    }

    /// The line under a post, worded like the site's, as in `7121 words
    /// published on October 19, 2026`, or a shorter wording when that's
    /// wider than `width`, dropping the word count first.
    pub fn meta(&self, width: usize) -> Option<String> {
        let verb = if self.draft { "drafted" } else { "published" };
        let mut wordings = Vec::new();
        if let Some(words) = &self.words {
            wordings.push(match &self.date {
                Some(date) => format!("{words} words {verb} on {date}"),
                None => format!("{words} words"),
            });
        }
        if let Some(date) = &self.date {
            wordings.push(format!(
                "{}{} on {date}",
                verb[..1].to_uppercase(),
                &verb[1..]
            ));
        }
        if self.draft {
            wordings.push("Draft".to_owned());
        }
        wordings
            .into_iter()
            .find(|w| Span::raw(w.as_str()).width() <= width)
    }
}

/// The front matter the Jekyll build publishes each page's Markdown with,
/// as far as the app uses it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct FrontMatter {
    /// The page's title, shown in the header. Pages without one use their
    /// path.
    title: Option<String>,
    /// When the page was published or drafted, as `YYYY-MM-DD`, on dated
    /// pages.
    date: Option<String>,
    /// Whether the page is a draft, which is dimmed and labeled `Draft`.
    draft: bool,
    /// The word count, on pages whose layout shows one.
    words: Option<u32>,
    /// A project's status, like `complete`.
    status: Option<String>,
}

/// `markdown` ready to draw, for the page at `url`.
fn view(url: &str, markdown: String) -> MarkdownState {
    MarkdownState::new(markdown).with_link_base(url)
}

/// Writes a `YYYY-MM-DD` date out like the site does, as `October 19, 2026`.
fn long_date(date: &str) -> Option<String> {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let mut parts = date.splitn(3, '-');
    let year = parts.next()?;
    let month: usize = parts.next()?.parse().ok()?;
    let day = parts.next()?.get(..2)?;
    Some(format!(
        "{} {day}, {year}",
        MONTHS.get(month.checked_sub(1)?)?
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(markdown: &str) -> Page {
        Page::new("/x", "https://nixpulvis.com/x".into(), markdown).0
    }

    #[test]
    fn front_matter() {
        let page = load("---\ntitle: 'It''s: here'\ndate: '2023-09-15'\ndraft: true\n---\n\nhi");
        assert_eq!(page.title, "It's: here");
        assert_eq!(page.date.as_deref(), Some("September 15, 2023"));
        assert!(page.draft);
        assert_eq!(
            page.meta(usize::MAX).as_deref(),
            Some("Drafted on September 15, 2023")
        );

        let page = load("---\ndate: '2026-10-19'\nwords: 7121\n---\n");
        assert_eq!(
            page.meta(usize::MAX).as_deref(),
            Some("7121 words published on October 19, 2026")
        );
        assert_eq!(
            page.meta(30).as_deref(),
            Some("Published on October 19, 2026")
        );
        assert_eq!(page.meta(20), None);

        let page = load("# no front matter");
        assert_eq!(
            (page.title.as_str(), page.date, page.draft),
            ("/x", None, false)
        );
    }

    #[test]
    fn bad_front_matter_is_reported() {
        let markdown = "---\ntitle: [a]\n---\n\nhi";
        let (page, error) = Page::new("/x", "https://nixpulvis.com/x".into(), markdown);
        assert_eq!(page.title, "/x");
        let expected = ratmark::front_matter::FrontMatter::split(markdown)
            .0
            .unwrap()
            .deserialize::<FrontMatter>()
            .unwrap_err();
        assert_eq!(
            error,
            Some(format!("Couldn't read /x's front matter: {expected}"))
        );
    }

    #[test]
    fn long_dates() {
        assert_eq!(long_date("2026-10-19").as_deref(), Some("October 19, 2026"));
        assert_eq!(long_date("2026-13-01"), None);
        assert_eq!(long_date("soon"), None);
    }
}
