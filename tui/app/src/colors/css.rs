//! Colors read from a stylesheet's custom properties, while compiling, and
//! mixed in oklch the way CSS mixes them.
//!
//! A property can be a hex color, `var(--name)`, `light-dark(light, dark)`,
//! or `color-mix(in oklch, a p%, b)` of the text and background, nested in
//! any way. `var(--foreground)` and `var(--background)` are the text and the
//! background themselves, so a mix of them reads as a share of the text,
//! which the view mixes from whatever it's drawn on, with [`mix`].
//! Anything else, or a property that isn't declared, fails the build.
//!
//! Every value is a slice of the stylesheet, cut down as it's read, with
//! slice patterns where they fit, since only `const fn`s run while
//! compiling.

/// Which side of `light-dark(light, dark)` to read.
#[derive(Debug, Clone, Copy)]
pub(super) enum Scheme {
    Light,
    Dark,
}

/// A color as a stylesheet gives it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Color {
    /// A color of its own.
    Rgb([u8; 3]),
    /// A share of the text over the background, mixed in oklch: 0 is the
    /// background and 1 the text.
    Mix(f32),
}

/// A stylesheet, whose custom properties are read as colors.
#[derive(Debug, Clone, Copy)]
pub(super) struct Stylesheet<'a>(&'a [u8]);

impl<'a> Stylesheet<'a> {
    pub(super) const fn new(css: &'a [u8]) -> Self {
        Self(css)
    }

    /// The color it gives `--name` in `scheme`.
    pub(super) const fn color(&self, name: &[u8], scheme: Scheme) -> Color {
        self.parse_color(self.value(name), scheme)
    }

    /// The color it gives `--name` in `scheme`, which must be one of its own.
    pub(super) const fn rgb(&self, name: &[u8], scheme: Scheme) -> [u8; 3] {
        match self.color(name, scheme) {
            Color::Rgb(rgb) => rgb,
            Color::Mix(_) => panic!("a color is mixed from the text, not one of its own"),
        }
    }

    /// The share of the text in the `color-mix()` it gives `--name` in
    /// `scheme`, which must be mixed from the text and the background.
    pub(super) const fn color_mix(&self, name: &[u8], scheme: Scheme) -> f32 {
        match self.color(name, scheme) {
            Color::Mix(mix) => mix,
            Color::Rgb(_) => panic!("a gray is a color of its own, not mixed from the text"),
        }
    }

    /// The percentage it gives `--name`, like `18%`, as a fraction.
    pub(super) const fn percentage(&self, name: &[u8]) -> f32 {
        self.parse_percentage(self.value(name))
    }

    /// `--name`'s value, between its `:` and its `;` or `}`. It's only found
    /// at the start of a declaration, so `--foreground` isn't found in
    /// `--ratzilla-foreground`, and not in comments.
    const fn value(&self, name: &[u8]) -> &'a [u8] {
        let mut rest = self.0;
        let mut can_declare = true;
        while let [c, tail @ ..] = rest {
            if let Some(comment) = strip_prefix(rest, b"/*") {
                rest = after(comment, b"*/");
                can_declare = true;
                continue;
            }
            if can_declare
                && let Some(declared) = strip_prefix(rest, b"--")
                && let Some(declared) = strip_prefix(declared, name)
                && let [b':', value @ ..] = declared
            {
                return until(until(value, b';'), b'}').trim_ascii();
            }
            can_declare = c.is_ascii_whitespace() || matches!(c, b'{' | b';');
            rest = tail;
        }
        panic!("a color is missing from the stylesheet");
    }

    /// The color the CSS value `expr` is in `scheme`, looking up the
    /// properties its `var()`s name.
    const fn parse_color(&self, expr: &[u8], scheme: Scheme) -> Color {
        let expr = expr.trim_ascii();
        if let [b'#', digits @ ..] = expr {
            return Color::Rgb(hex(digits));
        }
        if let Some(args) = call(expr, b"var") {
            let name = var_name(args);
            if equals(name, b"foreground") {
                return Color::Mix(1.0);
            }
            if equals(name, b"background") {
                return Color::Mix(0.0);
            }
            return self.parse_color(self.value(name), scheme);
        }
        if let Some(args) = call(expr, b"light-dark") {
            let Some((light, dark)) = split(args, b',') else {
                panic!("a light-dark() doesn't have two colors");
            };
            return match scheme {
                Scheme::Light => self.parse_color(light, scheme),
                Scheme::Dark => self.parse_color(dark, scheme),
            };
        }
        if let Some(args) = call(expr, b"color-mix") {
            let Some((space, args)) = split(args, b',') else {
                panic!("a color-mix() doesn't have its colors");
            };
            assert!(
                equals(space.trim_ascii(), b"in oklch"),
                "a color-mix() isn't in oklch"
            );
            let Some((a, b)) = split(args, b',') else {
                panic!("a color-mix() doesn't have two colors");
            };
            let (a, a_amount) = self.operand(a, scheme);
            let (b, b_amount) = self.operand(b, scheme);
            // As in CSS, a missing percentage is what the other leaves, and
            // two are scaled to add up to 100%.
            let amount = match (a_amount, b_amount) {
                (Some(a), Some(b)) => a / (a + b),
                (Some(a), None) => a,
                (None, Some(b)) => 1.0 - b,
                (None, None) => 0.5,
            };
            return match (a, b) {
                (Color::Mix(a), Color::Mix(b)) => Color::Mix(b + (a - b) * amount),
                _ => panic!("a color-mix() isn't of the text and the background"),
            };
        }
        panic!("a color isn't hex, var(), light-dark() or color-mix()");
    }

    /// A `color-mix()` color, and its percentage after it, if it has one.
    const fn operand(&self, expr: &[u8], scheme: Scheme) -> (Color, Option<f32>) {
        match split(expr.trim_ascii(), b' ') {
            Some((color, percent)) => (
                self.parse_color(color, scheme),
                Some(self.parse_percentage(percent)),
            ),
            None => (self.parse_color(expr, scheme), None),
        }
    }

    /// The percentage the CSS value `expr` is, `p%` or a `var()` of one, as
    /// a fraction.
    const fn parse_percentage(&self, expr: &[u8]) -> f32 {
        let expr = expr.trim_ascii();
        if let Some(args) = call(expr, b"var") {
            return self.parse_percentage(self.value(var_name(args)));
        }
        let [digits @ .., b'%'] = expr else {
            panic!("a percentage doesn't end in %");
        };
        // The digits as a whole number, then divided once, so `12%` reads
        // exactly as `0.12` would be written.
        let mut number: u32 = 0;
        let mut divisor: f32 = 100.0;
        let mut point = false;
        let mut rest = digits;
        while let [c, tail @ ..] = rest {
            match *c {
                b'.' if !point => point = true,
                b'0'..=b'9' => {
                    number = number * 10 + (*c - b'0') as u32;
                    if point {
                        divisor *= 10.0;
                    }
                }
                _ => panic!("a percentage isn't a plain number"),
            }
            rest = tail;
        }
        number as f32 / divisor
    }
}

// Reading syntax.

/// The arguments of `name(…)`, if that's all `expr` is.
const fn call<'a>(expr: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    let Some([b'(', args @ .., b')']) = strip_prefix(expr, name) else {
        return None;
    };
    // `name(a) (b)` starts and ends with parentheses too, but its first one
    // closes early.
    let mut depth = 0;
    let mut rest = args;
    while let [c, tail @ ..] = rest {
        match *c {
            b'(' => depth += 1,
            b')' if depth == 0 => return None,
            b')' => depth -= 1,
            _ => {}
        }
        rest = tail;
    }
    Some(args)
}

/// The name in `var(--name)` or `var(--name, fallback)`, from its
/// arguments, without its dashes.
const fn var_name(args: &[u8]) -> &[u8] {
    let [b'-', b'-', name @ ..] = args.trim_ascii() else {
        panic!("a var() doesn't name a custom property");
    };
    match split(name, b',') {
        Some((name, _fallback)) => name.trim_ascii(),
        None => name,
    }
}

/// `list` split around its first `separator` outside parentheses, where
/// `b' '` stands for any whitespace.
const fn split(list: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let mut depth = 0;
    let mut at = 0;
    while at < list.len() {
        let c = list[at];
        if depth == 0 && (c == separator || separator == b' ' && c.is_ascii_whitespace()) {
            let (before, after) = list.split_at(at);
            return Some((before, after.split_at(1).1));
        }
        match c {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        at += 1;
    }
    None
}

/// A hex color's digits, six or three.
const fn hex(digits: &[u8]) -> [u8; 3] {
    const fn digit(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("a hex color has a digit that isn't hex"),
        }
    }
    match *digits {
        [r1, r2, g1, g2, b1, b2] => [
            digit(r1) << 4 | digit(r2),
            digit(g1) << 4 | digit(g2),
            digit(b1) << 4 | digit(b2),
        ],
        [r, g, b] => [digit(r) * 0x11, digit(g) * 0x11, digit(b) * 0x11],
        _ => panic!("a hex color isn't six or three digits"),
    }
}

// Bytes, since the standard library's slice methods for these aren't
// `const`.

/// `bytes` up to the first `end`, or all of it.
const fn until(bytes: &[u8], end: u8) -> &[u8] {
    let mut at = 0;
    while at < bytes.len() && bytes[at] != end {
        at += 1;
    }
    bytes.split_at(at).0
}

/// `bytes` after the first `end`, or nothing.
const fn after<'a>(bytes: &'a [u8], end: &[u8]) -> &'a [u8] {
    let mut rest = bytes;
    while let [_, tail @ ..] = rest {
        if let Some(after) = strip_prefix(rest, end) {
            return after;
        }
        rest = tail;
    }
    rest
}

const fn strip_prefix<'a>(bytes: &'a [u8], prefix: &[u8]) -> Option<&'a [u8]> {
    if bytes.len() < prefix.len() {
        return None;
    }
    let (head, rest) = bytes.split_at(prefix.len());
    if equals(head, prefix) {
        Some(rest)
    } else {
        None
    }
}

const fn equals(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

// Mixing.

// TODO: Make this, `oklch` and `from_oklab` const once `powf`, `cbrt`, the
// trig functions and `rem_euclid` are, so `parse_color` can mix fixed colors
// too, and read the highlights, like `--highlight-red`, while compiling.
/// `amount` of `color` over `background`, mixed in oklch like CSS's
/// `color-mix(in oklch, ...)`, the shorter way around the hue.
pub(super) fn mix(color: [u8; 3], background: [u8; 3], amount: f32) -> [u8; 3] {
    let (l1, c1, h1) = oklch(color);
    let (l2, c2, h2) = oklch(background);
    // A gray has no hue, so it takes the other color's.
    let (h1, h2) = match (h1, h2) {
        (Some(h1), Some(h2)) => (h1, h2),
        (Some(h), None) | (None, Some(h)) => (h, h),
        (None, None) => (0.0, 0.0),
    };
    let turn = (h1 - h2 + 180.0).rem_euclid(360.0) - 180.0;
    let l = l2 + (l1 - l2) * amount;
    let c = c2 + (c1 - c2) * amount;
    let h = (h2 + turn * amount).to_radians();
    from_oklab(l, c * h.cos(), c * h.sin())
}

/// sRGB as oklch: lightness, chroma, and the hue in degrees, which a gray
/// doesn't have.
pub(super) fn oklch([r, g, b]: [u8; 3]) -> (f32, f32, Option<f32>) {
    let [r, g, b] = [r, g, b].map(|c| {
        let c = f32::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    let lightness = 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s;
    let a = 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s;
    let b = 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s;
    let chroma = a.hypot(b);
    let hue = (chroma > 1e-4).then(|| b.atan2(a).to_degrees());
    (lightness, chroma, hue)
}

/// oklab as sRGB, clipped to it.
fn from_oklab(lightness: f32, a: f32, b: f32) -> [u8; 3] {
    let l = (lightness + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
    let m = (lightness - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let s = (lightness - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
    [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ]
    .map(|c| {
        let c = if c <= 0.003_130_8 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (c.clamp(0.0, 1.0) * 255.0).round() as u8
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSS: Stylesheet = Stylesheet::new(
        b"
        /* --text: #000000; isn't read from a comment. */
        :root {
          --foreground: light-dark(#262626, #cacaca);
          --background: light-dark(#fafafa, #161616);
          --color-1: #fe340a;
          --color-10: #54e53e;
          --ratzilla-color-1: #000;
          --red: var(--color-1);
          --white: #fff;
          --text: var(--foreground);
          --mix: 18%;
          --light: light-dark(
            color-mix(in oklch, var(--text) 5%, var(--background)),
            color-mix(in oklch, var(--text) 19.5%, var(--background))
          );
          --border: light-dark(var(--text), color-mix(in oklch, var(--text) var(--mix), var(--background)));
          --half: color-mix(in oklch, var(--background), var(--text));
          --nested: color-mix(in oklch, var(--light) 50%, var(--background));
        }
    ",
    );

    #[test]
    fn reads_colors_by_their_whole_name() {
        assert_eq!(CSS.rgb(b"color-1", Scheme::Dark), [0xfe, 0x34, 0x0a]);
        assert_eq!(CSS.rgb(b"color-10", Scheme::Dark), [0x54, 0xe5, 0x3e]);
        assert_eq!(CSS.rgb(b"red", Scheme::Light), [0xfe, 0x34, 0x0a]);
        assert_eq!(CSS.rgb(b"white", Scheme::Dark), [0xff; 3]);
    }

    #[test]
    fn reads_each_side_of_light_dark() {
        assert_eq!(CSS.rgb(b"foreground", Scheme::Light), [0x26; 3]);
        assert_eq!(CSS.rgb(b"foreground", Scheme::Dark), [0xca; 3]);
        assert_eq!(CSS.color_mix(b"border", Scheme::Light), 1.0);
        assert_eq!(CSS.color_mix(b"border", Scheme::Dark), 0.18);
    }

    #[test]
    fn reads_mixes_as_shares_of_the_text() {
        assert_eq!(CSS.color_mix(b"light", Scheme::Light), 0.05);
        assert_eq!(CSS.color_mix(b"light", Scheme::Dark), 0.195);
        // Without percentages it's half and half.
        assert_eq!(CSS.color_mix(b"half", Scheme::Dark), 0.5);
        // A mix of a mix.
        assert_eq!(CSS.color_mix(b"nested", Scheme::Light), 0.025);
    }

    #[test]
    fn reads_percentages() {
        assert_eq!(CSS.percentage(b"mix"), 0.18);
    }

    #[test]
    fn highlights_mix_like_the_css() {
        // `color-mix(in oklch, #ff8000 18%, #161616)`, as browsers draw it.
        assert_eq!(mix([0xff, 0x80, 0x00], [0x16; 3], 0.18), [0x3a, 0x28, 0x1d]);
    }
}
