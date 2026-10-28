---
title: Building a Shell - Part 3
date: '2023-09-13'
draft: true
collection: ramblings
---

This is the long awaited part 3 of a series I'm writing on the creation of
[`oursh`][oursh]. Oursh's goals are to be POSIX compatible, while implementing
many new concepts on top of a new shell language that will make using the shell
clean, easy, and safe. In [part 2][part2] we looked at how the Rust project was
shaping up, including the basis of the grammar and one of the new features I'm
most excited about, which will serve as the basis for the multi-language shell.
I also identified a number of issues and since then have organized things in
the project's [issue tracker](gh-issues).

[oursh]:        https://nixpulvis.com/oursh/oursh/
[part2]:        2018-10-15-building-a-shell-part-2