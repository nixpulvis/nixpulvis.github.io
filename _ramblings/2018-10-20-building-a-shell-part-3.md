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
    oursh [options] [<file> [<arguments>...]]
    oursh -c [options] <command_string> [<command_name> [<arguments>...]]
        Read commands from the `command_string` operand. Set the value of
        special parameter 0 (see Section 2.5.2, Special Parameters) from the
        value of the `command_name` operand and the positional parameters
        ($1, $2, and so on) in sequence from the remaining `arguments` operands.
        No commands shall be read from the standard input.
    oursh -s [options] [<arguments>...]
        Read commands from the standard input.

Options:
    -i  Specify that the shell is interactive; see below. An implementation may
        treat specifying the −i option as an error if the real user ID of the
        calling process does not equal the effective user ID or if the real
        group ID does not equal the effective group ID.

    If there are no operands and the −c option is not specified, the −s option
    shallbe assumed.

    If the −i option is present, or if there are no operands and the shell's
    standard input and standard error are attached to a terminal, the shell is
    considered to be interactive.

    --alternate        Use alternate program syntax.
    --ast              Print program ASTs.
    --debug
    --debugger
    --dump-po-strings
    --dump-strings
    --help
    --init-file
    --login
    --noediting
    --noprofile
    --norc
    --posix
    --pretty-print
    --rcfile
    --restricted
    --verbose
    --version

Operands:
    −   A single <hyphen> shall be treated as the first operand and then
        ignored. If both '−' and '--' are given as arguments, or if other
        operands precede the single <hyphen>, the results are undefined.

    `arguments`  The positional parameters ($1, $2, and so on) shall be set to
                 arguments, if any.

    `command_file`
        The pathname of a file containing commands. If the pathname contains
        one or more <slash> characters, the implementation attempts to read
        that file; the file need not be executable. If the pathname does
        not contain a <slash> character:

        *  The implementation shall attempt to read that file from the current
           working directory; the file need not be executable.

        *  If the file is not in the current working directory, the implementa‐
           tion may perform a search for an executable file using the value of
           PATH, as described in Section 2.9.1.1, Command Search and Execution.

        Special parameter 0 (see Section 2.5.2, Special Parameters) shall  be
        set to  the value of command_file.  If sh is called using a synopsis
        form that omits command_file, special parameter 0 shall be set to the
        value  of  the first  argument passed to sh from its parent (for
        example, argv[0] for a C program), which is normally a pathname used to
        execute the sh utility.

    `command_name`
        A string assigned to special parameter 0 when executing the commands in
        command_string. If command_name is not specified, special parameter 0
        shall be set to the value of the first argument passed to sh from its
        parent (for example, argv[0] for a C program), which is normally a
        pathname used to execute the sh utility.

    `command_string`
        A string that shall be interpreted by the shell as one or more
        commands, as if the string were the argument to the system() function
        defined in the System Interfaces volume of POSIX.1‐2008. If the
        command_string operand is an empty string, sh shall exit with a zero
        exit status.
```

TODO: `man set` and it's tracking GH issue.
    `−a,  −b, −C, −e, −f, −m, −n, −o option, −u, −v, and −x` options are
    described as part of the set utility in Section 2.14, Special  Built-In
    Utilities.

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
