//! Small text transformations for the templates, named and behaving like
//! the Liquid filters the site's templates used to call.

use std::fmt::Write as _;

/// Text in a template.
pub struct Text;

impl Text {
    /// The text with HTML tags, comments, scripts and styles taken out.
    /// Entities are left alone.
    pub fn strip_html(html: &str) -> String {
        let mut out = String::with_capacity(html.len());
        let mut rest = html;
        while let Some(start) = rest.find('<') {
            out.push_str(&rest[..start]);
            rest = &rest[start..];
            let close = [
                ("<script", "</script>"),
                ("<style", "</style>"),
                ("<!--", "-->"),
            ]
            .iter()
            .find(|(open, _)| starts_with_ignore_case(rest, open))
            .map_or(">", |(_, close)| close);
            match find_ignore_case(rest, close) {
                Some(end) => rest = &rest[end + close.len()..],
                None => {
                    out.push_str(rest);
                    rest = "";
                }
            }
        }
        out.push_str(rest);
        out
    }

    /// How many words, separated by whitespace, are in `text`.
    pub fn number_of_words(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Runs of whitespace made single spaces.
    pub fn normalize_whitespace(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut space = false;
        for c in text.chars() {
            if c.is_whitespace() {
                space = true;
            } else {
                if space {
                    out.push(' ');
                    space = false;
                }
                out.push(c);
            }
        }
        if space {
            out.push(' ');
        }
        out
    }

    /// The text cut to `length` characters, ending with `...` where it's
    /// cut.
    pub fn truncate(text: &str, length: usize) -> String {
        if text.chars().count() <= length {
            return text.to_owned();
        }
        let mut out: String = text.chars().take(length.saturating_sub(3)).collect();
        out.push_str("...");
        out
    }

    /// `&`, `<`, `>`, `"` and `'` escaped for HTML.
    pub fn escape(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for c in text.chars() {
            match c {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&#39;"),
                c => out.push(c),
            }
        }
        out
    }

    /// Like [`Text::escape`], but leaves entities that are already there.
    pub fn escape_once(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for (i, c) in text.char_indices() {
            if c == '&' && Self::entity_at(&text[i..]) {
                out.push('&');
            } else {
                out.push_str(&Self::escape(c.encode_utf8(&mut [0; 4])));
            }
        }
        out
    }

    /// The text encoded for a URL's query, with spaces as `+`.
    pub fn url_encode(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for b in text.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                    out.push(b as char)
                }
                b' ' => out.push('+'),
                b => write!(out, "%{b:02X}").unwrap(),
            }
        }
        out
    }

    /// Newlines made `<br />` tags followed by the newline.
    pub fn newline_to_br(text: &str) -> String {
        text.replace('\n', "<br />\n")
    }

    /// The text without any newlines.
    pub fn strip_newlines(text: &str) -> String {
        text.replace(['\n', '\r'], "")
    }

    /// Whether `text` starts with an HTML entity, like `&amp;` or `&#35;`.
    fn entity_at(text: &str) -> bool {
        let Some(end) = text.find(';') else {
            return false;
        };
        let name = &text[1..end];
        !name.is_empty()
            && match name.strip_prefix('#') {
                Some(n) => match n.strip_prefix(['x', 'X']) {
                    Some(hex) => !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit()),
                    None => !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()),
                },
                None => name.chars().all(|c| c.is_ascii_alphanumeric()),
            }
    }
}

fn starts_with_ignore_case(text: &str, prefix: &str) -> bool {
    text.get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}

fn find_ignore_case(text: &str, needle: &str) -> Option<usize> {
    let lower = text.to_ascii_lowercase();
    lower.find(&needle.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_html() {
        assert_eq!(
            Text::strip_html("<p>Hi <b>there</b><!-- x --><script>no</script></p>"),
            "Hi there"
        );
    }

    #[test]
    fn escapes_once() {
        assert_eq!(
            Text::escape_once("a &amp; b & <c>"),
            "a &amp; b &amp; &lt;c&gt;"
        );
        assert_eq!(Text::escape_once("&#8217;"), "&#8217;");
    }

    #[test]
    fn truncates() {
        assert_eq!(Text::truncate("abcdef", 6), "abcdef");
        assert_eq!(Text::truncate("abcdefg", 6), "abc...");
    }

    #[test]
    fn url_encodes() {
        assert_eq!(
            Text::url_encode("https://nixpulvis.com/a b"),
            "https%3A%2F%2Fnixpulvis.com%2Fa+b"
        );
    }
}
