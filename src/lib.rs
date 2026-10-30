//! [nixpulvis.com](https://nixpulvis.com), as a Rust crate.
//!
//! [`pages`] is the site itself. Every page is a Markdown file, and also a
//! module, documented with the page, so those docs are another way to read
//! the site. [`pages::ramblings`] holds the ramblings, [`pages::projects`]
//! the projects, and so on, laid out like the site's URLs. Each page
//! module's `PAGE` is the [`Source`] it was made from.
//!
//! The rest of the crate builds the site and browses it:
//!
//! - [`site`] builds the site with [cratesite], behind the `site` feature.
//!   It reads the pages compiled into this crate and writes `_site`, the
//!   HTML GitHub Pages serves, along with each page's Markdown and
//!   `site.json`. Run it with `cargo site build`, or `cargo site serve`
//!   while writing. [`CONFIG`] lays the site out.
//! - [`tui`] is the terminal app, behind the `tui` feature, which the
//!   `nixpulvis-tui` and `nixpulvis-web` crates host in a terminal and in
//!   the browser. It reads the published Markdown over HTTP, so an
//!   installed copy stays current.
//! - [`index`] is `site.json`, which the generator writes and the terminal
//!   app reads.

mod config;
pub mod index;
#[cfg(feature = "site")]
pub mod site;
#[cfg(feature = "tui")]
pub mod tui;

pub use config::CONFIG;
pub use cratesite::Source;

/// The site's home page, and every other page under it, made modules by
/// `build.rs` from the Markdown files.
///
#[doc = include_str!(concat!(env!("OUT_DIR"), "/doc/index.md"))]
#[allow(
    clippy::doc_overindented_list_items,
    clippy::doc_lazy_continuation,
    clippy::doc_markdown,
    rustdoc::all
)]
pub mod pages {
    include!(concat!(env!("OUT_DIR"), "/pages.rs"));
}
