#!/usr/bin/env bash
# Install the system packages the build needs, beyond Rust.
# Runs on Debian or Ubuntu, like GitHub's runners.
set -euo pipefail

# LaTeX and dvisvgm for TikZ figures (src/site/tikz.rs).
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  texlive-latex-base \
  texlive-pictures \
  dvisvgm
