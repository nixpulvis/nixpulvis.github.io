# Each year's fortunes, i.e. fortunes/2017 fortunes/2018 ...
FORTUNES = $(filter-out %.dat,$(wildcard fortunes/[0-9][0-9][0-9][0-9]))
FORTUNE_DATS = $(addsuffix .dat,$(FORTUNES))

# Define our non-file based targets.
.PHONY: all clean _site fortunes

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

