//! A static site generator where the site is a crate.
//!
//! A site's pages are Markdown files in the crate, next to its `Cargo.toml`.
//! Its build script hands them to [`BuildScript`], which compiles every page
//! into the crate as a module, documented with the page itself. So the
//! crate's docs are the site too, and links between pages are links between
//! modules. The crate then builds its HTML from the same pages with
//! [`Theme::build`], wrapping them in layouts the site writes itself, as
//! askama templates or anything else that makes a string.
//!
//! # A site
//!
//! ```text
//! my-site/
//! ├── Cargo.toml       cratesite with `build`, and `generate` for the binary
//! ├── build.rs         BuildScript::new(&CONFIG).run()
//! ├── src/config.rs    pub const CONFIG: Config = ...
//! ├── src/lib.rs       include!(concat!(env!("OUT_DIR"), "/pages.rs"))
//! ├── src/bin/site.rs  a Theme, and Cli to build or serve it
//! ├── index.md         the home page, which is the crate's own docs
//! ├── about.md         about.html, and the module `about`
//! └── _posts/          a collection, like /posts/:name
//! ```
//!
//! A page is a file starting with front matter between `---` lines. Its
//! Markdown is rendered with pulldown-cmark, code is highlighted with
//! syntect, and ```` ```tikz ```` code is drawn to SVG. The `kramdown`
//! feature reads Markdown written for kramdown too. A page with template
//! tags, `{{ }}` or `{% %}`, also gets an askama template, rendered first,
//! with `site` and `page` to draw from.
//!
//! Every page's Markdown is also published next to its HTML, like
//! `/about.md`, wrapped in its layout, for anyone who'd rather read that.
//!
//! # Features
//!
//! - `build`, for the build script's [`BuildScript`].
//! - `generate`, for [`Site`], [`Theme`] and the rest of the generator.
//! - `kramdown`, to read Markdown written for kramdown.

mod config;
#[cfg(any(feature = "build", feature = "generate"))]
mod front;
pub use config::{Collection, Config, Name};

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "build")]
pub use build::BuildScript;

#[cfg(feature = "generate")]
mod cli;
#[cfg(feature = "generate")]
mod highlight;
#[cfg(feature = "generate")]
mod markdown;
#[cfg(feature = "generate")]
mod page;
#[cfg(feature = "generate")]
mod publish;
#[cfg(feature = "generate")]
mod serve;
#[cfg(feature = "generate")]
mod site;
#[cfg(feature = "generate")]
mod text;
#[cfg(feature = "generate")]
mod tikz;

#[cfg(feature = "generate")]
pub use cli::{Cli, Command};
#[cfg(feature = "generate")]
pub use highlight::Highlighter;
#[cfg(feature = "generate")]
pub use markdown::Markdown;
#[cfg(feature = "generate")]
pub use page::{FrontMatter, Page};
#[cfg(feature = "generate")]
pub use publish::Yaml;
#[cfg(feature = "generate")]
pub use serve::Server;
#[cfg(feature = "generate")]
pub use site::{Options, Site, Theme};
#[cfg(feature = "generate")]
pub use text::Text;
#[cfg(feature = "generate")]
pub use tikz::Tikz;

/// A page's source file, compiled into the site's crate.
#[derive(Debug)]
pub struct Source {
    /// The file's path from the top of the crate, like
    /// `_posts/2018-02-05-hello-world.md`.
    pub path: &'static str,
    /// The whole file, front matter and all.
    pub text: &'static str,
}
