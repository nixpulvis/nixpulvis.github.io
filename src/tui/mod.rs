//! Browse nixpulvis.com as a terminal app.
//!
//! The app reads the site's published Markdown and `site.json`, renders
//! pages with ratmark, and draws them with Ratatui. It never does I/O itself.
//! Methods return [`Effect`]s for the host to carry out, which keeps the
//! same code working in a real terminal and in the browser.
//!
//! ```no_run
//! # use nixpulvis::tui::{App, Host};
//! let mut app = App::new(Host::Terminal, "https://nixpulvis.com", "/");
//! let effects = app.start("/ramblings");
//! // Carry out the effects, pass what they fetch back to the app, and
//! // draw it and pass it input until it sends `Effect::Quit`.
//! ```

mod app;
mod colors;
mod link;
mod markup;
mod page;
mod site;

pub use app::App;

/// Where the app runs, which decides who keeps the history, and what `q`
/// does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Host {
    /// A terminal, where the app is all there is. It keeps its own history,
    /// and `q` quits.
    Terminal,
    /// A browser, with the app drawn over the site's own page. The browser
    /// loads the site's other pages, like it would any link, and so it keeps
    /// the history. `q` leaves the app for the page under it.
    Browser,
}

/// Something for the host to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Fetch `site.json` from this path and pass it to [`App::site_loaded`].
    FetchSite(String),
    /// Fetch a page's Markdown from `source` and pass it to
    /// [`App::page_loaded`] along with `path`.
    FetchPage { path: String, source: String },
    /// Open a link that isn't part of the site.
    Open(String),
    /// Load the site's page at this path, with any `#` section, and start
    /// the app on it again. Only sent in a [`Host::Browser`].
    Navigate(String),
    /// Go back to the page before this one. Only sent in a
    /// [`Host::Browser`].
    Back,
    /// Stop, or in a [`Host::Browser`], leave the app for the page under it.
    Quit,
}

/// Input, already translated from whatever the host's events look like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Tab,
    BackTab,
    Enter,
    Back,
    Esc,
    Char(char),
}
