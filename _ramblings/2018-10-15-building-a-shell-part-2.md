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
Rust                            12            166            580            987
Fish Shell                       2              7              0             43
Bourne Shell                     5              0              0              7
-------------------------------------------------------------------------------
SUM:                            19            173            580           1037
-------------------------------------------------------------------------------
```

Ok, not 50... but again just like last time I think you'll be surprised what
1,000 lines of Rust can get you.

### Rust Basics

To really get what's going on, I *strongly* urge you to download Rust and
compile `oursh` yourself. You can run examples from this post as we go through
them.

Installing Rust is very easy. Visit their [official website][rust] or simply
run `brew install rustup`, or `pacman -S rustup`, etc. Once you have `rustup`
installed, either install `oursh` globally with `cargo install oursh` or, from,
inside the oursh project directory run `cargo build` to compile the code and
produce a working shell. From there (assuming everything works), running the
shell can be done with `cargo` as well.

```sh
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

Before we dive into things, let me give you a taste of something I'm really
excited about. I'm calling them *shebang blocks* after the `#!` symbol you use
to specify the interpreter at the start of a script. These do the same thing
but are embedded in the shell's recursive grammar, just see for yourself.

```
PI={#!/usr/bin/env node; console.log(Math.PI)}
{#!/usr/bin/env ruby; require 'server'; Server.start}&
```

NOTE: This feature is currently experimental, but currently working. You can
try it for yourself by compiling with `cargo run --features=bridge`. The syntax
is also tempararily changed to `{#!/usr/bin/env ruby# puts 1}`, but this is
just a hack.

### `commit` [`a7142d8959c352937bbe2ad0f11f5cb286db3aa8`][a7142d8]

```rust
use std::io::{self, Read, Write};
use std::process::{Command, Output};
use std::str;
// Our shell, for the greater good. Ready and waiting.
fn main() {
    // Standard input file descriptor (0), used for user input from the user
    // of the shell.
    let mut stdin = io::stdin();
    // Standard output file descriptor (1), used to display program output to
    // the user of the shell.
    let mut stdout = io::stdout();
    // STDIN, input buffer, used to `read` text into for further processing.
    // TODO: Full fledged parser will be neato.
    let mut input = [0; 24];
    loop {
        // XXX: Blindly drop the contents of input, again this will be better
        // with a real parser.
        input = [0; 24];
        // Print a boring static prompt.
        print!("oursh> ");
        stdout.lock().flush();
        loop {
            // TODO: Enable raw access to STDIN, so we can read as the user
            // types. By default the underlying file is line buffered. This
            // will allow us to process history, syntax, and more!
            // Read what's avalible to us.
            stdin.read(&mut input).expect("error reading STDIN");
            // Once we've read a complete "program" (§2.10.2) we handle it,
            // until then we keep reading. Once we have a proper parser this
            // wont look like a huge hack.
            let vec: Vec<&[u8]> = input.splitn(2, |b| *b == '\n' as u8).collect();
            match &vec[..] {
                [line, rest] => {
                    let program = str::from_utf8(&line).expect("error reading utf8");
                    let mut output = handle_program(program);
                    stdout.write(&output.stdout).expect("error writing to STDOUT");
                    break
                }
                _ => {},
            }
        }
    }
}
// Obviously very wrong. Most notably this blocks until the command completes.
fn handle_program(program: &str) -> Output {
    Command::new(program)
        .output()
        .expect("error executing program")
}
```

### `extern crate termion;`
##### Cursors
##### History
##### Completion

### `extern crate lalrpop;`
##### Formal Grammar Review
##### Basic Programs
##### POSIX Programs
##### Meta Programs (just `{#}` syntax)

### `extern crate docopt;`

### Testing
##### Unit Tests
##### Integration Tests

### Next Steps
- Custom LALRPOP Lexer
- Proper job runtime, with status, backgrounding, and chained pipes
- Variables, functions, and the program environments






```



















```


[oursh]:     https://nixpulvis.com/oursh/oursh
[part1]:     2018-07-11-building-a-shell-part-1
[a7142d8]:   https://github.com/nixpulvis/oursh/commit/a7142d8
[lalrpop]:   https://github.com/lalrpop/lalrpop
[docopt]:    http://docopt.org/
[termion]:   https://gitlab.redox-os.org/redox-os/termion
[rust]:      https://www.rust-lang.org/en-US
[alacritty]: https://github.com/jwilm/alacritty
