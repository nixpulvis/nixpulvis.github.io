# https://nixpulvis.com

This is my personal, and semi-professional website. Built by its own Rust
crate, which is also documented with every page as a module, and hosted on
GitHub Pages (a service which I love).

The generator itself is [`cratesite/`](cratesite), a static site generator
for any site that's a crate like this one. This crate is just the site: its
pages, templates and data, `src/config.rs` saying how it's laid out, and
`src/site` for what only this site does.

## Setup

With Rust installed:

```sh
# Build the site into _site/.
cargo site build

# Start up a local webserver on port 4000 (by default), building again
# whenever a file changes.
cargo site serve

# The crate's docs, where every page is a module.
cargo doc --no-deps -p nixpulvis --features site --open
```

Pages are compiled into the generator, so a change to one builds it again.
`cargo site help` lists the options.

TikZ figures (```` ```tikz ```` code blocks) also need `latex` and `dvisvgm`
from TeX Live. Compiled figures are kept in `.site-cache/`.

```sh
sudo apt-get install texlive-latex-base texlive-pictures dvisvgm  # Debian/Ubuntu
brew install texlive dvisvgm                                     # macOS
```

## Usage

The main additions to this site are projects, and ramblings. Creating and
editing these is *very* easy, a Markdown file in `_projects/` or `_ramblings/`.

#### Layouts

Each post should have a layout specified in the YAML front-matter, otherwise
the `default` will be used. There are a number of layouts to choose from:

- `project` - A living document for something I may or may not work on again
- `rambling` - A potentially meaningful set of words
- `note` - A non-updated (except in extreme cases) document
- `article` - Something formally worth publishing

In addition to the base layout types above, there are a few options that may be
specified in the YAML front-matter.

- `draft: <bool>` - Indicate that this post is unfinished (grayed out)
- `hidden: <bool>` - Hide/show the post from index page
- `archived: <bool>` - Move the post from the index page to its archive
- `published: <bool>` - Do/don't generate the page entirely

Additional options may exist for some layouts, and should be documented at
some point.

#### Fortunes

`fortunes/` holds a `fortune` database for each year, like `2026.txt`, with
entries separated by a line holding only `%`. The build lists them on
`/fortunes/` and publishes the compiled `.dat` files next to the text. An empty
entry fails the build.

Add a fortune to this year's file, creating it in a new year, with:

```sh
bin/scribe
```

It opens `$EDITOR`, checks the entry and appends it. To read them locally:

```sh
cargo site build
fortune _site/fortunes
```

#### Markdown

Every page is also published as Markdown next to its HTML, after its template
tags run, so `/ramblings/foo` is readable at `/ramblings/foo.md` with a short
front matter (title, date and so on). `/site.json` lists the nav and every post
with its URL, title, date, collection and draft flag. cratesite publishes the
Markdown and `src/site` the index, for clients that read the site without
scraping HTML.

#### Templates

Pages with template tags, like the index pages, are [askama] templates, as are
the layouts in `templates/`. A page sees `site`, the whole `Site`, and `page`,
itself. Links to the site's own pages are plain root paths, like
`/img/photo.jpg`, and get the base URL added when the site is built under a
sub-path (PR previews).

[askama]: https://askama.rs

## Terminal view

The site can also be read as a terminal app, drawn with [Ratatui] from the
published Markdown and `/site.json` described above. The code is in `src/tui` and `tui/`:

- `src/tui` is the app itself, `nixpulvis::tui`, shared by both clients
- `tui/term` runs it in a real terminal, fetching pages over HTTP
- `tui/web` builds it to WebAssembly for the browser, where the `>_` key
  after the name, or any link ending in `?tui`, switches it on

#### Installing

The [`tui` release] has the terminal client built for Linux and macOS, on x86
and ARM, rebuilt whenever it changes on `master`. Download the one for your
machine and unpack it somewhere in your `PATH`.

Or build it from source with Rust. `--locked` keeps the pinned git
dependencies in `Cargo.lock`:

```sh
cargo install --locked --git https://github.com/nixpulvis/nixpulvis.github.io nixpulvis-tui
```

Either way, run it with `nixpulvis-tui`.

[`tui` release]: https://github.com/nixpulvis/nixpulvis.github.io/releases/tag/tui

#### Running from source

The terminal client reads nixpulvis.com by default. To read a local build
instead, start `cargo site serve` as above and, in another shell:

```sh
cargo run                                     # nixpulvis.com
cargo run -- --url http://localhost:4000      # a local build
cargo run -- /ramblings                       # start on a page
```

`cargo site build` doesn't build the browser view. It's Rust compiled to
WebAssembly, built into `_site/tui` by `make tui`, and the generator leaves
that folder alone when it rebuilds. So build it once and serve as usual. Edits
to pages show up in the view on reload, since it reads their Markdown at
runtime. Only changes under `src/tui/` or `tui/` need another `make tui`.

It needs the `wasm32-unknown-unknown` target and the `wasm-bindgen` CLI at the
version in `Cargo.lock`. If they drift apart, `make tui` stops and says which
version to install:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129
make tui
cargo site serve
```

[Ratatui]: https://ratatui.rs

## Publishing

GitHub Pages serves the `gh-pages` branch. Pushing to `master` runs
`.github/workflows/deploy.yml`, which builds the site and its docs (under `/doc`)
and commits them to the root of `gh-pages`.

Both workflows run `.github/scripts/setup.sh` to install system packages and
`.github/scripts/build.sh` to build, which takes extra `cargo site build` arguments.

Every pull request is built by `.github/workflows/preview.yml` and published to
`https://nixpulvis.com/pr-preview/pr-<number>/`; a comment on the PR links to
it. The preview is removed when the PR is closed.

Changes to the terminal client run `.github/workflows/tui.yml`, which builds it
for each target. On `master` it also replaces the `tui` release with the new
builds.
