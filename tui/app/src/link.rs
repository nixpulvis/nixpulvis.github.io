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
    /// site's address, without a trailing `/`, so absolute links back to it
    /// stay inside.
    pub(crate) fn resolve(href: &str, current: &str, origin: &str) -> Self {
        let href = href.trim();
        if let Some(anchor) = href.strip_prefix('#') {
            return Target::Anchor(anchor.to_owned());
        }
        let href = match href.strip_prefix(origin) {
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
            Some(_) => Target::External(format!("{origin}{path}")),
            None => Target::Page { path, anchor },
        }
    }
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
}
