//! A page of the site, read from its published Markdown.

use ratatui::text::Span;
use ratmark::MarkdownState;
use ratmark::front_matter::FrontMatter;

use crate::tui::markup;

pub(crate) struct Page {
    pub path: String,
    /// Its full address, like `https://nixpulvis.com/ramblings`, for links
    /// that work outside the app.
    #[cfg_attr(not(feature = "native-links"), allow(dead_code))]
    pub url: String,
    /// The whole page as the terminal shows it, before any tag filter.
    pub markdown: String,
    pub title: String,
    /// When it was published or drafted, like `October 19, 2026`.
    date: Option<String>,
    pub draft: bool,
    /// The word count, on pages whose layout shows one.
    words: Option<String>,
    /// A project's status, like `complete`.
    pub status: Option<String>,
    pub view: MarkdownState,
}

impl Page {
    /// The page at `path`, with the full address `url`, from its Markdown.
    pub fn new(path: &str, url: String, markdown: &str) -> Self {
        let front = FrontMatter::split(markdown).0;
        let get = |key| front.as_ref().and_then(|f| f.get(key));
        let markdown = markup::adapt(markdown);
        Self {
            path: path.to_owned(),
            view: view(&url, markdown.clone()),
            url,
            markdown,
            title: get("title").unwrap_or_else(|| path.to_owned()),
            date: get("date").map(|d| long_date(&d).unwrap_or(d)),
            draft: get("draft").is_some_and(|d| d == "true"),
            words: get("words"),
            status: get("status"),
        }
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

/// `markdown` ready to draw, for the page at `url`.
// Without native links, the page's address goes unused.
#[cfg_attr(not(feature = "native-links"), allow(unused_variables))]
fn view(url: &str, markdown: String) -> MarkdownState {
    let view = MarkdownState::new(markdown);
    #[cfg(feature = "native-links")]
    let view = view.with_base(url);
    view
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
        Page::new("/x", "https://nixpulvis.com/x".into(), markdown)
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
    fn long_dates() {
        assert_eq!(long_date("2026-10-19").as_deref(), Some("October 19, 2026"));
        assert_eq!(long_date("2026-13-01"), None);
        assert_eq!(long_date("soon"), None);
    }
}
