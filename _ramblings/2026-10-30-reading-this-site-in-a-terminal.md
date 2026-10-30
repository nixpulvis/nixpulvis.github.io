---
title: Reading this Site in a Terminal
layout: rambling
draft: true
---

This site can be read in a terminal now. Click the little `>_` key next to my
name up top (or just [follow this link](?tui)) and the page you're reading gets
redrawn as a terminal app, right there in the browser. If you'd rather use a
real terminal, there's a client for that too:

```sh
cargo install --locked --git https://github.com/nixpulvis/nixpulvis.github.io nixpulvis-tui
nixpulvis-tui /ramblings
```

Both are the same [Ratatui] app. The browser one is compiled to WebAssembly
and drawn with [Ratzilla], the terminal one fetches pages over HTTP and draws
with crossterm. It's `j`/`k` to scroll, `tab` to move between links, `enter`
to follow one, `h` to go back, and `1`-`9` for the nav. The mouse works too.

The idea comes from Tyler Bloom's [avid-rustacean], which does this for his
blog. I liked it enough to want it here, but I didn't want to copy it piece by
piece, so I built it fresh. This post is mostly about the parts that turned
out to be more interesting than I expected.

NOTE: screenshot of this post in the terminal view.

## Markdown is the API

The first question was what the app should actually read. Jekyll already turns
Markdown into HTML, and parsing that HTML back into something a terminal can
draw felt backwards. I thought about dumping kramdown's AST as JSON, but that
ties every client to kramdown's idea of a document.

So the site just publishes its Markdown. A small Jekyll plugin writes each
page's source next to its HTML, after Liquid has run but before kramdown, so
includes and loops are already filled in. Each page goes through its layouts
too (everything but the outer HTML shell), so the title, date and tags you see
in the terminal come from the same templates as the web page. Then there's a
`/site.json` with the nav and an index of pages, titles, dates and word counts.

That's the whole API. Static files, cacheable, and you could read them with
`curl` if you wanted.

## ratmark

Something has to turn that Markdown into Ratatui text, and that's [ratmark].
It parses with [pulldown-cmark] and handles the usual CommonMark plus tables,
footnotes, task lists and math. It keeps the document laid out to the current
width, tracks scrolling, and remembers where every link landed so keyboard
focus and clicks work on links in the middle of wrapped text.

Everything beyond that is an opt-in feature, and none of them are on by
default. It would have been easy to lump them together as "Jekyll support",
but they're really different things, and plenty of Markdown wants one without
the others.

- `front-matter` splits the YAML (`---`) or TOML (`+++`) block off the top of a
  document, and has a small helper for reading a value like the title. This
  isn't a Markdown thing at all, it's a static site generator thing. Jekyll,
  Hugo and Zola all do it, each with its own flavor of Markdown. It doesn't pull in a
  YAML parser either, it just hands you the text and lets you parse it however
  you want.

- `html` is for raw HTML, which is part of CommonMark itself, so any Markdown
  might have some. A GitHub README is full of it, with no front matter in sight.
  Without this feature the HTML is just left out. With it, ratmark runs
  [html5gum]'s tokenizer over the tags and keeps what makes sense in a terminal:
  text, links, emphasis, lists, tables, `<kbd>` and so on. There's no DOM, just
  a stream of tags, and anything like `<script>` or `<svg>` is skipped.

- `kramdown` is for Markdown written for kramdown, the dialect Jekyll uses,
  which reads a few things differently than CommonMark. `* TOC` with `{:toc}`
  turns into a list of links to the headings, Markdown inside a `markdown="1"`
  element gets read as Markdown, heading ids match the ones kramdown gives the
  HTML (so `#foo` links land in the same place), and attribute lists like
  `{:.no_toc}` are dropped instead of showing up as text. A class after inline
  code, like the `{:.new-syntax}` in my [anonymous struct post][anon-struct],
  can even pick a style from the theme.

- `syntax` highlights fenced code blocks with [syntect]. It uses syntect's pure
  Rust regex engine, so it still builds for wasm.

- `native-links` puts each link's URL on the cells it covers, which is what
  makes the real links below work. It's the only feature that needs an
  unreleased Ratatui.

So the site turns on all five, but something reading Hugo content would want
`front-matter` and `html` and nothing to do with kramdown, and a README viewer
might only want `html`.

I've tried hard to keep it a general crate. Whenever something on this site
rendered badly, the fix went into the site's markup or the site's app, not
into ratmark. A few times I had to revert something that was really just me
teaching the library about my own CSS.

One fun bit is wide code blocks. A terminal can't really scroll a single block
sideways, so ratmark cuts wide lines off with a `‹` or `›` marker and each
block keeps its own horizontal offset. Swipe or scroll sideways over a block to
move just that one, or use `<` and `>` to move the topmost wide block on screen.

## Real links

I wanted links to be real links. Clickable in the terminal, and actual `<a>`
elements in the browser so middle click and "copy link" work like you'd
expect. The problem is that a Ratatui cell is just a symbol and a style, with
nowhere to put a URL.

There's an open proposal for this, [ratatui#2854], with a branch from
WeirdBrains that adds extensions to cells. I'm building against that branch
for now. ratmark puts the URL on each linked cell, the terminal client writes
it out as an OSC 8 hyperlink, and the browser side wraps those cells in `<a>`
tags.

That last part needed changes to Ratzilla, so I [forked it][ratzilla-fork].
Every change sits on its own branch off upstream, so each one can go back as
its own PR, and they're all merged together into a `site` branch that this
site builds from.

## The scroll hitch

The browser view worked, but scrolling on my Mac was choppy. Not terrible, just
enough to notice. Ratzilla's DOM backend redrew changed cells through
`innerHTML`, so the first fix was setting text in place instead. That helped a
lot, and I figured that was the end of it.

It wasn't. There was still a little hitch every so often, and I spent a while
looking at my own code before trying it with my extensions turned off. It
was smooth. The hitching was my ad blocker. Its content script runs a
`querySelector` on a timer, and the screen was around sixteen thousand
`<span>` elements, one per cell. Every pass walked all of them.

None of that was a bug in my code, but it was still my problem, since plenty
of people run the same blocker. So two more changes went into the fork. Each
line is now drawn as runs of cells with the same style, which cuts the number
of elements by about 25x. And the backend can now draw inside any element, so
the site draws inside a shadow root where the rest of the page, ad blockers
included, doesn't see it change at all.

## Being a web page

A terminal app inside a web page has to give some things back to the browser.
Text selection was the first. Dragging across the screen used to grab the
borders and padding along with the words, so the page text is now the only
selectable area, and copying keeps the blank lines between paragraphs.

History was the second, and I had strong opinions here. Every link to another
page on the site is a real page load. The browser owns history, the back
button is the browser's back button, and `h` just calls `history.back()`. I
don't like sites that fake navigation with `pushState` and then try to keep
their own idea of history in sync with the browser's. A page load here is
cheap anyway, the app is cached and the Markdown is small.

And finally there's the toggle. I went through a few ideas, like a fake
`$ less foo.md` prompt, a bare glyph in the nav, or making the lambda clickable.
They were cluttered, looked like they were floating, or were impossible to
find. The framed `>_` key after my name is small but looks like something you
can press, which is all it needs to do. The choice sticks between pages, and a
link ending in `?tui` turns it on for whoever follows it.

## What's next

The terminal client is built for Linux and macOS on every change and attached
to the [`tui` release][tui-release], so you don't need Rust to try it. I'd like
to get it onto crates.io, but that's blocked until cell extensions land in a
Ratatui release, or I move the link code somewhere the published crates don't
need it.

The Ratzilla branches should go upstream, and I want to clean up the ratmark
API before giving it a real release.

Longer term I've been thinking about replacing Jekyll with a static site
generator written in Rust, documented with rustdoc, where the pages themselves
are modules of the crate. But that's a different post.

[Ratatui]: https://ratatui.rs
[Ratzilla]: https://github.com/ratatui/ratzilla
[avid-rustacean]: https://github.com/TylerBloom/avid-rustacean
[ratmark]: https://github.com/nixpulvis/ratmark
[pulldown-cmark]: https://github.com/pulldown-cmark/pulldown-cmark
[html5gum]: https://github.com/untitaker/html5gum
[syntect]: https://github.com/trishume/syntect
[anon-struct]: {% link _ramblings/2026-10-19-anonymous-struct-parameters.md %}
[ratatui#2854]: https://github.com/ratatui/ratatui/issues/2854
[ratzilla-fork]: https://github.com/nixpulvis/ratzilla
[tui-release]: https://github.com/nixpulvis/nixpulvis.github.io/releases/tag/tui
