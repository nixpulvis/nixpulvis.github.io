---
title: Building a Shell - Part 1
layout: rambling
---

A shell is one of the best computer interfaces I've ever used. It's been
*declarative*, before being declarative was cool. It's keyboard centric, making
it fast, and reliable. Sure, a shell isn't the right tool for everything. I'm
not going to try and convince you to do your photo editing by typing
`lighten(0, 0, 100, 100, 50%)`, but it is the right tool for more tasks
than many people think (a topic for another day).

In this post I want to explain what a shell is, and build a very simple one.
The shell we create will not be fast, or even good, but it'll work better than
you might think ~150 lines of Ruby would.

A *terminal* is a graphical application, or device which generally runs a
*shell*. A shell is a special kind of read eval print loop (REPL), that is
primarily concerned with running subprocesses within the operating system.
Another kind of REPL you may have used is the browser's web console.

```
$ date --iso-8601
2018-07-11
$
```


