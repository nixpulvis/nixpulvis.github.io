//! The site's own markup in its published Markdown, reworked for a terminal.
//! The site's layouts and CSS do things a terminal can't, like floating a
//! project's status beside its title, so these say the same in text.

/// `body`, a page without its front matter, as the terminal shows it:
/// without the title the box's border shows instead, and with projects'
/// statuses after their titles.
pub(crate) fn adapt(body: &str) -> String {
    status_after_title(without_title(body))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_after_title() {
        let markdown = r#"<li class="project">
    <span class="status complete">complete</span>
    <a class="name" href="/projects/epic-ui">Epic UI</a>
    <p>Text</p>
</li>
"#;
        assert_eq!(
            super::status_after_title(markdown),
            r#"<li class="project">
    <a class="name" href="/projects/epic-ui">Epic UI</a> <small>complete</small>
    <p>Text</p>
</li>
"#
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
            r#"

<ul class="references"></ul>

Text"#
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
        let markdown = "<ul><li data-tags=\"rust\">a</li><li data-tags=\"unix rust\">b</li><li data-tags=\"unix\">c</li></ul>";
        assert_eq!(
            filter_tagged(markdown, "unix").as_deref(),
            Some("<ul><li data-tags=\"unix rust\">b</li><li data-tags=\"unix\">c</li></ul>")
        );
        assert_eq!(filter_tagged(markdown, "all").as_deref(), Some(markdown));
        assert_eq!(filter_tagged(markdown, "intro"), None);
    }
}
