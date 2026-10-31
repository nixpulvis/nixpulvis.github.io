//! Code blocks, highlighted with syntect's classes, which `css/syntax.css`
//! colors.

use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

use crate::text::Text;

/// Highlights code blocks.
pub struct Highlighter {
    syntaxes: SyntaxSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            syntaxes: SyntaxSet::load_defaults_newlines(),
        }
    }
}

impl Highlighter {
    /// Languages left uncolored, but shown like highlighted code.
    const PLAIN: &[&str] = &["plaintext", "text", "conf", "elixir"];

    /// The HTML for a code block in `lang`, which may be empty.
    ///
    /// Languages syntect knows, and plain text, are highlighted inside
    /// `<div class="language-rust"><div class="highlight">`, like Jekyll's
    /// rouge did. Code in any other language is left in a plain `<pre>`.
    pub fn block(&self, lang: &str, code: &str) -> String {
        let lang = if lang.is_empty() { "plaintext" } else { lang };
        let syntax = match self.syntax(lang) {
            Some(syntax) => syntax,
            None if Self::PLAIN.contains(&lang) => self.syntaxes.find_syntax_plain_text(),
            None => {
                return format!(
                    "<pre><code class=\"language-{lang}\">{}</code></pre>\n",
                    Text::escape(code)
                );
            }
        };
        let mut html =
            ClassedHTMLGenerator::new_with_class_style(syntax, &self.syntaxes, ClassStyle::Spaced);
        for line in LinesWithEndings::from(code) {
            // Only fails on a broken syntax definition.
            let _ = html.parse_html_for_line_which_includes_newline(line);
        }
        format!(
            "<div class=\"language-{lang}\"><div class=\"highlight\"><pre class=\"highlight\"><code>{}</code></pre></div></div>\n",
            html.finalize()
        )
    }

    fn syntax(&self, lang: &str) -> Option<&SyntaxReference> {
        let token = match lang {
            "plaintext" | "text" => return None,
            // Close enough, and better than nothing.
            "racket" | "scheme" => "lisp",
            "rb" => "ruby",
            lang => lang,
        };
        self.syntaxes.find_syntax_by_token(token)
    }
}
