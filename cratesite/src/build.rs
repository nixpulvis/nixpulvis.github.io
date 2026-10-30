//! The build script helper, which makes a module for every page of a site,
//! in `$OUT_DIR/pages.rs`, for the site's crate to include.
//!
//! Each module is documented with its page, rewritten for rustdoc, and
//! holds the page's source as `PAGE`. Modules nest like the site's URLs, so
//! `/posts/2018-02-05-hello-world` is `posts::hello_world`. Each page with
//! template tags also gets an askama template, in `$OUT_DIR/templates.rs`,
//! for a module with `Site` and `Page` in scope to include.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
#[cfg(feature = "kramdown")]
use ratmark::kramdown::{AttributeList, Kramdown};

use crate::{Config, Name};

/// Makes the page modules, from a site's build script:
///
/// ```no_run
/// use cratesite::{BuildScript, Collection, Config};
///
/// const CONFIG: Config = Config {
///     title: "My site",
///     url: "https://example.com",
///     description: "Things I wrote.",
///     repository: None,
///     collections: &[Collection {
///         label: "posts",
///         permalink: "/posts/:name",
///     }],
///     exclude: &["Cargo.lock", "Cargo.toml", "build.rs", "src"],
///     unlisted: &[],
///     keep: &[],
///     tikz_preamble: "",
/// };
///
/// fn main() {
///     BuildScript::new(&CONFIG).run();
/// }
/// ```
///
/// The site's `lib.rs` then includes them:
///
/// ```ignore
/// include!(concat!(env!("OUT_DIR"), "/pages.rs"));
/// ```
///
/// The home page's docs are written to `$OUT_DIR/doc/crate.md`, for the
/// crate's own docs to include with `#![doc = include_str!(...)]`.
pub struct BuildScript {
    config: &'static Config,
    root: PathBuf,
    out: PathBuf,
}

impl BuildScript {
    /// For the crate cargo is building, with its pages laid out by `config`.
    pub fn new(config: &'static Config) -> Self {
        Self {
            config,
            root: PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()),
            out: PathBuf::from(env::var("OUT_DIR").unwrap()),
        }
    }

    /// Writes `pages.rs`, `templates.rs` and the docs, and tells cargo
    /// when to run again.
    pub fn run(self) {
        let pages = Page::find(self.config, &self.root);
        let mut tree = Module::default();
        for (i, page) in pages.iter().enumerate() {
            if let Some(segments) = page.segments() {
                tree.insert(&segments, i);
            }
        }
        let links = tree.links(&pages, "crate");

        let docs = self.out.join("doc");
        let _ = fs::remove_dir_all(&docs);
        let mut code = String::new();
        tree.write(
            self.config,
            &pages,
            &links,
            &self.root,
            &docs,
            "crate",
            &mut code,
        );
        // Every source, for the generator, including the ones that aren't
        // Markdown and so aren't modules, like `feed.xml`.
        code.push_str("\n/// Every page's source.\n#[allow(dead_code)]\npub static SOURCES: &[&::cratesite::Source] = &[\n");
        for page in &pages {
            match &page.module {
                Some(module) => writeln!(code, "    &{module}::PAGE,").unwrap(),
                None => writeln!(code, "    &{},", page.source(&self.root)).unwrap(),
            }
        }
        code.push_str("];\n");
        fs::write(self.out.join("pages.rs"), code).unwrap();

        let mut templates = String::new();
        for page in pages.iter().filter(|p| p.template) {
            templates.push_str(&page.template_struct());
        }
        templates.push_str("\n/// Renders the template page at `path`.\n#[allow(dead_code)]\npub(crate) fn render(path: &str, site: &Site, page: &Page) -> Option<askama::Result<String>> {\n    match path {\n");
        for page in pages.iter().filter(|p| p.template) {
            writeln!(
                templates,
                "        {:?} => Some({} {{ site, page }}.render()),",
                page.path,
                page.struct_name()
            )
            .unwrap();
        }
        templates.push_str("        _ => None,\n    }\n}\n");
        fs::write(self.out.join("templates.rs"), templates).unwrap();
    }
}

/// A page of the site: a file with front matter.
struct Page {
    /// From the top of the crate.
    path: String,
    url: String,
    text: String,
    front: serde_yaml::Mapping,
    /// Left out of the docs, like it is from the site.
    hidden: bool,
    /// Whether it uses template tags, rendered with askama before anything
    /// else.
    template: bool,
    /// The path of its module, once it has one.
    module: Option<String>,
}

impl Page {
    /// Every page, collections first.
    fn find(config: &Config, root: &Path) -> Vec<Self> {
        let mut files = Vec::new();
        for collection in config.collections {
            let dir = root.join(collection.dir());
            println!("cargo:rerun-if-changed={}", dir.display());
            walk(&dir, &mut files);
        }
        let mut top = Vec::new();
        walk(root, &mut top);
        files.extend(top.into_iter().filter(|f| {
            let path = relative(root, f);
            !config.ignored(&path) && !path.starts_with("_")
        }));

        let mut pages: Vec<Self> = files
            .into_iter()
            .filter_map(|file| {
                let path = relative(root, &file);
                let name = path.rsplit('/').next().unwrap();
                if name.starts_with('_') && config.holding(&path).is_some() {
                    return None;
                }
                let text = fs::read_to_string(&file).ok()?;
                if !text.starts_with("---\n") {
                    return None;
                }
                println!("cargo:rerun-if-changed={}", file.display());
                let yaml = crate::front::split(&text).0;
                let front = serde_yaml::from_str(yaml).unwrap_or_default();
                let template = text.contains("{%") || text.contains("{{");
                let mut page = Self {
                    url: String::new(),
                    hidden: config.unlisted(&path),
                    path,
                    text,
                    front,
                    template,
                    module: None,
                };
                page.url = page
                    .get("permalink")
                    .unwrap_or_else(|| config.permalink(&page.path));
                page.hidden |= page.get("published").as_deref() == Some("false");
                Some(page)
            })
            .collect();
        pages.sort_by(|a, b| a.path.cmp(&b.path));
        pages
    }

    /// The module path segments for the page's URL, or none for files that
    /// aren't Markdown.
    fn segments(&self) -> Option<Vec<String>> {
        if !self.path.ends_with(".md") {
            return None;
        }
        let url = self.url.trim_end_matches('/');
        let url = url.strip_suffix(".html").unwrap_or(url);
        Some(
            url.split('/')
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect(),
        )
    }

    /// The front matter's field `key`, if it's a string, number or bool.
    fn get(&self, key: &str) -> Option<String> {
        match self.front.get(key)? {
            serde_yaml::Value::String(s) => Some(s.clone()),
            serde_yaml::Value::Number(n) => Some(n.to_string()),
            serde_yaml::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// The `Source` literal for the page.
    fn source(&self, root: &Path) -> String {
        format!(
            "::cratesite::Source {{ path: {:?}, text: include_str!({:?}) }}",
            self.path,
            root.join(&self.path)
        )
    }

    fn struct_name(&self) -> String {
        let mut name = String::from("Page");
        for part in self.path.split(|c: char| !c.is_ascii_alphanumeric()) {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                name.push(first.to_ascii_uppercase());
                name.push_str(chars.as_str());
            }
        }
        name
    }

    fn template_struct(&self) -> String {
        format!(
            "\n/// The template in `{path}`.\n#[derive(askama::Template)]\n#[template(path = {path:?}, escape = \"none\")]\nstruct {name}<'a> {{\n    site: &'a Site,\n    #[allow(dead_code)]\n    page: &'a Page,\n}}\n",
            path = self.path,
            name = self.struct_name(),
        )
    }

    /// The page as rustdoc docs: its title and date, then its Markdown with
    /// template tags taken out, kramdown's extras made plain, code fences
    /// marked so they aren't run as tests, and links to other pages made
    /// links to their modules.
    fn doc(&self, config: &Config, links: &BTreeMap<String, String>) -> String {
        let body = crate::front::split(&self.text).1;
        let body = if self.template {
            Template::strip(body)
        } else {
            body.to_owned()
        };
        #[cfg(feature = "kramdown")]
        let body = Kramdown::to_commonmark(&body).into_owned();
        let mut doc = String::new();
        let title = self.get("title").unwrap_or_else(|| {
            let name = self.path.rsplit('/').next().unwrap();
            Name::title(name.trim_end_matches(".md"))
        });
        doc.push_str(&title);
        doc.push_str("\n\n");
        let mut meta = Vec::new();
        let name = self.path.rsplit('/').next().unwrap();
        if let Some(date) = self
            .get("date")
            .or_else(|| Name::date(name).map(str::to_owned))
        {
            meta.push(date);
        }
        if self.get("draft").as_deref() == Some("true") {
            meta.push("draft".to_owned());
        }
        meta.push(format!("[on the site]({}{})", config.url, self.url));
        if let Some(repository) = config.repository {
            meta.push(format!("[source]({repository}/blob/master/{})", self.path));
        }
        writeln!(doc, "<small>{}</small>\n", meta.join(" · ")).unwrap();
        doc.push_str(&Rustdoc::rewrite(&body, config.url, &self.url, links));
        doc
    }
}

/// A module of pages, with the page at its own URL, if there is one, and
/// those under it.
#[derive(Default)]
struct Module {
    page: Option<usize>,
    children: BTreeMap<String, Module>,
}

impl Module {
    fn insert(&mut self, segments: &[String], page: usize) {
        match segments.split_first() {
            None => self.page = Some(page),
            Some((first, rest)) => self
                .children
                .entry(first.clone())
                .or_default()
                .insert(rest, page),
        }
    }

    /// The identifier for each child, unique among them.
    fn idents(&self) -> Vec<(String, &String, &Module)> {
        let mut taken = Vec::new();
        let mut out = Vec::new();
        for (segment, child) in &self.children {
            let base = ident(segment);
            let mut name = base.clone();
            let mut n = 1;
            while taken.contains(&name) {
                n += 1;
                name = format!("{base}_{n}");
            }
            taken.push(name.clone());
            out.push((name, segment, child));
        }
        out
    }

    /// Every page's URL, without `.html` or a trailing `/`, to the path of
    /// its module.
    fn links(&self, pages: &[Page], path: &str) -> BTreeMap<String, String> {
        let mut links = BTreeMap::new();
        if let Some(i) = self.page {
            links.insert(link_key(&pages[i].url), path.to_owned());
        }
        for (name, _, child) in self.idents() {
            links.extend(child.links(pages, &format!("{path}::{name}")));
        }
        links
    }

    #[allow(clippy::too_many_arguments)]
    fn write(
        &self,
        config: &Config,
        pages: &[Page],
        links: &BTreeMap<String, String>,
        root: &Path,
        docs: &Path,
        path: &str,
        code: &mut String,
    ) {
        if let Some(i) = self.page {
            let page = &pages[i];
            // The home page's docs follow the crate's own, in lib.rs.
            let name = if path == "crate" {
                "crate".to_owned()
            } else {
                path.replace("::", "/")
            };
            let file = docs.join(format!("{name}.md"));
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, page.doc(config, links)).unwrap();
            writeln!(
                code,
                "/// This page's source, which the site is built from.\npub static PAGE: ::cratesite::Source = {};",
                page.source(root)
            )
            .unwrap();
        }
        for (name, _, child) in self.idents() {
            let child_path = format!("{path}::{name}");
            match child.page {
                Some(i) => {
                    let file = docs.join(format!("{}.md", child_path.replace("::", "/")));
                    if pages[i].hidden {
                        code.push_str("#[doc(hidden)]\n");
                    }
                    writeln!(code, "#[doc = include_str!({file:?})]").unwrap();
                }
                None => code.push_str("#[doc(hidden)]\n"),
            }
            // The pages are prose, not API docs.
            code.push_str("#[allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation, clippy::doc_markdown, rustdoc::all)]\n");
            writeln!(code, "pub mod {name} {{").unwrap();
            child.write(config, pages, links, root, docs, &child_path, code);
            code.push_str("}\n");
        }
    }
}

/// The module name for a URL's path segment: no date or number in front,
/// in lowercase, with underscores.
fn ident(segment: &str) -> String {
    if segment == "404" {
        return "not_found".to_owned();
    }
    let mut name = segment;
    if let Some(date) = Name::date(name) {
        name = &name[date.len() + 1..];
    }
    let digits = name.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 && name[digits..].starts_with(['-', '_']) {
        name = &name[digits + 1..];
    }
    let mut ident: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if ident.starts_with(|c: char| c.is_ascii_digit()) {
        ident.insert(0, '_');
    }
    if [
        "self", "super", "crate", "mod", "type", "fn", "use", "loop", "move",
    ]
    .contains(&ident.as_str())
    {
        ident.push('_');
    }
    ident
}

/// A URL as the key of the links map.
fn link_key(url: &str) -> String {
    let url = url.strip_suffix(".html").unwrap_or(url);
    url.trim_end_matches('/').to_owned()
}

/// Template tags, for docs of pages with them.
struct Template;

impl Template {
    /// Takes out `{% %}` and `{{ }}` tags, and `<style>` and `<script>`
    /// elements, which are usually made by them.
    fn strip(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        loop {
            let next = [
                ("{%", "%}"),
                ("{{", "}}"),
                ("<style", "</style>"),
                ("<script", "</script>"),
            ]
            .iter()
            .filter_map(|(open, close)| rest.find(open).map(|i| (i, *close)))
            .min_by_key(|(i, _)| *i);
            let Some((start, close)) = next else { break };
            out.push_str(&rest[..start]);
            match rest[start..].find(close) {
                Some(end) => rest = &rest[start + end + close.len()..],
                None => {
                    rest = "";
                    break;
                }
            }
        }
        out.push_str(rest);
        // What's left of the HTML the tags filled in, like empty lists, is
        // dropped, leaving the page's prose. The docs list the pages under
        // it anyway.
        let mut prose = String::with_capacity(out.len());
        for line in out.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('<') {
                continue;
            }
            if trimmed.is_empty() && (prose.is_empty() || prose.ends_with("\n\n")) {
                continue;
            }
            prose.push_str(if trimmed.is_empty() { "" } else { line });
            prose.push('\n');
        }
        prose
    }
}

/// Rewrites Markdown for rustdoc.
struct Rustdoc;

impl Rustdoc {
    fn rewrite(
        markdown: &str,
        site_url: &str,
        url: &str,
        links: &BTreeMap<String, String>,
    ) -> String {
        let mut edits: Vec<(std::ops::Range<usize>, String)> = Vec::new();
        let parser = Parser::new_ext(markdown, Options::all() - Options::ENABLE_SMART_PUNCTUATION);
        let definitions: Vec<_> = parser
            .reference_definitions()
            .iter()
            .map(|(_, def)| (def.span.clone(), def.dest.to_string()))
            .collect();
        for (span, dest) in definitions {
            if let Some(link) = Self::link(&dest, site_url, url, links) {
                Self::replace_last(markdown, span, &dest, &link, &mut edits);
            }
        }
        let mut paragraph: Vec<Event> = Vec::new();
        for (event, range) in parser.into_offset_iter() {
            match &event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    let block = &markdown[range.clone()];
                    match kind {
                        CodeBlockKind::Fenced(info) => {
                            let line_end = block.find('\n').unwrap_or(block.len());
                            let line = &block[..line_end];
                            let fence: String = line
                                .trim_start()
                                .chars()
                                .take_while(|&c| c == '`' || c == '~')
                                .collect();
                            let indent = &line[..line.len() - line.trim_start().len()];
                            let lang = info.split_whitespace().next().unwrap_or("");
                            let lang = match lang {
                                "" => "text",
                                "rust" => "rust,ignore",
                                lang => lang,
                            };
                            edits.push((
                                range.start..range.start + line_end,
                                format!("{indent}{fence}{lang}"),
                            ));
                        }
                        CodeBlockKind::Indented => {
                            // From the start of the first line, indent and all.
                            let start = markdown[..range.start].rfind('\n').map_or(0, |i| i + 1);
                            let block = &markdown[start..range.end];
                            let code: String = block
                                .lines()
                                .map(|l| {
                                    l.strip_prefix("    ").or(l.strip_prefix('\t')).unwrap_or(l)
                                })
                                .map(|l| format!("{l}\n"))
                                .collect();
                            edits.push((start..range.end, format!("```text\n{code}```\n")));
                        }
                    }
                }
                Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                    if let Some(link) = Self::link(dest_url, site_url, url, links) {
                        Self::replace_last(markdown, range.clone(), dest_url, &link, &mut edits);
                    }
                }
                Event::Html(html) | Event::InlineHtml(html) => {
                    let mut html = html.to_string();
                    let mut changed = false;
                    for attr in ["href=\"/", "src=\"/"] {
                        if html.contains(attr) && !html.contains(&format!("{attr}/")) {
                            html = html
                                .replace(attr, &format!("{}{site_url}/", &attr[..attr.len() - 1]));
                            changed = true;
                        }
                    }
                    if changed {
                        edits.push((range.clone(), html));
                    }
                }
                #[cfg(feature = "kramdown")]
                Event::Text(text) if text.starts_with("{:") => {
                    let rest = AttributeList::split(text).1;
                    let len = text.len() - rest.len();
                    if markdown[range.clone()].starts_with("{:") {
                        edits.push((range.start..range.start + len, String::new()));
                    }
                }
                Event::DisplayMath(math) => {
                    let alone = paragraph.is_empty();
                    let (open, close) = if alone {
                        ("\\[", "\\]")
                    } else {
                        ("\\(", "\\)")
                    };
                    let math = Self::entities(&format!("{open}{math}{close}"));
                    edits.push((range.clone(), format!("<span class=\"math\">{math}</span>")));
                }
                _ => {}
            }
            match event {
                Event::Start(Tag::Paragraph) => paragraph.clear(),
                Event::DisplayMath(_) => paragraph.push(Event::Text("math".into())),
                Event::SoftBreak => {}
                Event::Text(t) if t.trim().is_empty() => {}
                e => paragraph.push(e),
            }
        }
        edits.sort_by_key(|(r, _)| std::cmp::Reverse(r.start));
        let mut out = markdown.to_owned();
        let mut last = usize::MAX;
        for (range, text) in edits {
            // Skip anything inside an edit already made.
            if range.end > last {
                continue;
            }
            out.replace_range(range.clone(), &text);
            last = range.start;
        }
        out
    }

    /// Where a link to `dest`, on the page at `url`, goes from the docs: a
    /// page's module, or the site, or nowhere new.
    fn link(
        dest: &str,
        site_url: &str,
        url: &str,
        links: &BTreeMap<String, String>,
    ) -> Option<String> {
        if dest.contains("://")
            || dest.starts_with("mailto:")
            || dest.starts_with('#')
            || dest.starts_with("//")
        {
            return None;
        }
        let (path, anchor) = match dest.split_once('#') {
            Some((p, a)) => (p, Some(a)),
            None => (dest, None),
        };
        let path = if path.starts_with('/') {
            path.to_owned()
        } else {
            let dir = &url[..url.rfind('/').map_or(0, |i| i + 1)];
            format!("{dir}{path}")
        };
        match links.get(&link_key(&path)) {
            Some(module) => Some(match anchor {
                Some(anchor) => format!("{module}#{anchor}"),
                None => module.clone(),
            }),
            None => Some(format!("{site_url}{path}")),
        }
    }

    fn replace_last(
        markdown: &str,
        range: std::ops::Range<usize>,
        from: &str,
        to: &str,
        edits: &mut Vec<(std::ops::Range<usize>, String)>,
    ) {
        if let Some(i) = markdown[range.clone()].rfind(from) {
            let start = range.start + i;
            edits.push((start..start + from.len(), to.to_owned()));
        }
    }

    /// Every ASCII punctuation character as an HTML entity, so Markdown
    /// leaves it alone.
    fn entities(text: &str) -> String {
        text.chars()
            .map(|c| {
                if c.is_ascii_punctuation() {
                    format!("&#{};", c as u32)
                } else {
                    c.to_string()
                }
            })
            .collect()
    }
}

/// Every file under `dir`.
fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if path.is_dir() {
            walk(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}
