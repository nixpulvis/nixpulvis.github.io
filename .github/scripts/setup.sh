#!/usr/bin/env bash
# Install the system packages the build needs, beyond Ruby and the gems.
# Runs on Debian or Ubuntu, like GitHub's runners.
set -euo pipefail

# LaTeX and dvisvgm for TikZ figures (_plugins/tikz.rb).
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  texlive-latex-base \
  texlive-pictures \
  dvisvgm
