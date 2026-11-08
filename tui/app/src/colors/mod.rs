//! The view's colors, which the terminal and browser crates each pick to suit
//! where they draw.

use ratatui::style::{Color, Modifier, Style};
use ratmark::Theme;

use css::{Scheme, Stylesheet, mix, oklch};

mod css;

/// Palette entries the view draws in, by their ANSI names.
pub(crate) const RED: u8 = 1;
pub(crate) const GREEN: u8 = 2;
pub(crate) const YELLOW: u8 = 3;
pub(crate) const BLUE: u8 = 4;
pub(crate) const MAGENTA: u8 = 5;
pub(crate) const BRIGHT_BLACK: u8 = 8;

/// What the view is drawn in: a terminal's text, background and 16-color
/// palette, as RGB. `tui/term` and `tui/web` each build one with
/// [`Colors::from_terminal`], from the colors of the terminal they draw in:
/// a real terminal's, or the theme the page's CSS gives the browser view.
/// When a terminal doesn't say what its colors are, it's [`Colors::DARK`].
/// Whatever else the view draws in is mixed from these where it's used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colors {
    foreground: [u8; 3],
    background: [u8; 3],
    /// Palette colors 0 through 15.
    palette: [[u8; 3]; 16],
    /// Whether the terminal reported these as its own, so they're drawn as
    /// its text, background and palette entries, which it draws exactly,
    /// even a background it shows through. Otherwise they're drawn as RGB.
    reported: bool,
}

/// A gray between the text and the background, by how much of the text
/// shows in it over a dark background, and over a light one, mixed in oklch.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Gray {
    dark: f32,
    light: f32,
}

impl Gray {
    /// The gray the stylesheet gives `--name`, as shares of its text over
    /// its background in each scheme.
    const fn css(name: &[u8]) -> Self {
        Self {
            dark: STYLE.color_mix(name, Scheme::Dark),
            light: STYLE.color_mix(name, Scheme::Light),
        }
    }
}

/// `--light`, behind code blocks.
const LIGHT: Gray = Gray::css(b"light");

/// Between the background and `--light`, behind inline code: `#242424`, or
/// `#f6f6f6`. The HTML marks it with a font, which a terminal can't, so the
/// stylesheet has no color for it.
const CODE: Gray = Gray {
    dark: 0.095,
    light: 0.017,
};

/// `--border`.
pub(crate) const BORDER: Gray = Gray::css(b"border");

/// `--faint`.
pub(crate) const FAINT: Gray = Gray::css(b"faint");

/// How much of a hue shows in a highlight, `--highlight-mix`.
const HIGHLIGHT_MIX: f32 = STYLE.percentage(b"highlight-mix");

/// The site's stylesheet, whose `:root` defines its colors.
const STYLE: Stylesheet = Stylesheet::new(include_bytes!("../../../../css/style.css"));

/// The dark color the stylesheet gives `--name`.
const fn dark(name: &[u8]) -> [u8; 3] {
    STYLE.rgb(name, Scheme::Dark)
}

// NOTE: A constant is only evaluated where it's used, and outside tests
// nothing in this crate uses `Colors::DARK`: only `tui/term` and `tui/web`
// do. Without this, a palette color missing from the stylesheet passes
// `cargo check` and `cargo build` here, and fails only when one of them is
// built, as an error in that crate. The grays are used in this crate
// already, but are listed so every color read from the stylesheet is
// checked in one place.
const _: (Colors, [Gray; 3], f32) = (Colors::DARK, [LIGHT, BORDER, FAINT], HIGHLIGHT_MIX);

impl Colors {
    // TODO: Terminals without true color, down to the basic 16. `DARK` draws
    // everything as 24-bit RGB, and even a terminal's own colors have their
    // grays and tints mixed as RGB. The view is built on the 16-color
    // palette, so such a terminal could be sent only `Reset` and
    // `Indexed(0..=15)`, with each gray and tint taken as the nearest of
    // those.
    /// The site's dark colors, fixed: the dark half of the colors
    /// `css/style.css` defines, read from it while compiling.
    pub const DARK: Self = Self {
        foreground: dark(b"foreground"),
        background: dark(b"background"),
        palette: [
            dark(b"color-0"),
            dark(b"color-1"),
            dark(b"color-2"),
            dark(b"color-3"),
            dark(b"color-4"),
            dark(b"color-5"),
            dark(b"color-6"),
            dark(b"color-7"),
            dark(b"color-8"),
            dark(b"color-9"),
            dark(b"color-10"),
            dark(b"color-11"),
            dark(b"color-12"),
            dark(b"color-13"),
            dark(b"color-14"),
            dark(b"color-15"),
        ],
        reported: false,
    };

    /// The terminal's own colors, as it says they are: its text, its
    /// background, and palette colors 0 through 15.
    pub fn from_terminal(foreground: [u8; 3], background: [u8; 3], palette: [[u8; 3]; 16]) -> Self {
        Self {
            foreground,
            background,
            palette,
            reported: true,
        }
    }

    /// The text color.
    pub(crate) fn text(&self) -> Color {
        if self.reported {
            Color::Reset
        } else {
            Color::from(self.foreground)
        }
    }

    /// The background color.
    pub(crate) fn background(&self) -> Color {
        if self.reported {
            Color::Reset
        } else {
            Color::from(self.background)
        }
    }

    /// Palette entry `index`.
    pub(crate) fn ansi(&self, index: u8) -> Color {
        if self.reported {
            Color::Indexed(index)
        } else {
            Color::from(self.palette[usize::from(index)])
        }
    }

    /// `gray`, mixed from the text over the background in oklch like the
    /// site's CSS. It takes the light mix on a background lighter than the
    /// text, and the dark one otherwise.
    pub(crate) fn gray(&self, gray: Gray) -> Color {
        let Self {
            foreground,
            background,
            ..
        } = *self;
        let mix = if oklch(background).0 > oklch(foreground).0 {
            gray.light
        } else {
            gray.dark
        };
        Color::from(css::mix(foreground, background, mix))
    }

    /// ratmark's theme in these colors. Like the HTML, it's mostly text
    /// color, with weight and underlines doing the work. Code is highlighted
    /// with ratmark's `ansi` theme, in palette colors, drawn as RGB like
    /// everything else when these aren't the terminal's own.
    pub fn theme(&self) -> Theme {
        let text = Style::new().fg(self.text());
        let bold = text.add_modifier(Modifier::BOLD);
        let muted = Style::new().fg(self.ansi(BRIGHT_BLACK));
        let faint = Style::new().fg(self.gray(FAINT));
        // The backgrounds of highlighted code, a little of a hue over the
        // background as in the CSS, without the edges.
        let highlight = |index: u8| {
            let hue = self.palette[usize::from(index)];
            Style::new().bg(Color::from(mix(hue, self.background, HIGHLIGHT_MIX)))
        };
        Theme {
            text,
            headings: [
                bold.add_modifier(Modifier::UNDERLINED),
                bold,
                bold,
                bold,
                text,
                text,
            ],
            link: Style::new()
                .fg(self.ansi(BLUE))
                .add_modifier(Modifier::UNDERLINED),
            image: Style::new().fg(self.ansi(MAGENTA)),
            // No color of its own, so code in a link stays the link's color.
            code: Style::new().bg(self.gray(CODE)),
            code_block: text.bg(self.gray(LIGHT)),
            code_block_padding: false,
            quote: muted,
            quote_marker: faint,
            list_marker: muted,
            rule: faint,
            math: text,
            footnote: muted,
            table_border: faint,
            table_header: bold,
            small: muted,
            syntax_palette: (!self.reported).then(|| self.palette.map(Color::from)),
            classes: vec![
                (".highlight-block.green".into(), highlight(GREEN)),
                (".highlight-block.red".into(), highlight(RED)),
                (".highlight-block.blue".into(), highlight(BLUE)),
                (".new-syntax".into(), highlight(GREEN)),
            ],
            ..Theme::ansi_dark()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Colors with only a text color and a background.
    fn on(foreground: u8, background: u8) -> Colors {
        Colors {
            foreground: [foreground; 3],
            background: [background; 3],
            palette: [[0; 3]; 16],
            reported: false,
        }
    }

    #[test]
    fn grays_mix_like_the_css_by_the_background() {
        // `color-mix(in oklch, #cacaca 12%, #161616)` and so on, as browsers
        // draw them, with the dark mix on a dark background and the light
        // one on a light background.
        let grays = [
            Gray {
                dark: 0.12,
                light: 0.05,
            },
            Gray {
                dark: 0.485,
                light: 0.345,
            },
        ];
        assert_eq!(
            grays.map(|gray| on(0xca, 0x16).gray(gray)),
            [0x28, 0x66].map(|c| Color::Rgb(c, c, c)),
        );
        assert_eq!(
            grays.map(|gray| on(0x26, 0xfa).gray(gray)),
            [0xee, 0xaa].map(|c| Color::Rgb(c, c, c)),
        );
    }
}
