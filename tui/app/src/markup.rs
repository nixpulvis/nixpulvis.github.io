//! The site's own markup in its published Markdown, reworked for a terminal.
//! The site's layouts and CSS do things a terminal can't, like floating a
//! project's status beside its title, so these say the same in text.

/// `markdown` as the terminal shows it: without the title the box's border
/// shows instead, with projects' statuses after their titles, and with
/// drafts' links marked.
pub(crate) fn adapt(markdown: &str) -> String {
    draft_prefixes(&status_after_title(&without_title(markdown)))
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

/// `markdown` without the title its layout puts at the top, or the line
/// under it. Everything else the layout adds, like a project's references,
/// stays.
fn without_title(markdown: &str) -> String {
    // After the front matter, if there is any.
    let start = markdown
        .strip_prefix("---\n")
        .and_then(|rest| rest.find("\n---\n"))
        .map_or(0, |end| "---\n".len() + end + "\n---\n".len());
    let (front, body) = markdown.split_at(start);
    let Some(body) = [("<h1", "</h1>"), ("<h2", "</h2>")]
        .into_iter()
        .find_map(|(open, close)| skip_element(body.trim_start(), open, close))
    else {
        return markdown.to_owned();
    };
    let body = skip_element(body.trim_start(), "<p class=\"meta\"", "</p>").unwrap_or(body);
    format!("{front}{body}")
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
            && trimmed.ends_with("</span>")
            && let Some(start) = trimmed.find('>')
        {
            status = Some(trimmed[start + 1..trimmed.len() - 7].to_owned());
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
    fn status_after_title() {
        let markdown = "<li class=\"project\">\n    <span class=\"status complete\">complete</span>\n    <a class=\"name\" href=\"/projects/epic-ui\">Epic UI</a>\n    <p>Text</p>\n</li>\n";
        assert_eq!(
            super::status_after_title(markdown),
            "<li class=\"project\">\n    <a class=\"name\" href=\"/projects/epic-ui\">Epic UI</a> <small>complete</small>\n    <p>Text</p>\n</li>\n"
        );
    }

    #[test]
    fn draft_prefixes() {
        let markdown = "<ul>\n<li class=\"draft\">\n    <a href=\"/a\">A</a>\n</li>\n<li>\n    <a href=\"/b\">B</a>\n</li>\n<li class=\"project draft\">\n    <a class=\"name\" href=\"/c\">C</a>\n</li>\n</ul>";
        assert_eq!(
            super::draft_prefixes(markdown),
            "<ul>\n<li class=\"draft\">\n    <a href=\"/a\">Draft: A</a>\n</li>\n<li>\n    <a href=\"/b\">B</a>\n</li>\n<li class=\"project draft\">\n    <a class=\"name\" href=\"/c\">Draft: C</a>\n</li>\n</ul>"
        );
        assert_eq!(
            super::draft_prefixes("<link> <a href=x>y</a>"),
            "<link> <a href=x>y</a>"
        );
    }

    #[test]
    fn drops_the_layouts_title() {
        let markdown = "---\ntitle: Hi\n---\n\n<h2 class=\"title\">\n    Hi\n</h2>\n\n<ul class=\"references\"></ul>\n\nText";
        assert_eq!(
            without_title(markdown),
            "---\ntitle: Hi\n---\n\n\n<ul class=\"references\"></ul>\n\nText"
        );
        let markdown = "---\ntitle: Hi\n---\n\n<h2 class=\"title\">Hi</h2>\n<p class=\"meta\">\n    7 words\n</p>\n\nText";
        assert_eq!(without_title(markdown), "---\ntitle: Hi\n---\n\n\nText");
        assert_eq!(without_title("# Not a layout's\n"), "# Not a layout's\n");
    }

    #[test]
    fn filters_tagged_items() {
        let markdown = "<ul><li data-tags=\"rust\">a</li><li data-tags=\"unix rust\">b</li><li data-tags=\"unix\">c</li></ul>";
        assert_eq!(
            filter_tagged(markdown, "unix").as_deref(),
            Some("<ul><li data-tags=\"unix rust\">b</li><li data-tags=\"unix\">c</li></ul>")
        );
        assert_eq!(filter_tagged(markdown, "all").as_deref(), Some(markdown));
        assert_eq!(filter_tagged(markdown, "intro"), None);
    }
}
