---
title: Building a Shell - Part 3
layout: rambling
draft: true
---

This is the long awaited part 3 of a series I'm writing on the creation of
[`oursh`][oursh]. Oursh's goals are to be POSIX compatible, while implementing
many new concepts on top of a new shell language that will make using the shell
clean, easy, and safe. In [part 2][part2] we looked at how the Rust project was
shaping up, including the basis of the grammar and one of the new features I'm
most excited about, which will serve as the basis for the multi-language shell.
I also identified a number of issues and since then have organized things in
the project's [issue tracker](gh-issues).

Last time I promised we'd look at the command line more closely, as well as
dive into the guts of a few of the underlying libraries. In many ways this is a
distraction from the more important work of implementing the job runtime, which
is something I've been putting off.  Perhaps, with a bit of momentum again,
part 4 can focus solely on a possible `async` refactor, or just simply fixing
background jobs ([#6][issue-6]).

For now, we'll start with how [`docopt`][docopt] is used to describe the command
line arguments. We'll answer questions around [PR#64][pr-64] and most notably,
we'll decide if `docopt` is still the best crate for our use case.

Next we'll look at how [`termion`][termion] is used to manipulate the terminal
and look at a few `features` and flags that depend on these capabilities. As
part of this, we'll aim to finish [PR#68][pr-68], which will be a somewhat major
refactor of the [`repl`][repl] module. This should pave the way for completing
the main REPL interface ([#5][issue-5]) moving forward.

Finally, we'll take a brief dive into [`nix`][nix], which is mainly used under
the hood of many of our dependencies and OS integrations.


### Command Line Arguments

The goal of the [`docopt`][docopt] project is to build a fully functional
command line parser from the `USAGE` string itself. This is a noble and lofty
goal, which ensures that the `--help` information is kept in sync with the
project, because it defines the arguments themselves.

### Terminal Features

### You `nix`?

What about windows?


[docopt]:     https://nixpulvis.com/oursh/docopt
[gh-issues]:  https://github.com/nixpulvis/oursh/issues
[issue-5]:    https://github.com/nixpulvis/oursh/issues/5
[issue-6]:    https://github.com/nixpulvis/oursh/issues/6
[nix]:        https://nixpulvis.com/oursh/nix/
[oursh]:      https://nixpulvis.com/oursh/oursh/
[part2]:      2018-10-15-building-a-shell-part-2
[pr-64]:      https://github.com/nixpulvis/oursh/pull/64
[pr-68]:      https://github.com/nixpulvis/oursh/pull/68
[repl]:      https://nixpulvis.com/oursh/oursh/repl
[termion]:    https://nixpulvis.com/oursh/termion
