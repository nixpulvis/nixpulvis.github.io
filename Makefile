# Each year's fortunes, i.e. fortunes/2017 fortunes/2018 ...
FORTUNES = $(filter-out %.dat,$(wildcard fortunes/[0-9][0-9][0-9][0-9]))
FORTUNE_DATS = $(addsuffix .dat,$(FORTUNES))

# Define our non-file based targets.
.PHONY: all clean

# Build the site, and our fortunes.
all: _site fortunes

# Clean up the project directory.
clean:
	rm -rf _site/
	rm -f $(FORTUNE_DATS)

# Build the static site, in all its glory.
_site:
	jekyll build

# Compile the strfile .dat files for use with `fortune`. These aren't
# committed; the build compiles them and publishes them next to the text.
fortunes: $(FORTUNE_DATS)

# General `strfile` transform for inside the fortunes/ directory.
fortunes/%.dat: fortunes/%
	strfile $<

