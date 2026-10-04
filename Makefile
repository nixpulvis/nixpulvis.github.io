# The year this all began (or was it...)
EPOCH = 2017
# A list of all the years since, i.e. 2017 2018 2019 ...
YEARS = $(shell seq $(EPOCH) $(shell date +"%Y"))

FORTUNE_DATS = $(addsuffix .dat,$(addprefix fortunes/,$(YEARS)))

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

# Compile all the strfile .dat files for use with `fortune`.
#
# This expects there to be exactly one file named after the each four digit
# year from $EPOCH till now inside the _fortunes/ directory. If it's a new
# year and this is failing, just create the file, add it to `git` and move on
# there will eventually be things to put in there.
#
# TODO: What should be done about the .dat files in git?
fortunes: $(FORTUNE_DATS)

# General `strfile` transform for inside the _fortune directory.
fortunes/%.dat: fortunes/%
	strfile $^

