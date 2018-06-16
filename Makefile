# The year this all began (or was it...)
EPOCH = 2017
# A list of all the years since, i.e. 2017 2018 2019 ...
YEARS = $(shell seq $(EPOCH) $(shell date +"%Y"))

# Compile all the strfile .dat files for use with `fortune`.
#
# This expects there to be exactly one file named after the each four digit
# year from $EPOCH till now inside the _fortunes/ directory. If it's a new
# year and this is failing, just create the file, add it to `git` and move on
# there will eventually be things to put in there.
#
# TODO: What should be done about the .dat files in git?
fortunes: $(addsuffix .dat,$(addprefix _fortunes/,$(YEARS)))

# General `strfile` transform for inside the _fortune directory.
_fortunes/%.dat: _fortunes/%
	strfile $^

