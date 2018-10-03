---
title: Building a Shell - Part 1
layout: rambling
---

A shell is one of the best computer interfaces I've ever used.
Twas *declarative*, before being declarative was cool. It's
*keyboard centric*, making it fast, and reliable to use. Sure, a
shell isn't the right tool for everything. I'm not going to try
and convince you to do your photo editing by typing `lighten(0, 0,
100, 100, 50%)`. A shell **is** the right tool for more tasks than many
people think (a topic for another day).

In this post I want to explain what a shell is, and build a very simple one.
The shell we create will not be fast, or even good, but it'll work better than
you might think ~50 lines of Ruby would.

### Shell Basics

A *terminal* is a graphical application, or device which generally runs a
*shell*. A shell is a special kind of read eval print loop (REPL), that is
primarily concerned with running subprocesses within the operating system.

A shell generally prints a *prompt* (`$` for example) at the start of your
terminal's line before waiting for a command. Using the shell is easy, just
type your command and hit return. The shell will execute the command, and after
everything's done return you back to another prompt.

```
$ echo "Hello World"
Hello World
$ date --iso-8601
2018-07-11
...
```

You may have heard the saying that everything in UNIX is a file. Well that's
kinda true, but here we're not reading and writing to a file that's actually on
the filesystem, we're interacting with `STDIN`, `STDOUT` and `STDERR`. You
should think of these like virtual files, which a terminal gives you access to.
`STDIN` is the file descriptor for reading the text you type, while `STDOUT`
and `STDERR` are both displayed on the terminal when written to.

With the basics under our belt, let's get started building a very simple, but
functional shell. We'll use Ruby for this, because it'll make things very easy,
as you'll soon see. If you're on a Mac you've already got Ruby installed,
otherwise you can [install][install] it easily on many systems. We'll be
working on a program aptly named `shell.rb`, which you can run with `ruby
shell.rb` from the terminal.

### Our Overly Simplistic Ruby Shell

The acronym REPL gives us a starting point for designing our shell, with a
wishlist of functions to implement. We can get away without defining `print` or
`loop` since evaluation will print itself, and [`loop`][loop] is built into
Ruby.

- `read`
- `evaluate`

We'll start by implementing the `read` function. This function will block until
an entire line is read using [`gets`][gets]. A much more powerful shell might
read in pieces to update syntax highlighting as you type, or provide various
kinds of completion. However, we're not building anything that fancy at the
moment.

```rb
# Print a prompt, then read a line from STDIN.
def read
  print('$ ')

  # Try to read a line, exiting if there's no more lines.
  line = STDIN.gets
  if line
    line.chomp.split(' ')
  else
    exit
  end
end
```

The following implementation of `evaluate` is a copout, because we just pass
the input through to Ruby's implementation of [`system`][system] which calls
`/bin/sh` under the hood. It does the correct thing however, and is enough to
get us off the ground running.

```rb
# Run the given command in a subprocess.
def evaluate(argv)
  success = system(*argv)
  unless success
    puts("unknown command '#{argv[0]}'")
  end
end
```

Now that we've defined both `read` and `evaluate`, the full blown REPL is a
simple one liner.

```rb
# The glorious REPL itself.
loop { evaluate(read) }
```

Running this tiny Ruby program will give you a semi-functional shell. Take a
moment and try it out for yourself! There are however, a few critical features
missing, for example you'll notice `cd` doesn't work. This is where builtins
come into play.

### Extending Our Shell (a little)

Builtins are commands that the shell executes directly, without calling out
to a subprocess. This is exactly what we need for `cd`, since we don't want to
change the working directory of a sub-process, we want to update the shell's
working directory itself.

We'll define a hash of `'command' => lambda { |args| ... }` to serve as a
lookup table for the builtins. The [`lambda`][proc] here is a function which
performs the task of the given `command`.

```rb
# Commands to be executed directly by this shell.
BUILTINS = {}
```

Next we need to update our `evaluate` function to handle our special builtin
commands.

```rb
# Run the given command in a subprocess.
def evaluate(argv)
  if BUILTINS[argv[0]]
    BUILTINS[argv[0]].call(argv[1..-1])
  else
    success = system(*argv)
    unless success
      puts("unknown command '#{argv[0]}'")
    end
  end
end
```

Finally, we can start defining the needed builtins. We'll start with the two
most needed commands, `cd` and `exit`, however there are many others which
could be defined at this point. Mainly `.` (or `source`), `alias` and `exec`.
I'll leave those implementations as exercises for the reader.

```rb
# The builtin `cd` for changing the shell's working directory.
BUILTINS['cd'] = lambda do |args|
  # Change to the home directory by default.
  args << ENV['HOME'] if args.empty?

  # Read the destination path, doing very basic path expansion.
  dest = args.pop.gsub(/~/, ENV['HOME'])

  # Try to change this shell's working directory.
  begin
    Dir.chdir(dest)
  rescue Exception
    puts("no such directory: #{dest}")
  end
end
```

```rb
# The builtin to exit the shell.
BUILTINS['exit'] = lambda do |args|
  # Exit with a status of 0 by default.
  args << 0 if args.empty?

  # Exit the shell.
  exit args.pop.to_i
end
```

There's another small issue the observant reader may have noticed. If you enter
<kbd>ctrl-c</kbd> our shell process will exit with a nasty Ruby error. This is
because by default that sends a `SIGINT` *signal*. Signals are another way UNIX
processes can communicate with each other. In this case we'll [`trap`][trap]
the `SIGINT` signal, and simply return a new prompt.

```rb
# Don't exit on `SIGINT`, instead simply return a new prompt.
trap('INT') { print("\n$ ") }
```

### The Code

If you'd rather download the shell instead of copying / writing it based on the
above snippits, I can't fault you (too much). The pieces are glued together
for your pleasure as a
[Gist](https://gist.github.com/nixpulvis/59d4f60db401f4b3fba6d6781063c7f5).

### Conclusion

That's it, you've now got a working shell. In the future, we'll dive into both
the `read` and `evaluate` functions in more depth. Both functions need a lot
of work before we're anywhere near having a standalone POSIX shell ready for
the "real world". There's also many other topics worth exploring, like `ENV`
variables, job control, user groups, glob syntax, locale, and much more.

[install]: https://www.ruby-lang.org/en/documentation/installation/
[loop]:    https://ruby-doc.org/core-2.2.0/Kernel.html#method-i-loop
[gets]:    https://ruby-doc.org/core-2.2.0/Kernel.html#method-i-gets
[system]:  https://ruby-doc.org/core-2.2.0/Kernel.html#method-i-system
[proc]:    https://ruby-doc.org/core-2.2.0/Proc.html
[trap]:    https://ruby-doc.org/core-2.2.0/Kernel.html#method-i-trap
