//! Working out where links go.

/// Where following a link leads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    /// A page on the site, by its path, with an optional `#` section.
    Page {
        path: String,
        anchor: Option<String>,
    },
    /// A section of the current page.
    Anchor(String),
    /// Anything else, like another site or a PDF here, to hand off to a
    /// browser.
    External(String),
}

impl Target {
    /// Where `href`, found on the page at `current`, leads. `origin` is the
    /// site's address, so absolute links back to it stay inside.
    pub(crate) fn resolve(href: &str, current: &str, origin: &str) -> Self {
        let href = href.trim();
        if let Some(anchor) = href.strip_prefix('#') {
            return Target::Anchor(anchor.to_owned());
        }
        let href = match href.strip_prefix(origin.trim_end_matches('/')) {
            Some(rest) if !origin.is_empty() && (rest.is_empty() || rest.starts_with('/')) => {
                if rest.is_empty() {
                    "/"
                } else {
                    rest
                }
            }
            _ => href,
        };
        if href.contains("://") || href.starts_with("mailto:") || href.starts_with("//") {
            return Target::External(href.to_owned());
        }

        let (href, anchor) = match href.split_once('#') {
            Some((h, a)) => (h, Some(a.to_owned())),
            None => (href, None),
        };
        let href = href.split('?').next().unwrap_or("");
        let path = if href.starts_with('/') {
            normalize(href)
        } else {
            let dir = &current[..current.rfind('/').map_or(0, |i| i + 1)];
            normalize(&format!("{dir}{href}"))
        };

        let last = path.rsplit('/').next().unwrap_or("");
        match last.rsplit_once('.') {
            Some((stem, "html")) => Target::Page {
                path: format!("{}{stem}", &path[..path.len() - last.len()]),
                anchor,
            },
            Some(_) => Target::External(format!("{}{path}", origin.trim_end_matches('/'))),
            None => Target::Page { path, anchor },
        }
    }
}

/// The page a host starts on at `address`, and its `#` section. It may be an
/// old `.html` address.
pub(crate) fn page_at(address: &str) -> (String, Option<String>) {
    match Target::resolve(address, "/", "") {
        Target::Page { path, anchor } => (path, anchor),
        _ => (address.to_owned(), None),
    }
}

/// Where a page's Markdown is published: `/foo` at `/foo.md`, and `/foo/` at
/// `/foo/index.md`.
pub(crate) fn source(path: &str) -> String {
    if path.ends_with('/') {
        format!("{path}index.md")
    } else {
        format!("{path}.md")
    }
}

/// Whether two paths are the same page, with or without a trailing slash.
pub(crate) fn same_page(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

/// Resolves `.` and `..` in an absolute path.
fn normalize(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    let mut out = format!("/{}", parts.join("/"));
    if (path.ends_with('/') || path.ends_with("/.") || path.ends_with("/..")) && out != "/" {
        out.push('/');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGIN: &str = "https://nixpulvis.com";

    fn resolve(href: &str, current: &str) -> Target {
        Target::resolve(href, current, ORIGIN)
    }

    fn page(path: &str) -> Target {
        Target::Page {
            path: path.into(),
            anchor: None,
        }
    }

    #[test]
    fn site_paths() {
        assert_eq!(resolve("/ramblings", "/"), page("/ramblings"));
        assert_eq!(resolve("https://nixpulvis.com/about", "/"), page("/about"));
        assert_eq!(resolve("https://nixpulvis.com", "/x"), page("/"));
        assert_eq!(resolve("/old/page.html", "/"), page("/old/page"));
    }

    #[test]
    fn relative_paths() {
        assert_eq!(resolve("seen", "/MTk3ODkK/"), page("/MTk3ODkK/seen"));
        assert_eq!(resolve("../b", "/a/x/y"), page("/a/b"));
        assert_eq!(resolve("./", "/a/b"), page("/a/"));
    }

    #[test]
    fn anchors() {
        assert_eq!(resolve("#intro", "/x"), Target::Anchor("intro".into()));
        assert_eq!(
            resolve("/x#intro", "/"),
            Target::Page {
                path: "/x".into(),
                anchor: Some("intro".into())
            }
        );
    }

    #[test]
    fn external() {
        assert_eq!(
            resolve("https://github.com/x", "/"),
            Target::External("https://github.com/x".into())
        );
        assert_eq!(
            resolve("mailto:a@b.c", "/"),
            Target::External("mailto:a@b.c".into())
        );
        assert_eq!(
            resolve("/resume.pdf", "/"),
            Target::External("https://nixpulvis.com/resume.pdf".into())
        );
    }

    #[test]
    fn start_pages() {
        assert_eq!(page_at("/ramblings/x.html"), ("/ramblings/x".into(), None));
        assert_eq!(page_at("/a#b"), ("/a".into(), Some("b".into())));
    }

    #[test]
    fn sources() {
        assert_eq!(source("/"), "/index.md");
        assert_eq!(source("/fortunes/"), "/fortunes/index.md");
        assert_eq!(source("/ramblings/x"), "/ramblings/x.md");
    }
}
