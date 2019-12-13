# nixpulvis.com

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
