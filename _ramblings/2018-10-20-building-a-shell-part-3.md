---
title: Building a Shell - Part 3
layout: rambling
draft: true
---

This is part 3 of a series I'm writing on the creation of oursh, a modern POSIX
compatible shell written in Rust. In the [last post][part2] we introduced the
project a bit and went through the grammar. Now we're going to look at a few of
the remaining main components of `oursh`'s initial design.

### Program Arguments

Like any other UNIX program, `oursh` can take CLI flags and arguments. We are
using [`docopt`][docopt] for parsing a standardized help string into an
`ArgvMap` we can use in `oursh`.

Here is the current `docopt` usage string:

```
Usage:
    oursh    [options] [<file> [<arguments>...]]
    oursh -c [options] <command_string> [<command_name> [<arguments>...]]
    oursh -s [options] [<arguments>...]

Options:
    -h --help       Show this screen.
    -v --verbose    Print extra information.
    -a --ast        Print program ASTs.
    -# --alternate  Use alternate program syntax.
```

And, running `cargo run -- --ast -v -c 'echo $0 $1 $2' ls 1 2` will yield this
`ArgvMap`.

```
-#, --alternate => Switch(false)
-a, --ast => Switch(true)
-h, --help => Switch(false)
-v, --verbose => Switch(true)
-c => Switch(true)
-s => Switch(false)
<arguments> => List(["1", "2"])
<command_name> => Plain(Some("ls"))
<command_string> => Plain(Some("echo $0 $1 $2"))
<file> => Plain(None)
```

When you pass a _command_ with `-c` it simply parses and runs the command, no
fancy terminal interactions are needed.

TODO: [Fix and update for #5][#5].

```rust
if let Some(Value::Plain(Some(ref c))) = args.find("<command_string>") {
    parse_and_run(&args)(c)
} else {
    // ...
}
```

When you pass a _shell script_ `<file>` like `oursh hello.sh` it also simply
reads the file, parses and runs the program.

```rust
if let Some(Value::Plain(Some(ref filename))) = args.find("<file>") {
    // Open the given file path by name.
    let mut file = File::open(filename)
        .expect(&format!("error opening file: {}", filename));

    // Fill a string buffer from the file.
    let mut text = String::new();
    file.read_to_string(&mut text)
        .expect("error reading file");

    // Run the program.
    parse_and_run(&args)(&text)
} else {
    // ...
}
```

Similarly, when `STDIN` isn't a TTY device (more of this later), which can
happen for example like `echo date | oursh`, we can simply read from `STDIN`
without `termion` either.

### Raw Mode

However, when `oursh` is launched it enters [raw mode][raw-mode] to further
control the keyboard (and potentially even mouse). This is a cargo "feature"
meaning this can be contitionaly compiled. It's a default feature, so to
disable it, run oursh with `cargo run --no-default-features`.

When running `oursh` interactively we'll use [`termion`][termion] to help
control the terminal. `termion` gives us raw mode, various styles and colors.

([#5][#5])


[#5]: https://github.com/nixpulvis/oursh/issues/5
[#27]: https://github.com/nixpulvis/oursh/issues/27

[part2]: 2018-10-15-building-a-shell-part-2
[docopt]: https://docs.rs/docopt
[termion]: https://docs.rs/termion
