//! Querying the terminal for its colors.
//!
//! OSC 10 and 11 query the text and background colors, and OSC 4 palette
//! entries, and terminals report them with escape sequences on input. The
//! queries go through the terminal itself, so they work over SSH too.
//!
//! DA1 goes last, as an end marker. Terminals report in the order they're
//! queried, so once DA1's report comes, every color report that's coming has
//! come. And since nearly every terminal reports DA1, even one that ignores
//! the color queries, a terminal without them is known right away, with
//! nothing to wait out. Only a terminal that doesn't report DA1 either is
//! waited out, for a second (`TIMEOUT`).

use nixpulvis::Colors;

/// The terminal's colors, or `None` when it doesn't report them all. Call it
/// in raw mode, before anything reads input.
pub fn terminal_colors() -> Option<Colors> {
    #[cfg(unix)]
    return unix::terminal_colors();
    #[cfg(not(unix))]
    None
}

#[cfg(unix)]
mod unix {
    use std::io::{self, IsTerminal, Stdin, Stdout, Write};
    use std::time::{Duration, Instant};

    use nixpulvis::Colors;
    use rustix::event::{PollFd, PollFlags, Timespec, poll};
    use rustix::io::read;

    /// OSC 10, querying the text color.
    const OSC_10: &str = "\x1b]10;?\x07";

    /// OSC 11, querying the background color.
    const OSC_11: &str = "\x1b]11;?\x07";

    /// OSC 4, querying palette entry `n`.
    fn osc_4(n: u8) -> String {
        format!("\x1b]4;{n};?\x07")
    }

    /// DA1, querying the terminal's attributes, like `ESC [ ? 62 ; 22 c` for
    /// a VT220-level terminal with color. Nothing here reads them: its report
    /// only marks the end of the reports before it.
    const DA1: &str = "\x1b[c";

    /// How long to wait for reports, should a terminal not report DA1.
    const TIMEOUT: Duration = Duration::from_secs(1);

    pub(super) fn terminal_colors() -> Option<Colors> {
        let stdin = io::stdin();
        if !stdin.is_terminal() {
            return None;
        }
        let mut stdout = io::stdout();
        send_queries(&mut stdout).ok()?;
        parse(&read_reports(&stdin)?)
    }

    /// Writes the queries, in order: the text, the background, each of the
    /// 16 palette entries, and DA1 last.
    fn send_queries(stdout: &mut Stdout) -> io::Result<()> {
        let mut stdout = stdout.lock();
        write!(stdout, "{OSC_10}{OSC_11}")?;
        for n in 0..16 {
            write!(stdout, "{}", osc_4(n))?;
        }
        write!(stdout, "{DA1}")?;
        stdout.flush()
    }

    /// What the terminal reports, through its DA1 report. `None` when it
    /// doesn't report DA1 in time.
    fn read_reports(stdin: &Stdin) -> Option<Vec<u8>> {
        let deadline = Instant::now() + TIMEOUT;
        let mut reports = Vec::new();
        let mut buffer = [0; 256];
        while !da1_reported(&reports) {
            let left = deadline.checked_duration_since(Instant::now())?;
            let left = Timespec::try_from(left).ok()?;
            if poll(&mut [PollFd::new(stdin, PollFlags::IN)], Some(&left)).ok()? == 0 {
                return None;
            }
            // From the descriptor itself, not `Stdin`'s buffer, which would
            // take in more than it hands back, where `poll` can't see it.
            match read(stdin, &mut buffer).ok()? {
                0 => return None,
                n => reports.extend_from_slice(&buffer[..n]),
            }
        }
        Some(reports)
    }

    /// The colors in `reports`, if every one is there.
    fn parse(reports: &[u8]) -> Option<Colors> {
        let mut foreground = None;
        let mut background = None;
        let mut palette = [None; 16];
        for report in escapes(reports).filter_map(|e| e.strip_prefix(b"]")) {
            // Reports end in BEL, or in ST, `ESC \`, whose `\` starts the
            // next escape instead.
            let report = report.strip_suffix(b"\x07").unwrap_or(report);
            let Ok(report) = std::str::from_utf8(report) else {
                continue;
            };
            match report.split_once(';') {
                Some(("10", spec)) => foreground = rgb(spec),
                Some(("11", spec)) => background = rgb(spec),
                Some(("4", entry)) => {
                    if let Some((index, spec)) = entry.split_once(';')
                        && let Some(slot) =
                            index.parse().ok().and_then(|i: usize| palette.get_mut(i))
                    {
                        *slot = rgb(spec);
                    }
                }
                _ => {}
            }
        }
        let mut colors = [[0; 3]; 16];
        for (color, reported) in colors.iter_mut().zip(palette) {
            *color = reported?;
        }
        Some(Colors::from_terminal(foreground?, background?, colors))
    }

    /// What follows each ESC in `input`.
    fn escapes(input: &[u8]) -> impl Iterator<Item = &[u8]> {
        input.split(|&b| b == 0x1b).skip(1)
    }

    /// An X11 color spec as terminals report it, `rgb:RRRR/GGGG/BBBB`, with
    /// one to four hex digits a channel.
    fn rgb(spec: &str) -> Option<[u8; 3]> {
        let mut channels = spec.strip_prefix("rgb:")?.split('/').map(|hex| {
            if !(1..=4).contains(&hex.len()) {
                return None;
            }
            // Scaled from its digits' range to a byte's, rounding.
            let max = (1u32 << (4 * hex.len())) - 1;
            let value = u32::from_str_radix(hex, 16).ok()?;
            u8::try_from((value * 255 + max / 2) / max).ok()
        });
        let color = [channels.next()??, channels.next()??, channels.next()??];
        channels.next().is_none().then_some(color)
    }

    /// Whether `reports` hold DA1's, `ESC [ ? 62 ; 22 c`: numbers and `;`s,
    /// ending in `c`.
    fn da1_reported(reports: &[u8]) -> bool {
        escapes(reports).any(|escape| {
            escape
                .strip_prefix(b"[?")
                .and_then(|rest| rest.iter().find(|b| !(b.is_ascii_digit() || **b == b';')))
                == Some(&b'c')
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Reports of palette entries `indices`, each `n` as one hex digit a
        /// channel, which is `n * 0x11`, ending in BEL and ST in turn.
        fn palette_reports(indices: impl Iterator<Item = usize>) -> String {
            indices
                .map(|n| {
                    let end = if n % 2 == 0 { "\x07" } else { "\x1b\\" };
                    format!("\x1b]4;{n};rgb:{n:x}/{n:x}/{n:x}{end}")
                })
                .collect()
        }

        #[test]
        fn reads_reports_ending_either_way() {
            let reports = format!(
                "\x1b]10;rgb:caca/caca/caca\x1b\\\x1b]11;rgb:16/16/16\x07{}\x1b[?62;22c",
                palette_reports(0..16),
            );
            assert!(da1_reported(reports.as_bytes()));
            assert_eq!(
                parse(reports.as_bytes()),
                Some(Colors::from_terminal(
                    [0xca, 0xca, 0xca],
                    [0x16, 0x16, 0x16],
                    std::array::from_fn(|n| [n as u8 * 0x11; 3]),
                ))
            );
        }

        #[test]
        fn needs_every_color() {
            // A terminal that only knows the text and background.
            let reports = b"\x1b]10;rgb:caca/caca/caca\x07\x1b]11;rgb:1616/1616/1616\x07\x1b[?1;2c";
            assert_eq!(parse(reports), None);
            // One that doesn't report its last palette entry.
            let reports = format!(
                "\x1b]10;rgb:caca/caca/caca\x07\x1b]11;rgb:1616/1616/1616\x07{}\x1b[?1;2c",
                palette_reports(0..15),
            );
            assert_eq!(parse(reports.as_bytes()), None);
        }

        #[test]
        fn waits_for_all_of_da1() {
            assert!(!da1_reported(b"\x1b]10;rgb:caca/caca/caca\x07"));
            assert!(!da1_reported(b"\x1b[?62;2"));
            assert!(da1_reported(b"\x1b[?62;22c"));
        }
    }
}
