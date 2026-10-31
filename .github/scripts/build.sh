#!/usr/bin/env bash
# Build the site into _site/. Extra arguments go to `cargo site build`, e.g.
# `--unpublished` or `--baseurl /some/path`.
set -euo pipefail
cd "$(dirname "$0")/../.."

cargo site build "$@"

# The crate's own docs, with every page as a module, and cratesite's, under
# /doc.
RUSTDOCFLAGS="--html-in-header $PWD/src/doc-header.html" \
  cargo doc --locked --no-deps --lib -p nixpulvis -p cratesite \
  --features nixpulvis/site,nixpulvis/syntax,nixpulvis/native-links,cratesite/build
rm -rf _site/doc
cp -r target/doc _site/doc
echo '<meta http-equiv="refresh" content="0; url=nixpulvis/">' > _site/doc/index.html

# The terminal view, which the generator keeps in _site/tui.
make tui
