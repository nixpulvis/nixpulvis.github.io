# https://nixpulvis.com

This is my personal, and semi-professional website. Created using Jekyll, and
hosted on GitHub Pages (a service which I love).

## Setup

The Gemfile requires the exact Ruby version in `.ruby-version` (currently
4.0.7), so `bundle install` fails on anything else. A version manager like
`rbenv`, `chruby` or `asdf` will pick it up from that file.

```sh
# Only if you don't already have bundler.
gem install bundler

# Install the dependencies.
bundle install

# Start up a local webserver on port 4000 (by default).
bundle exec jekyll serve
```

TikZ figures (`_plugins/tikz.rb`) also need `latex` and `dvisvgm` from TeX Live,
and `make fortunes` needs `strfile` from fortune-mod:

```sh
.github/scripts/setup.sh              # Debian/Ubuntu
brew install texlive dvisvgm fortune  # macOS
```

## Usage

The main additions to this site are projects, and ramblings. Creating and
editing these is *very* easy thanks to Jekyll.

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
make fortunes
fortune fortunes
```

#### Markdown

Every page is also published as Markdown next to its HTML, after Liquid runs,
so `/ramblings/foo` is readable at `/ramblings/foo.md` with a short front
matter (title, date and so on). `/site.json` lists the nav and every post with
its URL, title, date, collection and draft flag. Both come from
`_plugins/markdown_source.rb`, for clients that read the site without scraping
HTML.

#### Links

Write site-internal links, images, and assets through `relative_url` so they
keep working when the site is built under a sub-path (PR previews):

```md
![Photo]({{ '/img/photo.jpg' | relative_url }})
[Ramblings]({{ '/ramblings' | relative_url }})
```

A bare `/img/photo.jpg` still works on nixpulvis.com but breaks in previews.

## Terminal view

The site can also be read as a terminal app, drawn with [Ratatui] from the
published Markdown and `/site.json` described above. The code is in `tui/`:

- `tui/app` is the app itself, shared by both clients
- `tui/term` runs it in a real terminal, fetching pages over HTTP
- `tui/web` builds it to WebAssembly for the browser

#### Running from source

The terminal client reads nixpulvis.com by default. To read a local build
instead, start `jekyll serve` as above and, in another shell:

```sh
cargo run                                     # nixpulvis.com
cargo run -- --url http://localhost:4000      # a local build
cargo run -- /ramblings                       # start on a page
```

[Ratatui]: https://ratatui.rs

## Publishing

GitHub Pages serves the `gh-pages` branch. Pushing to `master` runs
`.github/workflows/deploy.yml`, which builds the site (with our own plugins and
TeX Live, which GitHub's built-in Jekyll wouldn't allow) and commits it to the
root of `gh-pages`.

Both workflows run `.github/scripts/setup.sh` to install system packages and
`.github/scripts/build.sh` to build, which takes extra `jekyll build` arguments.

Every pull request is built by `.github/workflows/preview.yml` and published to
`https://nixpulvis.com/pr-preview/pr-<number>/`; a comment on the PR links to
it. The preview is removed when the PR is closed.
