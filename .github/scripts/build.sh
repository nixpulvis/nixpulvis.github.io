#!/usr/bin/env bash
# Build the site into _site/. Extra arguments go to `jekyll build`, e.g.
# `--drafts` or `--baseurl /some/path`.
set -euo pipefail

# Compile the fortunes first, so Jekyll publishes the .dat files with them.
make fortunes

bundle exec jekyll build "$@"

# The terminal view, which Jekyll keeps in _site/tui.
make tui

# The site is already built, so stop Pages from running Jekyll over it.
touch _site/.nojekyll
