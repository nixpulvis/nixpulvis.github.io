---
title: Building a Shell - Part 1
layout: rambling
draft: true
---

A shell is one of the best computer interfaces I've ever used. It's been
*declarative*, before being declarative was cool. It's keyboard centric, making
it fast, and reliable. Sure, a shell isn't the right tool for everything. I'm
not going to try and convince you to do your photo editing by typing
`lighten(0, 0, 100, 100, 50%)`, but it is the right tool for more tasks
than many people think (a topic for another day).

In this post I want to explain what a shell is, and build a very simple one.
The shell we create will not be fast, or even good, but it'll work better than
you might think ~50 lines of Ruby would.

A *terminal* is a graphical application, or device which generally runs a
*shell*. A shell is a special kind of read eval print loop (REPL), that is
primarily concerned with running subprocesses within the operating system.
Another kind of REPL you may have used is the browser's web console.

```
$ date --iso-8601
2018-07-11
```

### Our Overly Simplistic Ruby Shell

The acronym REPL gives us a very clear path to building a shell. Based on that
we have a nice wishlist of functions to implement. The shell we're building
is very simple, reading only one *line* at a time.

- `read`
- `evaluate`

A much more complete shell might want to read in pieces to update syntax colors
as you type, or provide history completion. We're not building anything that
fancy at the moment however. The `eval` function covers both the Evaluate and
Print steps in a REPL, since we're using the same `STDOUT` and `STDERR`.

```rb
# Print a prompt, then read a line from STDIN or file arguments,
# splitting it into an ARGV-like array.
def read
  print('$ ')
  gets.chomp.split(' ')
end
```

The following implementation of `evaluate` is a copout, because we just pass
the input through to Ruby's implementation of `system` which calls `/bin/sh`.
It does the correct thing however, and is enough to get us off the ground
running.

```rb
# Run the given command in a subprocess (using /bin/sh).
def evaluate(*argv)
  system(*argv)
end
```

With this the full REPL is pretty easy to write.

```rb
# The glorious REPL.
loop do
  evaluate(*read)
end
```

Running this tiny Ruby program will give you a semi-functional shell. Try it
out for yourself! There are however a few critical features missing. For
example, you'll notice `cd` doesn't work. This is where builtins come into
play.

### Extending Our Shell (a little)

Builtins are commands that the shell executes directly, without calling out
to a subprocess. This is exactly what we need for `cd`, since we don't want to
change the working directory of a sub-process, we want to update the shell's
working directory itself.

We'll define a hash of `'command' => lambda { |*args| ... }` to serve as a
lookup table for the builtins.

```rb
# Commands to be executed directly by this shell.
BUILTINS = {}
```

Next we need to update our `evaluate` function to handle our special builtin
commands.

```rb
# Run the given command in a subprocess (using /bin/sh).
def evaluate(*argv)
  if BUILTINS[argv[0]]
    BUILTINS[argv[0]].call(*argv[1..-1])
  else
    system(*argv)
  end
end
```

Finally, we can start defining the needed builtins. We'll start with just the
two most needed commands, `cd` and `exit`, however there are many others which
could be defined at this point. Mainly `.` (or `source`), `alias` and `exec`.
I'll leave those implementations as exercises for the reader.

```rb
# The builtin `cd` for changing the shell's working directory.
BUILTINS['cd'] = lambda do |*args|
  # Change to the home directory by default.
  args << ENV['HOME'] if args.empty?

  # Read the destination path, doing very basic path expansion.
  dest = args.pop.gsub(/~/, ENV['HOME'])

  # Try to change this shell's working directory.
  begin
    Dir.chdir(dest)
  rescue Exception
    puts "no such directory: #{dest}"
  end
end

# The builtin to exit the shell.
BUILTINS['exit'] = lambda do |*args|
  # Exit with a status of 0 by default.
  args << 0 if args.empty?

  # Exit the shell.
  exit args.pop.to_i
end
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
the "real world".
