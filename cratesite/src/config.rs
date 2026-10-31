//! What a site says about itself: its name and address, its collections,
//! and which files are part of it.
//!
//! A site writes its [`Config`] as a `const` in a file of its own, which
//! both its build script and the crate itself include, so the pages are
//! laid out the same way when they're made modules and when they're built.

/// A site's settings.
#[derive(Debug)]
pub struct Config {
    /// The site's name, like `nixpulvis`.
    pub title: &'static str,
    /// Where it's published, like `https://nixpulvis.com`, without a `/` at
    /// the end.
    pub url: &'static str,
    /// What the site is, for pages without a summary of their own.
    pub description: &'static str,
    /// Where its source lives, like `https://github.com/you/site`, for
    /// links to each page's source from the docs.
    pub repository: Option<&'static str>,
    /// Folders of pages of one kind, like ramblings or projects.
    pub collections: &'static [Collection],
    /// Files and folders at the top of the crate that aren't part of the
    /// site, like `Cargo.toml` and `src`. `target` and names starting with
    /// `_` or `.` never are.
    pub exclude: &'static [&'static str],
    /// Folders whose pages are published but left out of the docs.
    pub unlisted: &'static [&'static str],
    /// What a build leaves in the destination, for things built another
    /// way, like `doc` for the crate's own docs.
    pub keep: &'static [&'static str],
    /// LaTeX put in every TikZ figure's document, for packages and TikZ
    /// libraries, like `\usepackage{tikz-cd}`.
    pub tikz_preamble: &'static str,
}

/// A folder of pages of one kind, like `_ramblings`, published under a
/// common path.
#[derive(Debug, PartialEq, Eq)]
pub struct Collection {
    /// Like `ramblings`, for the folder `_ramblings`.
    pub label: &'static str,
    /// The path each page is published at, with `:name` for the page's
    /// slug, the file's name made a slug, date and all.
    pub permalink: &'static str,
}

impl Config {
    /// The collection with the label `label`.
    pub fn collection(&self, label: &str) -> Option<&'static Collection> {
        self.collections.iter().find(|c| c.label == label)
    }

    /// The collection whose folder holds the file at `path`, from the top
    /// of the crate.
    pub fn holding(&self, path: &str) -> Option<&'static Collection> {
        let dir = path.split('/').next()?.strip_prefix('_')?;
        self.collection(dir)
    }

    /// Whether the file or folder at `path`, from the top of the crate, is
    /// left out of the site. Names starting with `_` or `.` are, like
    /// collections' folders, which are read separately.
    pub fn ignored(&self, path: &str) -> bool {
        path.split('/')
            .next()
            .is_some_and(|top| top == "target" || self.exclude.contains(&top))
            || path
                .split('/')
                .any(|name| name.starts_with('_') || name.starts_with('.'))
    }

    /// Whether the page at `path` is left out of the docs.
    pub fn unlisted(&self, path: &str) -> bool {
        self.unlisted
            .iter()
            .any(|dir| path.strip_prefix(dir).is_some_and(|p| p.starts_with('/')))
    }

    /// Where the page in the file at `path` is published, from the top of
    /// the crate, unless its front matter gives a `permalink`.
    ///
    /// Pages in a collection go where the collection says. Others keep their
    /// path, as `.html`, except an `index` which is its folder. Files that
    /// aren't Markdown keep their path as it is.
    pub fn permalink(&self, path: &str) -> String {
        let Some(stem) = path.strip_suffix(".md") else {
            return format!("/{path}");
        };
        if let Some(collection) = self.holding(path) {
            let name = stem.rsplit('/').next().unwrap_or(stem);
            return collection.url(name);
        }
        match stem.strip_suffix("index") {
            Some(dir) if dir.is_empty() || dir.ends_with('/') => format!("/{dir}"),
            _ => format!("/{stem}.html"),
        }
    }
}

impl Collection {
    /// The folder the collection's pages are in, like `_ramblings`.
    pub fn dir(&self) -> String {
        format!("_{}", self.label)
    }

    /// Where the page in the file `stem`, without its extension, is
    /// published. Its name is made a slug, so `00_euler` in a collection at
    /// `/math/:name` is at `/math/00-euler`.
    pub fn url(&self, stem: &str) -> String {
        self.permalink.replace(":name", &Name::slug(stem))
    }
}

/// What a page's file name says about it.
pub struct Name;

impl Name {
    /// The name lowercased, with anything but letters and digits made a
    /// dash.
    pub fn slug(name: &str) -> String {
        let mut slug = String::new();
        for c in name.chars() {
            if c.is_ascii_alphanumeric() {
                slug.push(c.to_ascii_lowercase());
            } else if !slug.ends_with('-') {
                slug.push('-');
            }
        }
        slug.trim_matches('-').to_owned()
    }

    /// A title from a file's name, like `Led Cube` from `led-cube`, without
    /// a date at the start.
    pub fn title(name: &str) -> String {
        let name = match Self::date(name) {
            Some(date) => &name[date.len() + 1..],
            None => name,
        };
        name.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first
                        .to_uppercase()
                        .chain(chars.flat_map(char::to_lowercase))
                        .collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// The date at the start of a file's name, like `2018-02-05` in
    /// `2018-02-05-hello-world`.
    pub fn date(name: &str) -> Option<&str> {
        let date = name.get(..10)?;
        let digits = |r: std::ops::Range<usize>| date[r].bytes().all(|b| b.is_ascii_digit());
        let dashes = date.as_bytes()[4] == b'-' && date.as_bytes()[7] == b'-';
        (digits(0..4) && digits(5..7) && digits(8..10) && dashes && name[10..].starts_with('-'))
            .then_some(date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: Config = Config {
        title: "test",
        url: "https://example.com",
        description: "",
        repository: None,
        collections: &[
            Collection {
                label: "mathematics",
                permalink: "/math/:name",
            },
            Collection {
                label: "ramblings",
                permalink: "/ramblings/:name",
            },
        ],
        exclude: &["src"],
        unlisted: &["secret"],
        keep: &[],
        tikz_preamble: "",
    };

    #[test]
    fn permalinks() {
        assert_eq!(
            CONFIG.permalink("_ramblings/2018-02-05-hello-world.md"),
            "/ramblings/2018-02-05-hello-world"
        );
        assert_eq!(
            CONFIG.permalink("_mathematics/00_euler.md"),
            "/math/00-euler"
        );
        assert_eq!(CONFIG.permalink("index.md"), "/");
        assert_eq!(CONFIG.permalink("secret/index.md"), "/secret/");
        assert_eq!(
            CONFIG.permalink("secret/dreams/1.md"),
            "/secret/dreams/1.html"
        );
        assert_eq!(CONFIG.permalink("about.md"), "/about.html");
        assert_eq!(CONFIG.permalink("feed.xml"), "/feed.xml");
    }

    #[test]
    fn files() {
        assert!(CONFIG.ignored("src/lib.rs"));
        assert!(CONFIG.ignored("_ramblings/a.md"));
        assert!(CONFIG.ignored("img/.DS_Store"));
        assert!(!CONFIG.ignored("img/a.png"));
        assert!(CONFIG.unlisted("secret/index.md"));
        assert!(!CONFIG.unlisted("secrets.md"));
    }

    #[test]
    fn names() {
        assert_eq!(Name::date("2018-02-05-hello-world"), Some("2018-02-05"));
        assert_eq!(Name::date("00_euler"), None);
        assert_eq!(Name::date("2018-02-05"), None);
        assert_eq!(Name::title("2018-02-05-led-cube"), "Led Cube");
    }
}
