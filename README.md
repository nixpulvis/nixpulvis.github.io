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

#### Projects

To create a project, make a new file in the `_projects` directory with the name
of the file, and extension `.md`. The only required YAML front matter attribute
is `layout: project`. Otherwise, you may set `draft: true` to make the project
appear greyed out to viewers, or `published: false` to completely hide it.

#### Ramblings

To create a rambling simply make a new file in the `_ramblings/` directory, and
name it with the format `YYYY-MM-DD-the-title.md`. There is only one required
YAML front matter attribute, `layout: rambling`. Otherwise, you may set `draft:
true` to make the post appear greyed out to viewers, or `published: false` to
completely hide it.
