//! How the site is laid out, for cratesite. `build.rs` reads this file too,
//! to make the page modules.

use cratesite::{Collection, Config};

/// The site's name, collections, and which files are part of it.
pub const CONFIG: Config = Config {
    title: "nixpulvis",
    url: "https://nixpulvis.com",
    description: "Projects, ramblings and notes by Nathan Lilienthal.",
    repository: Some("https://github.com/nixpulvis/nixpulvis.github.io"),
    // In the order `site.json` lists their pages.
    collections: &[
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
    ],
    exclude: &[
        "Cargo.lock",
        "Cargo.toml",
        "Makefile",
        "README.md",
        "TODO-espresso",
        "askama.toml",
        "bin",
        "build.rs",
        "cratesite",
        "src",
        "templates",
        "tui",
    ],
    unlisted: &["MTk3ODkK"],
    // The terminal view and the crate's docs, built separately.
    keep: &["tui", "doc"],
    tikz_preamble: "\\usepackage{tikz-cd}\n\\usetikzlibrary{arrows.meta,graphs,positioning,automata}\n",
};
