//! The site's collections, and where their pages and the others are
//! published. `build.rs` reads this file too, to lay out the page modules.

/// A folder of pages of one kind, like `_ramblings`, published under a
/// common path.
#[derive(Debug, PartialEq, Eq)]
pub struct Collection {
    /// Like `ramblings`, for the folder `_ramblings`.
    pub label: &'static str,
    /// The path each page is published at, with `:name` for the page's
    /// slug, the file's name without its date.
    pub permalink: &'static str,
}

impl Collection {
    /// Every collection, in the order `site.json` lists their pages.
    pub const ALL: &[Collection] = &[
        Collection {
            label: "mathematics",
            permalink: "/math/:name",
        },
        Collection {
            label: "musings",
            permalink: "/musings/:name",
        },
        Collection {
            label: "projects",
            permalink: "/projects/:name",
        },
        Collection {
            label: "ramblings",
            permalink: "/ramblings/:name",
        },
        Collection {
            label: "research",
            permalink: "/research/:name",
        },
    ];

    /// The collection with the label `label`.
    pub fn named(label: &str) -> Option<&'static Collection> {
        Self::ALL.iter().find(|c| c.label == label)
    }

    /// The collection whose folder holds the file at `path`, from the top of
    /// the repository.
    pub fn holding(path: &str) -> Option<&'static Collection> {
        let dir = path.split('/').next()?.strip_prefix('_')?;
        Self::named(dir)
    }

    /// The folder the collection's pages are in, like `_ramblings`.
    pub fn dir(&self) -> String {
        format!("_{}", self.label)
    }

    /// Where the page in the file `stem`, without its extension, is
    /// published. Its name is made a slug, so `00_euler` is at
    /// `/math/00-euler`.
    pub fn url(&self, stem: &str) -> String {
        self.permalink.replace(":name", &Permalink::slug(stem))
    }
}

/// Which files in the repository are part of the site.
pub struct Files;

impl Files {
    /// Files and folders at the top of the repository that aren't part of
    /// the site.
    pub const EXCLUDE: &[&str] = &[
        "Cargo.lock",
        "Cargo.toml",
        "README.md",
        "TODO-espresso",
        "askama.toml",
        "bin",
        "build.rs",
        "src",
        "target",
        "templates",
        "tui",
    ];

    /// Whether the file or folder at `path`, from the top of the
    /// repository, is left out of the site. Names starting with `_` or `.`
    /// are, like collections' folders, which are read separately.
    pub fn ignored(path: &str) -> bool {
        path.split('/')
            .next()
            .is_some_and(|top| Self::EXCLUDE.contains(&top))
            || path
                .split('/')
                .any(|name| name.starts_with('_') || name.starts_with('.'))
    }
}

/// Where pages are published.
pub struct Permalink;

impl Permalink {
    /// Where the page in the file at `path` is published, from the top of
    /// the repository, unless its front matter gives a `permalink`.
    ///
    /// Pages in a collection go where the collection says. Others keep their
    /// path, as `.html`, except an `index` which is its folder.
    pub fn of(path: &str) -> String {
        let Some(stem) = path.strip_suffix(".md") else {
            return format!("/{path}");
        };
        if let Some(collection) = Collection::holding(path) {
            let name = stem.rsplit('/').next().unwrap_or(stem);
            return collection.url(name);
        }
        match stem.strip_suffix("index") {
            Some(dir) if dir.is_empty() || dir.ends_with('/') => format!("/{dir}"),
            _ => format!("/{stem}.html"),
        }
    }

    /// The name lowercased, with anything but letters and digits made a
    /// dash, the way Jekyll makes slugs.
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

    #[test]
    fn permalinks() {
        assert_eq!(
            Permalink::of("_ramblings/2018-02-05-hello-world.md"),
            "/ramblings/2018-02-05-hello-world"
        );
        assert_eq!(Permalink::of("_mathematics/00_euler.md"), "/math/00-euler");
        assert_eq!(
            Permalink::of("_research/notes/achilles/2019-10-13-achilles-8.md"),
            "/research/2019-10-13-achilles-8"
        );
        assert_eq!(Permalink::of("index.md"), "/");
        assert_eq!(Permalink::of("MTk3ODkK/index.md"), "/MTk3ODkK/");
        assert_eq!(
            Permalink::of("MTk3ODkK/dreams/1.md"),
            "/MTk3ODkK/dreams/1.html"
        );
        assert_eq!(Permalink::of("about.md"), "/about.html");
        assert_eq!(Permalink::of("feed.xml"), "/feed.xml");
    }

    #[test]
    fn dates() {
        assert_eq!(
            Permalink::date("2018-02-05-hello-world"),
            Some("2018-02-05")
        );
        assert_eq!(Permalink::date("00_euler"), None);
        assert_eq!(Permalink::date("2018-02-05"), None);
    }
}
