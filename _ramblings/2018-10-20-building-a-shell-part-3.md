---
title: Building a Shell - Part 3
layout: rambling
draft: true
---

In the [last post][part2] we introduced the project a bit and went through the
grammar.  Now we're going to look at a few of the remaining main components of
`oursh`.

Like any other UNIX program, `oursh` can take CLI flags and arguments. We are
using [`docopt`][docopt] for parsing a standardized help string into an
`ArgvMap` we can use in `oursh`.

When you pass a _shell script_ `<file>` like `oursh hello.sh` it simply reads
the file and parses it directly, no fancy terminal interactions are needed.

Similarly, when `STDIN` isn't a TTY device (more of this later), which can
happen for example like `echo date | oursh`, we can simply read from `STDIN`
without `termion` either.

However, when `oursh` is launched it enters [raw mode][raw-mode] to further
control the keyboard (and potentially even mouse).

When running `oursh` interactively we'll use [`termion`][termion] to help
control the terminal. `termion` gives us raw mode, various styles and colors.




[part2]: 2018-10-15-building-a-shell-part-2
[docopt]: https://docs.rs/docopt
[termion]: https://docs.rs/termion
