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

I was originally going to write up a comparison of `docopt` and `clap` in the
hopes that I would be able to choose between them more easily. However, after
thinking through the requirements of the command line argument parser a bit
more, I'm starting to think it might actually be easier to write my own, even
possibly leveraging LALRPOP yet again! That would be really cool actually. So
first, let's look at what exactly we're trying to parse. Starting with the
[`bash` invocation][bash-invoke] description online. Here's the synopsis:

```sh
bash [long-opt] [-ir] [-abefhkmnptuvxdBCDHP] [-o option]
    [-O shopt_option] [argument …]

bash [long-opt] [-abefhkmnptuvxdBCDHP] [-o option]
    [-O shopt_option] -c string [argument …]

bash [long-opt] -s [-abefhkmnptuvxdBCDHP] [-o option]
    [-O shopt_option] [argument …]
```

Let's see... First we notice that `[long-opt]` is always parsed first, before
anything else. That should be easy to make consistent across the three
invocation forms. We can also see that the set of single letter flags are the
same, followed by `[-o option] [-O shopt_option]` each time.

So the general syntax is: first parse long options (e.g. `--init-file
filename`) and the `-ir` and `-s` flags, then parse short `set` options (e.g.
`-x`, an option I use a lot to debug/trace my shell programs) and then the
complete `-+o` options for `set`. Next, we parse options for `shopt` itself
with `-O`. Finally, we parse `-c string` and `argument ...` which are passed
through to the running program in `$@`. 

Here are a few examples to help make the semantics more clear:

```sh
# Run a shell script from file with an argument.
oursh /path/to/script.sh 123
# Equivalent to running `oursh` without arguments.
oursh -s
# Run the shell with a couple arguments.
oursh -s 123 456
# Run a command string with an argument.
oursh -c "echo $@" hello world
```

This shouldn't be hard to write a grammar for...


### Terminal Features

### You `nix`?

What about windows?


[docopt]:       https://nixpulvis.com/oursh/docopt
[gh-issues]:    https://github.com/nixpulvis/oursh/issues
[issue-5]:      https://github.com/nixpulvis/oursh/issues/5
[issue-6]:      https://github.com/nixpulvis/oursh/issues/6
[nix]:          https://nixpulvis.com/oursh/nix/
[oursh]:        https://nixpulvis.com/oursh/oursh/
[part2]:        2018-10-15-building-a-shell-part-2
[pr-64]:        https://github.com/nixpulvis/oursh/pull/64
[pr-68]:        https://github.com/nixpulvis/oursh/pull/68
[repl]:         https://nixpulvis.com/oursh/oursh/repl
[termion]:      https://nixpulvis.com/oursh/termion
[bash-invoke]:  https://www.gnu.org/savannah-checkouts/gnu/bash/manual/bash.html#Invoking-Bash
