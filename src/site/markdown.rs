//! Markdown to HTML, with pulldown-cmark, reading the site's Markdown the
//! way kramdown did.
//!
//! ratmark's [`Kramdown`] does the rewriting kramdown needs first, and
//! hands out heading ids by kramdown's rule, so links to `#sections` keep
//! working. Attribute lists like `{:.no_toc}` give classes to the heading or
//! inline code before them. `$$` math is written for MathJax, as `\[ \]` on
//! its own and `\( \)` in a sentence. Code is highlighted, and `tikz` code
//! is drawn.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd};
use ratmark::kramdown::{AttributeList, HeaderIds, Kramdown};

use super::highlight::Highlighter;
use super::text::Text;
use super::tikz::Tikz;

/// Renders Markdown to HTML.
pub struct Markdown<'a> {
    pub highlighter: &'a Highlighter,
    pub tikz: &'a Tikz,
}

impl Markdown<'_> {
    /// The HTML for `markdown`.
    pub fn render(&self, markdown: &str) -> Result<String, String> {
        let markdown = Kramdown::to_commonmark(markdown);
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_SMART_PUNCTUATION
            | Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_MATH;
        let events: Vec<Event> = Parser::new_ext(&markdown, options).collect();
        let events = self.transform(events)?;
        let mut html = String::with_capacity(markdown.len() * 3 / 2);
        pulldown_cmark::html::push_html(&mut html, events.into_iter());
        Ok(html)
    }

    fn transform<'e>(&self, events: Vec<Event<'e>>) -> Result<Vec<Event<'e>>, String> {
        let mut out: Vec<Event> = Vec::with_capacity(events.len());
        let mut ids = HeaderIds::default();
        let mut i = 0;
        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::Heading { .. }) => {
                    let end = Self::end_of(&events, i);
                    let Event::Start(Tag::Heading {
                        level,
                        id,
                        mut classes,
                        attrs,
                    }) = events[i].clone()
                    else {
                        unreachable!()
                    };
                    let text = Self::plain_text(&events[i + 1..end]);
                    let id = id.unwrap_or_else(|| ids.next(&text).into());
                    // An attribute list on the next line, read as its own
                    // paragraph, belongs to the heading.
                    let mut next = end + 1;
                    if let Some(list) = Self::lone_attribute_list(&events, next) {
                        let (list, _) = AttributeList::split(&list);
                        classes.extend(list.classes.iter().map(|c| CowStr::from(c.to_string())));
                        next += 3;
                    }
                    out.push(Event::Start(Tag::Heading {
                        level,
                        id: Some(id),
                        classes,
                        attrs,
                    }));
                    out.extend(self.transform(events[i + 1..end].to_vec())?);
                    out.push(events[end].clone());
                    i = next;
                    continue;
                }
                Event::Start(Tag::Paragraph) => {
                    // A paragraph of only an attribute list is dropped.
                    if Self::lone_attribute_list(&events, i).is_some() {
                        i += 3;
                        continue;
                    }
                    // So is one of only `$$` math, which becomes a block.
                    if let Some(math) = Self::lone_math(&events, i) {
                        out.push(Event::Html(
                            format!("\\[{}\\]\n", Text::escape(math.trim())).into(),
                        ));
                        i = Self::end_of(&events, i) + 1;
                        continue;
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let end = Self::end_of(&events, i);
                    let code: String = events[i + 1..end]
                        .iter()
                        .filter_map(|e| match e {
                            Event::Text(t) => Some(t.as_ref()),
                            _ => None,
                        })
                        .collect();
                    let info = match kind {
                        CodeBlockKind::Fenced(info) => info.as_ref(),
                        CodeBlockKind::Indented => "",
                    };
                    let (lang, label) = info.split_once(' ').unwrap_or((info, ""));
                    let html = if lang == "tikz" {
                        self.tikz.figure(&code, label.trim())?
                    } else {
                        self.highlighter.block(lang, &code)
                    };
                    out.push(Event::Html(html.into()));
                    i = end + 1;
                    continue;
                }
                Event::Code(code) => {
                    // An attribute list right after inline code gives it
                    // classes.
                    if let Some(Event::Text(text)) = events.get(i + 1)
                        && text.starts_with("{:")
                    {
                        let (list, rest) = AttributeList::split(text);
                        let classes = list.classes.join(" ");
                        out.push(Event::InlineHtml(
                            format!("<code class=\"{classes}\">{}</code>", Text::escape(code))
                                .into(),
                        ));
                        if !rest.is_empty() {
                            out.push(Event::Text(rest.to_owned().into()));
                        }
                        i += 2;
                        continue;
                    }
                }
                // kramdown shows an email address without its `mailto:`.
                Event::Start(Tag::Link {
                    link_type: LinkType::Email | LinkType::Autolink,
                    ..
                }) => {
                    out.push(events[i].clone());
                    if let Some(Event::Text(text)) = events.get(i + 1)
                        && let Some(address) = text.strip_prefix("mailto:")
                    {
                        out.push(Event::Text(address.to_owned().into()));
                        i += 1;
                    }
                    i += 1;
                    continue;
                }
                Event::Text(text) if text.starts_with("{:") => {
                    let rest = AttributeList::split(text).1;
                    if !rest.is_empty() {
                        out.push(Event::Text(rest.to_owned().into()));
                    }
                    i += 1;
                    continue;
                }
                Event::InlineMath(math) => {
                    // kramdown only reads `$$` as math, so a lone `$` is a
                    // dollar sign.
                    out.push(Event::Text(format!("${math}$").into()));
                    i += 1;
                    continue;
                }
                Event::DisplayMath(math) => {
                    out.push(Event::InlineHtml(
                        format!("\\({}\\)", Text::escape(math.trim())).into(),
                    ));
                    i += 1;
                    continue;
                }
                Event::Html(html) | Event::InlineHtml(html) => {
                    let html: CowStr = Self::tidy_html(html).into();
                    out.push(match &events[i] {
                        Event::Html(_) => Event::Html(html),
                        _ => Event::InlineHtml(html),
                    });
                    i += 1;
                    continue;
                }
                // Tables a row and a cell to a line, like kramdown's, so
                // their words are counted apart.
                Event::Start(Tag::Table(_) | Tag::TableHead | Tag::TableRow)
                | Event::End(
                    TagEnd::TableCell | TagEnd::TableRow | TagEnd::TableHead | TagEnd::Table,
                ) => {
                    out.push(events[i].clone());
                    out.push(Event::Html("\n".into()));
                    i += 1;
                    continue;
                }
                _ => {}
            }
            out.push(events[i].clone());
            i += 1;
        }
        Ok(out)
    }

    /// Raw HTML as kramdown writes it back out: without `markdown="1"`,
    /// and with a `<` that doesn't start a tag escaped, like in code.
    fn tidy_html(html: &str) -> String {
        let html = html.replace(" markdown=\"1\"", "");
        let mut out = String::with_capacity(html.len());
        let mut chars = html.chars().peekable();
        while let Some(c) = chars.next() {
            let tag = chars
                .peek()
                .is_some_and(|n| n.is_ascii_alphabetic() || matches!(n, '/' | '!' | '?'));
            if c == '<' && !tag {
                out.push_str("&lt;");
            } else {
                out.push(c);
            }
        }
        out
    }

    /// The index of the event ending the element started at `start`.
    fn end_of(events: &[Event], start: usize) -> usize {
        let mut depth = 0;
        for (i, event) in events.iter().enumerate().skip(start) {
            match event {
                Event::Start(_) => depth += 1,
                Event::End(_) => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        events.len() - 1
    }

    /// The text of inline events, without formatting.
    fn plain_text(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|e| match e {
                Event::Text(t) | Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                    Some(t.as_ref())
                }
                _ => None,
            })
            .collect()
    }

    /// The attribute list making up the whole paragraph starting at `i`.
    fn lone_attribute_list(events: &[Event], i: usize) -> Option<String> {
        match events.get(i..i + 3)? {
            [
                Event::Start(Tag::Paragraph),
                Event::Text(text),
                Event::End(TagEnd::Paragraph),
            ] if text.starts_with("{:") && AttributeList::split(text).1.trim().is_empty() => {
                Some(text.to_string())
            }
            _ => None,
        }
    }

    /// The `$$` math making up the whole paragraph starting at `i`.
    fn lone_math<'e>(events: &'e [Event], i: usize) -> Option<&'e str> {
        let end = Self::end_of(events, i);
        let mut math = None;
        for event in &events[i + 1..end] {
            match event {
                Event::DisplayMath(m) if math.is_none() => math = Some(m.as_ref()),
                Event::SoftBreak => {}
                Event::Text(t) if t.trim().is_empty() => {}
                _ => return None,
            }
        }
        math
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(markdown: &str) -> String {
        let highlighter = Highlighter::default();
        let tikz = Tikz::new(std::env::temp_dir());
        Markdown {
            highlighter: &highlighter,
            tikz: &tikz,
        }
        .render(markdown)
        .unwrap()
    }

    #[test]
    fn headings() {
        assert_eq!(
            render("# Title\n{:.no_toc}\n\n## One `a`\n\n## One `a`\n"),
            "<h1 id=\"title\" class=\"no_toc\">Title</h1>\n<h2 id=\"one-a\">One <code>a</code></h2>\n<h2 id=\"one-a-1\">One <code>a</code></h2>\n"
        );
    }

    #[test]
    fn code_classes() {
        assert_eq!(
            render("`x`{:.new-syntax}: y\n"),
            "<p><code class=\"new-syntax\">x</code>: y</p>\n"
        );
    }

    #[test]
    fn math() {
        assert_eq!(render("$$\na < b\n$$\n"), "\\[a &lt; b\\]\n");
        assert_eq!(render("so $$x$$, $5\n"), "<p>so \\(x\\), $5</p>\n");
    }
}
