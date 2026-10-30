//! Each page's Markdown, published next to its HTML for anyone who'd rather
//! read it, like `/posts/foo.md`.
//!
//! It's the Markdown after its template tags are rendered, wrapped in the
//! page's layout, but not the document around it. Its front matter is
//! what the site's [`Theme::front_matter`](crate::Theme::front_matter)
//! says, by default the title, date, description and whether it's a draft.

use std::fmt::{self, Write as _};

/// Front matter to publish, as YAML, with its fields in the order they're
/// added. Fields without a value are left out.
#[derive(Debug, Default)]
pub struct Yaml {
    text: String,
}

impl Yaml {
    pub fn new() -> Self {
        Self::default()
    }

    /// A string field, quoted where plain would read as something else.
    pub fn string(mut self, key: &str, value: Option<&str>) -> Self {
        if let Some(value) = value {
            writeln!(self.text, "{key}: {}", Self::quote(value)).unwrap();
        }
        self
    }

    /// A field that's only written when true.
    pub fn flag(mut self, key: &str, value: bool) -> Self {
        if value {
            writeln!(self.text, "{key}: true").unwrap();
        }
        self
    }

    /// A number field.
    pub fn number(mut self, key: &str, value: Option<usize>) -> Self {
        if let Some(value) = value {
            writeln!(self.text, "{key}: {value}").unwrap();
        }
        self
    }

    /// A list of strings, left out when empty.
    pub fn list(mut self, key: &str, values: &[String]) -> Self {
        if !values.is_empty() {
            writeln!(self.text, "{key}:").unwrap();
            for value in values {
                writeln!(self.text, "- {}", Self::quote(value)).unwrap();
            }
        }
        self
    }

    /// The Markdown `body` with this front matter on top, or as it is
    /// without any.
    pub fn with_body(&self, body: &str) -> String {
        if self.text.is_empty() {
            body.to_owned()
        } else {
            format!("---\n{}---\n\n{body}", self.text)
        }
    }

    /// The string plain, or in single quotes where plain would read as
    /// something else, the way Ruby's Psych writes it.
    fn quote(s: &str) -> String {
        let looks_like_other = s.is_empty()
            || s.parse::<f64>().is_ok()
            || ["true", "false", "yes", "no", "on", "off", "null", "~"]
                .contains(&s.to_ascii_lowercase().as_str())
            || (s.len() >= 10
                && s.as_bytes()[4] == b'-'
                && s[..4].bytes().all(|b| b.is_ascii_digit()));
        let special = s.contains(": ")
            || s.contains(" #")
            || s.ends_with(':')
            || s.starts_with(char::is_whitespace)
            || s.ends_with(char::is_whitespace)
            || s.starts_with([
                '-', '?', ':', ',', '[', ']', '{', '}', '#', '&', '*', '!', '|', '>', '\'', '"',
                '%', '@', '`',
            ]);
        if looks_like_other || special {
            format!("'{}'", s.replace('\'', "''"))
        } else {
            s.to_owned()
        }
    }
}

impl fmt::Display for Yaml {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_matter() {
        let yaml = Yaml::new()
            .string("title", Some("Yes: no"))
            .string("date", Some("2026-10-19"))
            .flag("draft", false)
            .list("references", &["a".into()])
            .number("words", Some(3));
        assert_eq!(
            yaml.with_body("Hi\n"),
            "---\ntitle: 'Yes: no'\ndate: '2026-10-19'\nreferences:\n- a\nwords: 3\n---\n\nHi\n"
        );
        assert_eq!(Yaml::new().with_body("Hi\n"), "Hi\n");
    }
}
