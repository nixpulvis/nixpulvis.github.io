# https://nixpulvis.com

This is my personal, and semi-professional website. Created using Jekyll, and
hosted on GitHub Pages (a service which I love).

## Setup

Running Jekyll requires a modern version of Ruby. See your operating system's
help for installing that.

```sh
# Only if you don't already have bundler.
gem install bundler

# Install the dependencies.
bundle install

# Start up a local webserver on port 4000 (by default).
jekyll serve
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
- `published: <bool>` - Do/don't generate the page entirely

Additional options may exist for some layouts, and should be documented at
some point.

#### Fortunes

`fortunes/` holds a `fortune` database for each year. Add to this year's with:

```sh
bin/scribe
```

It opens `$EDITOR`, checks the entry, and appends it, creating the year's file
if needed.

#### Links

Write site-internal links, images, and assets through `relative_url` so they
keep working when the site is built under a sub-path (PR previews):

```md
![]({{ '/img/photo.jpg' | relative_url }})
[Ramblings]({{ '/ramblings' | relative_url }})
```

A bare `/img/photo.jpg` still works on nixpulvis.com but breaks in previews.

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
