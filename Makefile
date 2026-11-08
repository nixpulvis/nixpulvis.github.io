# Each year's fortunes, i.e. fortunes/2017.txt fortunes/2018.txt ...
FORTUNES = $(wildcard fortunes/[0-9][0-9][0-9][0-9].txt)
FORTUNE_DATS = $(addsuffix .dat,$(FORTUNES))

# Define our non-file based targets.
.PHONY: all clean _site fortunes tui

# Build the site, and our fortunes.
all: _site

# Clean up the project directory.
clean:
	rm -rf _site/
	rm -f $(FORTUNE_DATS)

# Build the static site, in all its glory. The fortunes come first so Jekyll
# publishes the .dat files with them.
_site: fortunes
	bundle exec jekyll build

# Compile the strfile .dat files for use with `fortune`. These aren't
# committed. The build compiles them and publishes them next to the text.
fortunes: $(FORTUNE_DATS)

# General `strfile` transform for inside the fortunes/ directory. strfile
# only splits on a bare `%`, so stop on one with trailing whitespace or a
# CRLF ending rather than merge the fortunes around it.
fortunes/%.dat: fortunes/%
	@if grep -Hn '^%[[:space:]]\{1,\}$$' $< >&2; then \
		echo "$<: \`%\` separator has trailing whitespace" >&2; exit 1; \
	fi
	strfile $<

# TODO: Remove this once these crates are consistently supporting what we need.
#
# Build against local checkouts of the git dependencies instead, to try changes
# to them that aren't pushed yet, e.g. `make tui RATMARK=../ratmark`. RATATUI
# is the root of the Ratatui repository. Each one overrides its git source or
# patch in Cargo.toml, and PULLDOWN_KRAM the one in ratmark's.
ifdef RATMARK
LOCAL += --config 'patch."https://github.com/nixpulvis/ratmark".ratmark.path="$(abspath $(RATMARK))"'
endif
ifdef PULLDOWN_KRAM
LOCAL += --config 'patch."https://github.com/nixpulvis/pulldown-kram".pulldown-kram.path="$(abspath $(PULLDOWN_KRAM))"'
endif
ifdef RATZILLA
LOCAL += --config 'patch.crates-io.ratzilla.path="$(abspath $(RATZILLA))"'
endif
ifdef RATATUI
LOCAL += $(foreach crate,ratatui ratatui-core ratatui-crossterm, \
	--config 'patch.crates-io.$(crate).path="$(abspath $(RATATUI))/$(crate)"')
endif

# The terminal view, Rust built to WebAssembly in _site/tui. Needs the
# wasm32-unknown-unknown target and the wasm-bindgen CLI at the version in
# Cargo.lock. Jekyll keeps the folder when it rebuilds (keep_files).
#
# Local checkouts can't be built `--locked`, since Cargo points Cargo.lock at
# them, so it's put back after the build to keep the paths out of commits.
TUI_BUILD = cargo build --release -p nixpulvis-web --target wasm32-unknown-unknown
tui:
ifdef LOCAL
	mkdir -p target && cp Cargo.lock target/Cargo.lock.pinned && \
		trap 'mv -f target/Cargo.lock.pinned Cargo.lock' EXIT && \
		trap 'exit 130' INT TERM && \
		$(TUI_BUILD) $(LOCAL)
else
	$(TUI_BUILD) --locked
endif
	wasm-bindgen --target web --no-typescript --out-dir _site/tui --out-name tui \
		target/wasm32-unknown-unknown/release/nixpulvis_web.wasm
