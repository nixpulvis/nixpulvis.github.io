//! [nixpulvis.com](https://nixpulvis.com), as a Rust crate.
//!
//! The site's pages are Markdown files, and every one of them is also a
//! module of this crate, documented with the page itself. So these docs are
//! another way to read the site. [`ramblings`] holds the ramblings,
//! [`projects`] the projects, and so on, laid out like the site's URLs. Each
//! page module's `PAGE` is the [`Source`] it was made from.
//!
//! The same crate builds the site and browses it:
//!
//! - [`site`] is the static site generator, behind the `site` feature. It
//!   reads the pages compiled into this crate and writes `_site`, the HTML
//!   GitHub Pages serves, along with each page's Markdown and `site.json`.
//!   Run it with `cargo site build`, or `cargo site serve` while writing.
//! - [`tui`] is the terminal app, behind the `tui` feature, which the
//!   `nixpulvis-tui` and `nixpulvis-web` crates host in a terminal and in
//!   the browser. It reads the published Markdown over HTTP, so an
//!   installed copy stays current.
//! - [`index`] is `site.json`, which the generator writes and the terminal
//!   app reads.
//!
//! What follows is the site's home page.
#![doc = include_str!(concat!(env!("OUT_DIR"), "/doc/crate.md"))]

pub mod collection;
pub mod index;
#[cfg(feature = "site")]
pub mod site;
#[cfg(feature = "tui")]
pub mod tui;

/// A page's source file, compiled into the crate.
#[derive(Debug)]
pub struct Source {
    /// The file's path from the top of the repository, like
    /// `_ramblings/2018-02-05-hello-world.md`.
    pub path: &'static str,
    /// The whole file, front matter and all.
    pub text: &'static str,
}

// One module for every page, made by `build.rs` from the Markdown files.
include!(concat!(env!("OUT_DIR"), "/pages.rs"));
