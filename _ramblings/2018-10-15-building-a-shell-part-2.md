---
title: Building a Shell - Part 2
layout: rambling
draft: true
---

This is part 2 of a series I'll be writing on the creation of [`oursh`][oursh],
a modern POSIX compatible shell written in Rust. If you haven't read [Part
1][part1], and want a breif overview of shell terminology and functionality I
recomend reading it, otherwise this will be the first post on the Rust project.

In this post I want to dive into the skeleton of the project so far.  Last time
we made a shell in 58 lines of Ruby. I then started the `oursh` project in
(amazingly) the [same number of lines][a7142d8]. Now the project has grown.
I've added [LALRPOP][lalrpop] for parsing the POSIX grammar, it has a separate
parser known as [docopt][docopt] for handling arguments, and it has something
starting to resemble a `readline(3)` like library using [termion][termion]. With
everything, and the growing test suite this is the output of `cloc`.

```
-------------------------------------------------------------------------------
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            15            268            677           1747
Bourne Shell                     2              0              0              3
-------------------------------------------------------------------------------
SUM:                            17            268            677           1750
-------------------------------------------------------------------------------
```

Ok, not 50... but again just like last time I think you'll be surprised what
2,000 lines of Rust can get you. For example, these shell programs all
currently work.

```sh
ls -la
cat README.md | wc
git add -u
true || false && true && echo reached
! true && echo not reached
{ sleep 1; date; }&
```

### Build it to Believe

Before we begin, in order to really get what's going on, I **strongly** urge
you to download Rust and compile `oursh` yourself. You can run examples from
this post as we go through them. Otherwise, just use `bash` and skip this
section.

First clone the repository from GitHub into your local directory.

```sh
git clone https://github.com/nixpulvis/oursh
```

Then install Rust by either, visiting their [official website][rust] or simply
running `brew install rustup`, `pacman -S rustup`, etc. Once you have
`rustup` installed, from inside the oursh project directory `cargo` has many
commands at your disposal.

```unix
cargo build  # Just compile
cargo run    # Compile and run oursh
cargo test   # Compile and run the test suite
```

If you're lazy (like me), here's a script to copy-paste. This, in a way is the
power of UNIX.

```sh
# macOS
brew install rustup
# Arch Linux
pacman -S rustup

git clone https://github.com/nixpulvis/oursh
cd oursh
cargo run
```

Depending on what version of `oursh` you have checked out the shell's prompt
may look different, and there are a few compilation features, but generally
speaking you'll get a shell that looks like this:

```
oursh-0.3$
```

Play around with it, it's a real shell. If you want even more Rust between you
and your OS I recommend [Alacritty][alacritty], a GPU-accelerated terminal
also written in Rust.

### Something New, Before a lot of Old

Before we dive into the dense topic of language, let me give you a taste of
something I'm really excited about, which is only possible with some knowledge
of the following material. I'm calling them _[shebang][shebang] blocks_ after
the `#!` symbol you can use to specify an interpreter at the start of a script
file. These do the same thing but are embedded in the shell's recursive
grammar, just see for yourself.

Print my cat's name (π) from Ruby.

```
{#!/usr/bin/env ruby; puts Math::PI}
```

Same thing, just syntactic sugar.
```
{#!ruby; puts Math::PI}
{#!node; console.log(Math.PI)}
{#!racket; #lang racket (println pi)}
```

This feature is currently experimental, but working. You can try it for
yourself by compiling with `cargo run --features=bridge`. These more advanced
commands will also Just Work once I get to implementing background programs
([#6][issue-6]) and environments/variables ([#27][issue-27]).

```
# Save π to the ENV var $PI.
PI={#!node; console.log(Math.PI)}; echo $PI

# Start a Ruby server in the background.
{#!ruby; require 'server'; Server.start}&
```

OK, now back to your regularly scheduled series on programming languages,
grammar, POSIX shell scripts and Rust.

### Shell Language

A _script_ is, in general, some text which is _interpreted_ by a program on a
computer. A _shell_, like the one we're building (or `bash`, `sh`, ...), is one
such interpreter. You can pass a shell script to it and it will correctly
execute the program. A POSIX shell program is typically just known as "shell
script", though other types of shell scripts exist; like [fish][fish] scripts.
For convenience, we'll simply refer to a POSIX compatible shell as `sh`.

The first implementation of `sh` was by Ken Thompson around 1971, which is why
you may also hear this refered to as a Thompson shell. The next major
development was in 1979 with the Bourne shell, also known as `sh`, which was a
rewrite by Stephen Bourne at Bell Labs. The most commmon shell these days is
`bash`, which stands for Bourne-Again shell.

Now finally, for some Rust code...

##### [`use oursh::program::basic::Program;`][basic::Program]

This is the most basic implementation of a `Program` within oursh. The only
thing `BasicProgram` is able to do is run commands like `ls` or `git status`,
but nothing more complex like `cat $1 | wc -c`, or new like `{#!ruby; p 1}`.

This implementation doesn't use an fancy parsers, and simply reads a full line
and splits it by spaces, passing the result directly through to `exec` on
`run`.

```sh
# Run oursh using alternate (basic currently) program parsing.
cargo run -- -#
```


##### Formal Grammar Review

A formal _grammar_ is a set of rules for creating _non-terminals_ from
_terminals_, where the terminals represent the set of characters (or tokens) in
the _language_. A set of tokens in a language is known as an _alphabet_. A
non-terminal represents some production based on underlying non-terminals and
terminals, in this way allowing for recursive definitions.

##### [`extern crate lalrpop;`][lalrpop]

LALRPOP is a **parser generator**, which is essentially a framework for
creating parsers for some kinds of languages. In this case, LALRPOP runs in a
build phase before the compiler and generates a valid Rust parser module. The
parser is defined to correctly parse the grammar defined in a corresponding
`*.lalrpop` file. To understand what this means we need to understand what we
mean by _parser_.

The AST is in our case mainly one `struct` and one heavily recursive `enum`.
The main goal of the AST is to preserve the needed information from the text
to easily map to the execution semantics. For example consider this example
from grade school: `1 + 2 * 3`. We typically learn a rule called PEMDAS to
remember the order of _precedence_ so we correctly read the previous
expression as `1 + (2 * 3)`.  Similarly, we have `true && false || true`, which
could be parsed one of two ways:

```sh
(false && true) || true  #=> true
false && (true || true)  #=> false
```

A parser's job is to take in a stream of tokens (things like `#`,`"`, `}`,
`WORD`, etc) and produce an AST. Tokens are the terminals, while the grammar
description `*.lalrpop` file defines the non-terminal rules. Before we look at
the grammar rules, let's look at our AST definition.

##### [`use oursh::program::posix::ast::Program;`][posix::ast::Program]

Internally, `posix::ast::Program` uses the parser module generated
automatically in it's `parse` function.

```rs
let program = Program::parse(b"! true || false" as &[u8])?;
println!("{:#?}", program);
-----
Program(
    [
        Or(
            And(
                Simple(
                    [
                        Word(
                            "true"
                        )
                    ]
                ),
                Simple(
                    [
                        Word(
                            "false"
                        )
                    ]
                )
            ),
            Simple(
                [
                    Word(
                        "true"
                    )
                ]
            )
        )
    ]
)
```

The only major piece left to explain is the [grammar
description][posix.lalrpop] file. There [are][lalrpop-full-expr] a
[lot][lalrpop-lexer] of [details][lalrpop-macros] to go over with
the LALRPOP parser generator, but luckily it's simple enough to understand the
basics.

These are the needed rules inside `posix.lalrpop` to parse the above AST.
```rs
Commands: ast::Command = {
    // ...
    <cs: Commands> "&&" <p: Pipeline> => {
        ast::Command::And(box cs, box p)
    },
    <cs: Commands> "||" <p: Pipeline> => {
        ast::Command::Or(box cs, box p)
    },
    // ...
    Pipeline => <>,
}

// ...

Simple: ast::Command = {
    "WORD"+ => ast::Command::Simple(<>.iter().map(|w| {
        ast::Word(w.to_string())
    }).collect())
}
```

TODO: Explain a bit about the recursion going on here.

TODO: Explain the lexer (a bit) and how a WORD is generated.

<details>
  <summary>
    If parsers are your thing, you may also be interested in a
    parser-combinator I wrote for students at NEU's introduction computer
    science class. It's writen in Racket, and comes with a fully baked JSON
    parser.
  </summary>
  <p>
    <a href="https://github.com/nixpulvis/parser-combinator">
      <code>(require parser-combinator/json)</code>
    </a>
  </p>
  <p>
    TODO
  </p>
</details>

### Testing

Just a quick note on testing, because it's an important part of this project.
Rust's macro system does a great job allowing developers to write clean and
fast tests. Here are a few integration tests from this project's suite.

```rs
assert_oursh!("head README.md -n 1", "# oursh\n");
assert_oursh!("false && echo 1", "");
assert_oursh!("false || echo 1", "1\n");
assert_oursh!("{ echo pi; echo e; }", "pi\ne\n");
assert_oursh!("{#!ruby; puts 1}", "1\n");
```

### Next Steps

- Add a proper job runtime, with status, backgrounding, and chained pipes. I'm
  still not 100% sure how to best implement this.
- Programs need variables, functions, and the program environments.

TODO: Short description of the next post.

[a7142d8]:       https://github.com/nixpulvis/oursh/commit/a7142d8
[alacritty]:     https://github.com/jwilm/alacritty
[docopt]:        http://docopt.org/
[explainshell]:  https://explainshell.com
[fish]:          https://fishshell.com
[issue-27]:      https://github.com/nixpulvis/oursh/issues/27
[issue-6]:       https://github.com/nixpulvis/oursh/issues/6
[oursh]:         https://nixpulvis.com/oursh/oursh
[part1]:         2018-07-11-building-a-shell-part-1
[rust]:          https://www.rust-lang.org/en-US
[shebang]:       https://en.wikipedia.org/wiki/Shebang_(Unix)
[termion]:       https://gitlab.redox-os.org/redox-os/termion

[lalrpop-full-expr]: http://lalrpop.github.io/lalrpop/tutorial/005_full_expressions.html
[lalrpop-lexer]:     http://lalrpop.github.io/lalrpop/tutorial/004_controlling_lexer.html
[lalrpop-macros]:    http://lalrpop.github.io/lalrpop/tutorial/007_macros.html
[posix.lalrpop]:     https://github.com/nixpulvis/oursh/blob/master/src/program/posix.lalrpop

[basic::Program]:      https://nixpulvis.com/oursh/oursh/program/basic/struct.Program
[posix::ast::Program]: https://nixpulvis.com/oursh/oursh/program/posix/ast/struct.Program
[lalrpop]:             https://docs.rs/lalrpop/0.16.0/lalrpop
