---
layout: project
status: complete
draft: true
references:
- https://nixpulvis.com/parser-combinator/
- https://github.com/nixpulvis/parser-combinator
- https://github.com/nixpulvis/parser-combinator/blob/master/lab8.pdf
---

<!-- Outline only. Target is about 400 to 700 words in total. -->

Lexers and Parsers are at the core of every language. The main goal is to turn
a stream of text into something structured and spesific to the language at
hand. Here I show how the parser library I wrote works by combining small
parsers into larger ones with only a few kinds of combinators.

<!-- Intro: tighten the paragraph above and fix "spesific". Point at the
     manual for everything beyond this page. -->

### Where It Came From

<!--
- Third of four years TAing CS 2500 (Fall 2014).
- Wrote it as the JSON teachpack for Lab 8. Core written in about three days,
  first commit 2014-11-03.
- Two reasons: curiosity about writing parsers, and giving the faster students
  in lab something interesting to read. The lab handout says parsing "is not
  trivial" and invites them to read the teachpack source.
- The lab was used for a few more years after that.
- Became a raco package in 2018, then a cleanup and a Scribble manual in 2026.
-->

### The Idea

<!--
- Two steps: a parser turns a string into an AST, then an ordinary recursive
  function turns the AST into data, following the shape of the grammar (the
  design recipe).
- One example: the NUMBER grammar from the manual's Quick Start, about 15
  lines, ending with string->number* on "-7".
- Optionally one line of string->json to show the bundled JSON parser.
- One sentence naming the core combinators (lit, rx, alt, seq, opt, star,
  plus, wrap) and linking to the manual's reference for the rest.
-->

### Getting Started

<!--
- raco pkg install parser-combinator
- A prominent link to the manual at /parser-combinator/, Quick Start first.
-->

### Looking Back

<!--
- Optional. A short paragraph on what reading 2014 code is like now, or what
  you'd do differently. Skip it if there's nothing to say.
-->
