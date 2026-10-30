//! The site's own markup in its published Markdown, reworked for a terminal.
//! The site's layouts and CSS do things a terminal can't, like floating a
//! project's status beside its title, so these say the same in text.

/// `body`, a page without its front matter, as the terminal shows it. The
/// title is left out, since the box's border shows it instead. Projects'
/// statuses go after their titles, and links to drafts are marked.
pub(crate) fn adapt(body: &str) -> String {
    draft_prefixes(&status_after_title(without_title(body)))
}

/// `markdown` with only the `<li data-tags="…">` items tagged `tag`, or
/// all of them for `all`. `None` when no item has the tag, so the link is
/// an ordinary one.
pub(crate) fn filter_tagged(markdown: &str, tag: &str) -> Option<String> {
    const ITEM: &str = "<li data-tags=\"";
    let tagged = |tags: &str| tag == "all" || tags.split_whitespace().any(|t| t == tag);
    let mut items = markdown.match_indices(ITEM).map(|(start, _)| {
        let rest = &markdown[start + ITEM.len()..];
        let tags = &rest[..rest.find('"').unwrap_or(0)];
        let end = markdown[start..]
            .find("</li>")
            .map_or(markdown.len(), |e| start + e + "</li>".len());
        (start, end, tagged(tags))
    });
    if !items.clone().any(|(_, _, keep)| keep) {
        return None;
    }
    let mut out = String::with_capacity(markdown.len());
    let mut at = 0;
    for (start, end, keep) in &mut items {
        if !keep {
            out.push_str(&markdown[at..start]);
            at = end;
        }
    }
    out.push_str(&markdown[at..]);
    Some(out)
}

/// `body` without the title its layout puts at the top, or the line under
/// it. Everything else the layout adds, like a project's references, stays.
fn without_title(body: &str) -> &str {
    let Some(rest) = [("<h1", "</h1>"), ("<h2", "</h2>")]
        .into_iter()
        .find_map(|(open, close)| skip_element(body.trim_start(), open, close))
    else {
        return body;
    };
    skip_element(rest.trim_start(), "<p class=\"meta\"", "</p>").unwrap_or(rest)
}

/// What follows the element `html` starts with, if it starts with `open`.
fn skip_element<'a>(html: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let end = html.strip_prefix(open)?.find(close)? + open.len();
    Some(&html[end + close.len()..])
}

/// Moves a project's status from before its title to after it, as small
/// text, so `complete Epic UI` reads `Epic UI complete`.
fn status_after_title(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut status = None;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<span class=\"status")
            && let Some(span) = trimmed.strip_suffix("</span>")
            && let Some(start) = span.find('>')
        {
            status = Some(span[start + 1..].to_owned());
            continue;
        }
        out.push_str(line);
        if !trimmed.is_empty()
            && let Some(status) = status.take()
        {
            out.push_str(&format!(" <small>{status}</small>"));
        }
        out.push('\n');
    }
    out
}

/// Starts the link in each draft's list item with `Draft: `, like a
/// draft's own title. The site fades these items, which says the same thing
/// in a browser.
fn draft_prefixes(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(start) = rest.find("<li") {
        let Some(end) = rest[start..].find('>').map(|i| start + i + 1) else {
            break;
        };
        let tag = &rest[start..end];
        out.push_str(&rest[..end]);
        rest = &rest[end..];
        let draft = tag
            .split_once("class=\"")
            .and_then(|(_, classes)| classes.split('"').next())
            .is_some_and(|classes| classes.split_whitespace().any(|c| c == "draft"));
        if draft
            && let Some(a) = rest.find("<a ")
            && let Some(close) = rest[a..].find('>').map(|i| a + i + 1)
        {
            out.push_str(&rest[..close]);
            out.push_str("Draft: ");
            rest = &rest[close..];
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_statuses_after_titles() {
        let markdown = r#"<li class="project">
    <span class="status complete">complete</span>
    <a class="name" href="/projects/epic-ui">Epic UI</a>
    <p>Text</p>
</li>
"#;
        assert_eq!(
            status_after_title(markdown),
            r#"<li class="project">
    <a class="name" href="/projects/epic-ui">Epic UI</a> <small>complete</small>
    <p>Text</p>
</li>
"#
        );
    }

    #[test]
    fn marks_draft_links() {
        let markdown = r#"<ul>
<li class="draft">
    <a href="/a">A</a>
</li>
<li>
    <a href="/b">B</a>
</li>
<li class="project draft">
    <a class="name" href="/c">C</a>
</li>
</ul>"#;
        assert_eq!(
            draft_prefixes(markdown),
            r#"<ul>
<li class="draft">
    <a href="/a">Draft: A</a>
</li>
<li>
    <a href="/b">B</a>
</li>
<li class="project draft">
    <a class="name" href="/c">Draft: C</a>
</li>
</ul>"#
        );
        assert_eq!(
            draft_prefixes("<link> <a href=x>y</a>"),
            "<link> <a href=x>y</a>"
        );
    }

    #[test]
    fn drops_the_layouts_title() {
        let body = r#"
<h2 class="title">
    Hi
</h2>

<ul class="references"></ul>

Text"#;
        assert_eq!(
            without_title(body),
            "\n\n<ul class=\"references\"></ul>\n\nText"
        );
        let body = r#"
<h2 class="title">Hi</h2>
<p class="meta">
    7 words
</p>

Text"#;
        assert_eq!(without_title(body), "\n\nText");
        assert_eq!(without_title("# Not a layout's\n"), "# Not a layout's\n");
    }

    #[test]
    fn filters_tagged_items() {
        let markdown = r#"<ul><li data-tags="rust">a</li><li data-tags="unix rust">b</li><li data-tags="unix">c</li></ul>"#;
        assert_eq!(
            filter_tagged(markdown, "unix").as_deref(),
            Some(r#"<ul><li data-tags="unix rust">b</li><li data-tags="unix">c</li></ul>"#)
        );
        assert_eq!(filter_tagged(markdown, "all").as_deref(), Some(markdown));
        assert_eq!(filter_tagged(markdown, "intro"), None);
    }
}
