//! The fortunes, a year to a file in `fortunes/`, published as they are
//! along with a `strfile` database for `fortune` to read.

use std::fs;
use std::path::Path;

use cratesite::Text;

/// A year of fortunes, from `fortunes/<year>.txt`. Entries are separated by
/// a line holding only `%`.
#[derive(Debug)]
pub struct Fortunes {
    pub year: String,
    /// The whole file.
    pub text: String,
    pub entries: Vec<String>,
}

impl Fortunes {
    /// Every year's fortunes in `dir`, newest year first. A `%` with
    /// anything after it, or an empty entry, is an error, since `strfile`
    /// would quietly merge or keep them.
    pub fn read(dir: &Path) -> Result<Vec<Self>, String> {
        let mut years = Vec::new();
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(year) = name.strip_suffix(".txt") else {
                continue;
            };
            if year.len() != 4 || !year.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let text = fs::read_to_string(entry.path()).map_err(|e| format!("{name}: {e}"))?;
            years.push(Self::parse(year, text, &format!("fortunes/{name}"))?);
        }
        years.sort_by(|a, b| b.year.cmp(&a.year));
        Ok(years)
    }

    fn parse(year: &str, text: String, name: &str) -> Result<Self, String> {
        for (n, line) in text.split('\n').enumerate() {
            if line.starts_with('%') && line != "%" && line[1..].trim().is_empty() {
                return Err(format!(
                    "{name}:{}: `%` separator has trailing whitespace",
                    n + 1
                ));
            }
        }
        let mut entries = Vec::new();
        let mut entry = String::new();
        let mut line_number = 0;
        for (n, line) in text.trim().split('\n').enumerate() {
            if line == "%" {
                if entry.trim().is_empty() {
                    return Err(format!("{name}:{}: empty fortune", n + 1));
                }
                entries.push(entry.trim().to_owned());
                entry.clear();
                line_number = n + 1;
            } else {
                entry.push_str(line);
                entry.push('\n');
            }
        }
        if entry.trim().is_empty() {
            return Err(format!("{name}:{line_number}: empty fortune"));
        }
        entries.push(entry.trim().to_owned());
        Ok(Self {
            year: year.to_owned(),
            text,
            entries,
        })
    }

    /// An entry as HTML, its lines kept with `<br />`.
    pub fn html(&self, entry: &str) -> String {
        Text::strip_newlines(&Text::newline_to_br(&Text::escape(entry)))
    }

    /// Where the year's text is published, like `/fortunes/2018.txt`.
    pub fn path(&self) -> String {
        format!("/fortunes/{}.txt", self.year)
    }

    /// Where the year's `strfile` database is published.
    pub fn dat(&self) -> String {
        format!("{}.dat", self.path())
    }

    /// The year's database, the way `strfile` writes it: a header of
    /// big-endian numbers, then where each fortune starts in the text, and
    /// where the text ends.
    pub fn strfile(&self) -> Vec<u8> {
        let mut offsets = vec![0u32];
        let mut lengths = Vec::new();
        let mut start = 0;
        let mut position = 0;
        for line in self.text.split_inclusive('\n') {
            if line == "%\n" || line == "%" {
                lengths.push(position - start);
                start = position + line.len();
                offsets.push(start as u32);
            }
            position += line.len();
        }
        if position > start {
            lengths.push(position - start);
            offsets.push(position as u32);
        }
        let mut dat = Vec::new();
        let header = [
            2,
            lengths.len() as u32,
            lengths.iter().max().copied().unwrap_or(0) as u32,
            lengths.iter().min().copied().unwrap_or(0) as u32,
            0,
        ];
        for n in header {
            dat.extend(n.to_be_bytes());
        }
        dat.extend([b'%', 0, 0, 0]);
        for offset in offsets {
            dat.extend(offset.to_be_bytes());
        }
        dat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strfile() {
        let fortunes =
            Fortunes::parse("2018", "One.\n%\nTwo,\ntwo.\n%\nThree.\n".into(), "x").unwrap();
        assert_eq!(fortunes.entries, ["One.", "Two,\ntwo.", "Three."]);
        let dat = fortunes.strfile();
        let words: Vec<u32> = dat
            .chunks(4)
            .map(|c| u32::from_be_bytes(c.try_into().unwrap()))
            .collect();
        assert_eq!(&words[..5], [2, 3, 10, 5, 0]);
        assert_eq!(&words[6..], [0, 7, 19, 26]);
    }

    #[test]
    fn bad_separators() {
        assert!(Fortunes::parse("x", "a\n% \nb\n".into(), "x").is_err());
        assert!(Fortunes::parse("x", "a\n%\n%\nb\n".into(), "x").is_err());
        assert!(Fortunes::parse("x", "a\n%\n".into(), "x").is_err());
    }
}
