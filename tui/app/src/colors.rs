//! The site's dark color scheme, from `css/style.css`, for the terminal.

use ratatui::style::{Color, Modifier, Style};
use ratmark::Theme;

/// `--background`
pub const BACKGROUND: Color = Color::Rgb(0x16, 0x16, 0x16);
/// `--text`
pub const TEXT: Color = Color::Rgb(0xca, 0xca, 0xca);
/// `--muted`
pub const MUTED: Color = Color::Rgb(0x99, 0x99, 0x99);
/// `--border`
pub const BORDER: Color = Color::Rgb(0x77, 0x77, 0x77);
/// `--faint`
pub const FAINT: Color = Color::Rgb(0x66, 0x66, 0x66);
/// `--light`, behind code blocks.
pub const LIGHT: Color = Color::Rgb(0x28, 0x28, 0x28);
/// Between the background and `--light`, behind inline code. The HTML
/// marks it with a font, which a terminal can't.
pub const SHADE: Color = Color::Rgb(0x24, 0x24, 0x24);
/// `--highlight`, behind highlighted code.
pub const HIGHLIGHT: Color = Color::Rgb(0x35, 0x2d, 0x17);
/// `--highlight-red`
pub const HIGHLIGHT_RED: Color = Color::Rgb(0x3b, 0x1f, 0x1d);
/// `--highlight-blue`
pub const HIGHLIGHT_BLUE: Color = Color::Rgb(0x1c, 0x28, 0x40);
/// `--highlight-edge`
pub const YELLOW: Color = Color::Rgb(0xf0, 0xc9, 0x4a);
/// `--highlight-blue-edge`
pub const BLUE: Color = Color::Rgb(0x5a, 0x8e, 0xe0);
/// `--highlight-purple-edge`
pub const PURPLE: Color = Color::Rgb(0x9a, 0x6a, 0xd6);

/// ratmark's theme in the site's colors. Like the HTML, it's mostly text
/// color, with weight and underlines doing the work.
pub fn theme() -> Theme {
    let bold = Style::new().fg(TEXT).add_modifier(Modifier::BOLD);
    Theme {
        text: Style::new().fg(TEXT),
        headings: [
            bold.add_modifier(Modifier::UNDERLINED),
            bold,
            bold,
            bold,
            Style::new().fg(TEXT),
            Style::new().fg(TEXT),
        ],
        link: Style::new().fg(BLUE).add_modifier(Modifier::UNDERLINED),
        image: Style::new().fg(PURPLE),
        // No color of its own, so code in a link stays the link's color.
        code: Style::new().bg(SHADE),
        code_block: Style::new().fg(TEXT).bg(LIGHT),
        quote: Style::new().fg(MUTED),
        quote_marker: Style::new().fg(FAINT),
        list_marker: Style::new().fg(MUTED),
        rule: Style::new().fg(FAINT),
        math: Style::new().fg(TEXT),
        footnote: Style::new().fg(MUTED),
        table_border: Style::new().fg(FAINT),
        table_header: bold,
        small: Style::new().fg(MUTED),
        // The backgrounds of highlighted code, as in the CSS, without the
        // edges.
        classes: vec![
            (".highlight-block".into(), Style::new().bg(HIGHLIGHT)),
            (
                ".highlight-block.red".into(),
                Style::new().bg(HIGHLIGHT_RED),
            ),
            (
                ".highlight-block.other".into(),
                Style::new().bg(HIGHLIGHT_BLUE),
            ),
            (".new-syntax".into(), Style::new().bg(HIGHLIGHT)),
        ],
        ..Theme::gruvbox()
    }
}
