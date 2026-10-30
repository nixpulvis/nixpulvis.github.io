//! TikZ figures, drawn to inline SVG at build time.
//!
//! A fenced code block in `tikz` is a figure, and the rest of its info
//! string is the figure's accessible label:
//!
//! ````markdown
//! ```tikz A two state automaton
//! \begin{tikzpicture}
//!   \node[state] (a) {$q_0$};
//!   \node[state, right=of a] (b) {$q_1$};
//!   \draw[->] (a) -- node[above] {$\sigma$} (b);
//! \end{tikzpicture}
//! ```
//! ````
//!
//! The code is placed verbatim in a LaTeX document, so `tikzcd` works as
//! well as `tikzpicture`, compiled with `latex`, and converted with
//! `dvisvgm`. Extra packages and libraries go in the
//! site's [`Config::tikz_preamble`](crate::Config::tikz_preamble).
//!
//! Figures are cached in `.site-cache/tikz`, by a hash of their document,
//! so a figure is only compiled again when it changes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::text::Text;

/// Draws TikZ figures, keeping a cache of them.
pub struct Tikz {
    cache: PathBuf,
    preamble: String,
}

impl Tikz {
    /// Bump to invalidate every cached figure after changing the output.
    const VERSION: u32 = 1;

    const HEADER: &str = r"\def\pgfsysdriver{pgfsys-dvisvgm.def}
\documentclass{article}
\usepackage{amsmath,amssymb}
\usepackage{tikz}
\pagestyle{empty}
\hoffset=-1in \voffset=-1in \oddsidemargin=0pt \topmargin=0pt
\headheight=0pt \headsep=0pt \topskip=0pt \parindent=0pt
\newsavebox\tikzfigure
\newdimen\tikzborder \tikzborder=1pt
";

    // Like the standalone class, which TeX Live's base packages lack: box
    // the figure at the page's top left corner and size the page to it, so
    // the SVG is cropped to the picture. TikZ leaves half of each line's
    // width out of its bounding box, so a 1pt border keeps the outer edges
    // of strokes from being clipped when the SVG is scaled down.
    const DOCUMENT: [&str; 2] = [
        "\\begin{document}\n\\begin{lrbox}{\\tikzfigure}%\n",
        "%\n\\end{lrbox}%\n\\sbox{\\tikzfigure}{%\n  \\kern\\tikzborder\n  \\vbox{\\kern\\tikzborder\\copy\\tikzfigure\\kern\\tikzborder}%\n  \\kern\\tikzborder}%\n\\special{papersize=\\the\\wd\\tikzfigure,\\the\\ht\\tikzfigure}%\n\\noindent\\usebox{\\tikzfigure}\n\\end{document}\n",
    ];

    /// Draws figures, caching them in `cache`, with `preamble` in every
    /// figure's document for the packages and libraries they use.
    pub fn new(cache: impl Into<PathBuf>, preamble: &str) -> Self {
        Self {
            cache: cache.into(),
            preamble: preamble.to_owned(),
        }
    }

    /// The figure for `code`, labeled `label`, as a `<figure>` holding its
    /// SVG, from the cache when it's there.
    pub fn figure(&self, code: &str, label: &str) -> Result<String, String> {
        let tex = format!(
            "{}{}\n{}{}{}",
            Self::HEADER,
            self.preamble,
            Self::DOCUMENT[0],
            code.trim(),
            Self::DOCUMENT[1]
        );
        let key = format!(
            "{:x}",
            Sha256::digest(format!("{}\0{tex}", Self::VERSION).as_bytes())
        );
        let cached = self.cache.join(format!("{key}.svg"));
        let svg = match fs::read_to_string(&cached) {
            Ok(svg) => svg,
            Err(_) => {
                let svg = Self::compile(&tex, &key[..8])?;
                fs::create_dir_all(&self.cache).map_err(|e| e.to_string())?;
                fs::write(&cached, &svg).map_err(|e| e.to_string())?;
                svg
            }
        };
        let svg = if label.is_empty() {
            svg
        } else {
            svg.replacen(
                "<svg ",
                &format!("<svg role=\"img\" aria-label=\"{}\" ", Text::escape(label)),
                1,
            )
        };
        // Kept on one line, like a single raw HTML block.
        Ok(format!("<figure class=\"tikz\">{svg}</figure>\n"))
    }

    fn compile(tex: &str, id: &str) -> Result<String, String> {
        eprintln!("TikZ: compiling figure {id}");
        let dir = std::env::temp_dir().join(format!("tikz-{id}-{}", std::process::id()));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::write(dir.join("fig.tex"), tex).map_err(|e| e.to_string())?;
        let result = Self::run(
            &dir,
            "latex",
            &["-interaction=nonstopmode", "-halt-on-error", "fig.tex"],
        )
        .and_then(|()| {
            Self::run(
                &dir,
                "dvisvgm",
                &[
                    "--no-fonts",
                    "--bbox=papersize",
                    "--optimize",
                    "fig.dvi",
                    "-o",
                    "fig.svg",
                ],
            )
        })
        .and_then(|()| fs::read_to_string(dir.join("fig.svg")).map_err(|e| e.to_string()));
        let result = result.map_err(|e| match fs::read_to_string(dir.join("fig.log")) {
            // The error, from its `!` line through the `l.<n>` line.
            Ok(log) => {
                let lines: Vec<&str> = log.lines().skip_while(|l| !l.starts_with('!')).collect();
                let end = lines
                    .iter()
                    .position(|l| {
                        l.starts_with("l.") && l[2..].starts_with(|c: char| c.is_ascii_digit())
                    })
                    .map_or(6, |i| i + 2);
                format!("{e}\n{}", lines[..end.min(lines.len())].join("\n"))
            }
            Err(_) => e,
        });
        let _ = fs::remove_dir_all(&dir);
        Ok(Self::tidy(&result?, id))
    }

    fn run(dir: &Path, program: &str, args: &[&str]) -> Result<(), String> {
        let output = Command::new(program)
            .args(args)
            .current_dir(dir)
            .output()
            .map_err(|e| {
                format!("TikZ: couldn't run `{program}`, install TeX Live and dvisvgm: {e}")
            })?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "TikZ: {program} failed\n{}",
                String::from_utf8_lossy(&output.stdout)
            ))
        }
    }

    /// Makes dvisvgm's SVG fit for inlining in a page.
    fn tidy(svg: &str, id: &str) -> String {
        // Drop the XML declaration and comment.
        let svg = &svg[svg.find("<svg").unwrap_or(0)..];
        // dvisvgm names glyphs g0-97 and so on in every file, so give each
        // figure its own ids to keep figures on the same page from sharing
        // glyphs.
        let svg = svg
            .replace(" id='", &format!(" id='tikz-{id}-"))
            .replace("href='#", &format!("href='#tikz-{id}-"))
            .replace("url(#", &format!("url(#tikz-{id}-"));
        // Draw black in the page's text color.
        let svg = svg.replace("='#000'", "='currentColor'");
        // Size in em, taking TeX's 10pt as 1em, so labels match the
        // surrounding text, and MathJax, at any font size.
        let svg = Self::em_size(&svg);
        Text::normalize_whitespace(&svg.replace('\n', " "))
            .trim()
            .to_owned()
    }

    /// The SVG with its width and height in points made ems.
    fn em_size(svg: &str) -> String {
        let attr = |name: &str| -> Option<(usize, usize, f64)> {
            let start = svg.find(&format!(" {name}='"))? + name.len() + 3;
            let end = start + svg[start..].find("pt'")?;
            Some((start, end, svg[start..end].parse().ok()?))
        };
        let (Some(width), Some(height)) = (attr("width"), attr("height")) else {
            return svg.to_owned();
        };
        let mut sizes = [width, height];
        // Replace the later one first, so the other's place stays put.
        sizes.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));
        let mut out = svg.to_owned();
        for (start, end, pt) in sizes {
            out.replace_range(start..end + 2, &format!("{:.4}em", pt / 10.0));
        }
        out
    }
}
