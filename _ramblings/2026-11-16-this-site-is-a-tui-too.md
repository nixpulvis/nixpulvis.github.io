---
title: This Site is a TUI too
layout: rambling
draft: true
---

This site is now available as a TUI! Click the `>_` key next to my name up top
(or just [follow this link](?tui)) and the page you're reading gets redrawn
inside a text-based UI, right in the browser.

If you'd rather read it in an actual terminal, there's a client for that too.
Download a build from the [`tui` release][tui-release], using `curl` to skip
macOS gatekeeper bullshit.

```sh
# aarch64-apple-darwin, x86_64-apple-darwin,
# aarch64-unknown-linux-gnu, x86_64-unknown-linux-gnu
target=aarch64-apple-darwin
curl -L https://github.com/nixpulvis/nixpulvis.github.io/releases/download/tui/nixpulvis-tui-$target.tar.gz | tar -xz
```

Or build it yourself:

```sh
cargo install --locked --git https://github.com/nixpulvis/nixpulvis.github.io nixpulvis-tui
```

Then run it (as `./nixpulvis-tui` if you just downloaded it outside of your
`$PATH`):

```sh
nixpulvis-tui /ramblings/2026-11-16-this-site-is-a-tui-too
```

Both are the same [Ratatui] app, drawing the same widgets, with basically only
the backend swapped. [Ratzilla] renders the browser with WebAssembly, and
[`crossterm`] renders in the terminal.

![This post in the TUI]({{ '/img/ramblings/2026-11-16-this-site-is-a-tui-too/tui.png' | relative_url }})

The idea comes from Tyler Bloom's [avid-rustacean], which does this for their
blog. I liked it enough to want it here, but I didn't want to copy it piece by
piece, so I built it fresh. The main difference is what it renders from. Their
posts are rendered from an intermediate JSON format, and I wanted to render
straight from the Markdown. Along the way it grew into a few crates of its own:
[`ratmark`] is a Ratatui `StatefulWidget` that draws the Markdown,
[`pulldown-kram`] reads it the way Jekyll does, and [`ratjest`] does the
scrolling. This post is mostly about the parts that turned out to be more
interesting than I expected.

## Serving Markdown

Jekyll already turns the site's Markdown into HTML, but parsing that HTML back
into something a TUI can draw would be backwards. So the site publishes the
Markdown itself. A small Jekyll plugin writes each page's source next to its
HTML, after Liquid has run but before kramdown, so includes and loops are
already filled in. Each page goes through its layouts too (everything but the
outer HTML shell), so the title, date and tags you see in the TUI come from
the same templates as the web page.

The only other piece is `/site.json`, with the nav and an index of pages,
titles, dates and word counts. That's the whole API: static files, cacheable,
and you could read them with `curl` if you wanted.

```sh
curl https://nixpulvis.com/ramblings/2026-11-16-this-site-is-a-tui-too.md
```

## `ratmark`

Something has to turn that Markdown into Ratatui text, and that's [`ratmark`].
It parses with [`pulldown-cmark`] and handles the usual CommonMark plus tables,
footnotes, strikethrough, task lists, definition lists and math. You draw a
`MarkdownView` like any other Ratatui widget, and its `MarkdownState` keeps the
document laid out to the current width, tracks scrolling, and remembers where
every link landed so keyboard focus and clicks work on links in the middle of
wrapped text.

Everything beyond that is an opt-in feature, and none of them are on by
default. It would have been easy to lump them together as "Jekyll support",
but they're really different things, and plenty of Markdown wants one without
the others.

- `front-matter-yaml` and `front-matter-toml` are for the YAML (`---`) or TOML
  (`+++`) block at the top of a document. This isn't a Markdown thing at all,
  it's a static site generator thing. Jekyll reads it for a page's title,
  date, tags and layout. `ratmark` doesn't look for it while rendering. You
  split it off first, hand the body to `ratmark`, and deserialize the front
  matter into your own serde type, with [`serde-saphyr`] or [`toml`]. Each
  format is its own feature, so you only build the parser you need.

- `html` is for raw HTML, which is part of CommonMark itself, so any Markdown
  might have some. A GitHub README is full of it, with no front matter in sight.
  Without this feature the HTML is just left out. With it, `ratmark` runs
  [`html5gum`]'s tokenizer over the tags and keeps what makes sense in a TUI:
  text, links, emphasis, lists, tables, `<kbd>` and so on. There's no DOM, just
  a stream of tags, and anything like `<script>` or `<svg>` is skipped.

- `kramdown` is for Markdown written for kramdown, the dialect Jekyll uses,
  which reads a few things differently than CommonMark. It reads with
  [`pulldown-kram`], which gets its own section below. `* TOC` with `{:toc}`
  turns into a list of links to the headings, Markdown inside a `markdown="1"`
  element gets read as Markdown, and attribute lists like `{: #intro .note}`
  are read off blocks and spans instead of showing up as text. A theme can
  style elements by class, so the `{:.new-syntax}` after inline code in my
  [anonymous struct post][anon-struct] picks its color from the theme. It
  wants `html` on too, since kramdown reads the text inside elements like
  `<kbd>` as HTML.

- `syntax` highlights fenced code blocks with [`syntect`]. It uses
  `syntect`'s pure Rust regex engine, so it still builds for wasm.

- `links` puts each link's URL on the cells it covers, which is what makes the
  real links below work. It's the only feature that needs cell extensions,
  which aren't in a Ratatui release yet ([ratatui#2854]), though for now the
  whole crate builds against the [branch][ratatui-cells] that adds them.

- `scroll` makes `MarkdownState` scroll with [`ratjest`], from a wheel, a
  trackpad or a finger. More on that below too.

I've tried hard to keep it a general crate. Whenever something on this site
rendered badly, the fix went into the site's markup or the site's app, not
into `ratmark`. A few times I had to revert something that was really just me
teaching the library about my own CSS.

One fun bit is wide code blocks. A TUI can't really scroll a single block
sideways, so `ratmark` cuts wide lines off with a `‹` or `›` marker and each
block keeps its own horizontal offset. Swipe or scroll sideways over a block to
move just that one, or use `<` and `>` to move the topmost wide block on screen.

Try it on the site's own `Cargo.toml` line for `ratmark`:

```toml
[workspace.dependencies]
ratmark = { git = "https://github.com/nixpulvis/ratmark", default-features = false, features = ["front-matter-yaml", "html", "kramdown", "scroll"] }
```

## `pulldown-kram`

`ratmark` used to handle kramdown itself, with a pile of small rewrites and
special cases. Knowing what kramdown means by a document is really a parser's
job though, and nothing about it is specific to drawing a TUI, so it moved
out into its own crate, [`pulldown-kram`].

It reads kramdown into ordinary `pulldown-cmark` events, each paired with the
attributes an attribute list gives it. First it lightly rewrites the input
where kramdown and CommonMark disagree, then `pulldown-cmark` parses the
result, and then kramdown's extensions are added to the events: attribute
lists, header ids, definition lists, `$$` math, `markdown="1"` HTML, `{:toc}`
and so on. Since what comes out is plain `pulldown-cmark` events, anything
that already renders them keeps working, and can look at the attributes where
it cares. Each extension can be turned off on its own, and there's an HTML
writer that writes elements the way kramdown does.

The tests are the interesting part. The suite runs kramdown's own test cases,
plus a set of cases for each extension, and compares `pulldown-kram`'s HTML
with what kramdown 2.5.2 makes of the same Markdown. A case it doesn't get
right yet goes in a known failures file, with why. Its test passes while the
case still fails, so the suite shows what's missing without being red, and it
fails once the case starts passing, so it comes off the list.

## Links

I wanted links to be real links. Clickable in the terminal, and actual `<a>`
elements in the browser so middle click and "copy link" work like you'd
expect. The problem is that a Ratatui cell is just a symbol and a style, with
nowhere to put a URL.

There's an open proposal for this, [ratatui#2854], with a branch from
WeirdBrains, [`multi-char-cells`][ratatui-cells], that adds extensions to
cells. Its PR, [ratatui#2853], is closed, but I'm building against the branch
for now, and since `ratmark` depends on it, everything else here does too.
`ratmark`'s `links` feature puts the URL on each linked cell, the terminal
client writes it out as an OSC 8 hyperlink, and the browser side wraps those
cells in `<a>` tags.

That last part needed changes to Ratzilla, so I [forked it][ratzilla-fork].
Every change sits on its own branch off upstream, so each one can go back as
its own PR, and they're all merged together into a `site` branch that this
site builds from. The links are [`hyperlinks`][rz-hyperlinks], which wraps
each run of cells that share a link in one `<a href>`, behind a `hyperlinks`
feature, so hovering shows the URL and screen readers see a link.

Two fixes came out of that. Ratzilla replaces its grid element when the
window changes size, and the key and mouse listeners went with it, so input
stopped working after a resize. That's
[`dom-input-after-resize`][rz-input], and `hyperlinks` carries the same
fixes. Someone else ran into the same bug, [ratzilla#192], and their fix,
[ratzilla#194], supersedes mine. And clicks were mapped to cells by splitting
the grid's width between its columns, but the grid is as wide as its parent,
so a click near the right edge could land a column off.
[`dom-mouse-cell-size`][rz-mouse] uses the cell size the backend already
measures instead.

## Scrolling

Reading is mostly scrolling, and it's where a TUI in a browser feels most out
of place. A web page follows your fingers on a trackpad by the pixel and
keeps gliding after a flick, while a terminal jumps a whole line at a
time. Getting the view to feel right took two things: drawing fast enough
that scrolling doesn't stutter, and turning the pixels a trackpad or finger
moves into whole lines and columns.

### The scroll hitch

The first version scrolled, but on my Mac it was choppy. Not terrible, just
enough to notice. Ratzilla's DOM backend redrew changed cells through
`innerHTML`, so the first fix was setting text in place instead, on
[`dom-text-in-place`][rz-text]. That helped a lot, and I figured that was the
end of it.

It wasn't. There was still a little hitch every so often, and I spent a while
looking at my own code before trying it with my extensions turned off. It
was smooth. The hitching was my ad blocker. Its content script runs a
`querySelector` on a timer, and the screen was around sixteen thousand
`<span>` elements, one per cell. Every pass walked all of them.

None of that was a bug in my code, but it was still my problem, since plenty
of people run the same blocker. So two more changes went into the fork. Each
line is now drawn as runs of cells with the same style, which cuts the number
of elements by about 25x. That went on `dom-text-in-place` too. And the
backend can now draw inside any element, on
[`dom-parent-element`][rz-parent], so the site draws inside a shadow root
where the rest of the page, ad blockers included, doesn't see it change at
all.

### A cell at a time

Turning pixels into cells takes more than dividing by the size of a cell. A
slow swipe should still move, a swipe that's a little diagonal shouldn't move
both ways, and a flick should keep gliding for a bit, slowing down, like a
page does.

This started out inside `ratmark`, but none of it is about Markdown, so it moved
out too, into [`ratjest`]. It takes a host's wheel and touch events, adds up
pixels until there's a whole line or column's worth, and locks each gesture to
one axis. In a terminal, `terminal::Wheel` takes `crossterm`'s wheel ticks. In
the browser, its `web` feature listens for wheel and touch events on the
element the view is drawn in, the same one Ratzilla draws in.

What it scrolls is a `Scroll`, a trait with one method. It's given a step of
whole cells and the area the widget was drawn in, scrolls if the step is over
that area, and says whether anything moved. When nothing does, the gesture
ends, which also stops a glide. Ratatui's `ListState` and `TableState` are
`Scroll`s already, and an app with several widgets is one too, by splitting
its area the way it does to draw and trying each part in turn. With `ratmark`'s
`scroll` feature `MarkdownState` is a `Scroll` that moves the page, or the wide
code block under the gesture when it goes sideways.

## Colors

In a terminal, the TUI draws in that terminal's own colors. `ratmark`'s default
themes only use terminal color numbers, like 4 for blue, so the terminal's
palette decides what they look like. The site's app goes a bit further and
mixes the rest of its colors from the terminal's text, background and
16-color palette. The terminal client asks for those with OSC 10, 11 and 4
queries, which work over SSH too.

The site's stylesheet is where its colors are defined, and I didn't want a
second copy of them in Rust that drifts. So the app reads `css/style.css`
while it compiles. It's a tiny parser of `const fn`s that knows just enough
CSS for the site's custom properties: hex colors, `var()`, `light-dark()` and
`color-mix(in oklch, ...)`. The dark palette is what it falls back on when a
terminal doesn't answer. The grays, like the background behind code blocks
and the borders, are written in the CSS as so much of the text mixed into the
background, and the app keeps just that share. Then it mixes them in oklch,
like the browser does, from whatever text and background it's drawn on. If a
color is missing, or written in a way it can't read, the build fails.

In the browser there's no terminal to ask, so the page stands in for one: the
stylesheet gives Ratzilla its palette, as variables like `--ratzilla-color-4`,
set to the site's light or dark colors. Ratzilla doesn't read CSS variables
upstream, so that's another branch in the fork, [`dom-palette`][rz-palette],
which will go back as its own PR. The app reads the same variables, by
letting the browser resolve them on a hidden element, and reads them again
whenever the page switches between light and dark.

## Browser

A TUI inside a web page has to give some things back to the browser.
Text selection was the first. Dragging across the screen used to grab the
borders and padding along with the words, so the page text is now the only
selectable area, and copying keeps the blank lines between paragraphs. That's
[`dom-selectable-area`][rz-select], where whatever draws tells the backend
where its text is, and everything else is drawn with `user-select: none`.
It's built on top of `dom-text-in-place`.

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
to the [`tui` release][tui-release], so you don't need Rust to try it.

`pulldown-kram` and `ratjest` only depend on released crates, so they can go to
crates.io first, and `pulldown-kram`'s 0.1.0 is close. Then `ratmark` can follow
once cell extensions ([ratatui#2854]) land in a Ratatui release.

And the Ratzilla branches should go upstream:

- [`hyperlinks`][rz-hyperlinks], linked cells as `<a>` elements
  ([PR][rz-hyperlinks-pr])
- [`dom-input-after-resize`][rz-input], input after a resize, superseded by
  [ratzilla#194], which is open now
- [`dom-mouse-cell-size`][rz-mouse], clicks in the right column
  ([PR][rz-mouse-pr])
- [`dom-text-in-place`][rz-text], fewer elements and DOM updates
  ([PR][rz-text-pr])
- [`dom-parent-element`][rz-parent], drawing in a given element
  ([PR][rz-parent-pr])
- [`dom-selectable-area`][rz-select], selecting just the text
  ([PR][rz-select-pr])
- [`dom-palette`][rz-palette], a palette from CSS variables
  ([PR][rz-palette-pr])

Longer term I've been thinking about replacing Jekyll with a static site
generator written in Rust, documented with rustdoc, where the pages themselves
are modules of the crate. It would start from the same kind of structures as
the app behind the TUI, in the browser and the terminal alike: the site, with
its nav and index, and its pages, with their front matter. But that's a
different post.

[Ratatui]: https://ratatui.rs
[Ratzilla]: https://github.com/ratatui/ratzilla
[`crossterm`]: https://github.com/crossterm-rs/crossterm
[avid-rustacean]: https://github.com/TylerBloom/avid-rustacean
[`ratmark`]: https://github.com/nixpulvis/ratmark
[`pulldown-kram`]: https://github.com/nixpulvis/pulldown-kram
[`ratjest`]: https://github.com/nixpulvis/ratjest
[`pulldown-cmark`]: https://github.com/pulldown-cmark/pulldown-cmark
[`serde-saphyr`]: https://github.com/bourumir-wyngs/serde-saphyr
[`toml`]: https://github.com/toml-rs/toml
[`html5gum`]: https://github.com/untitaker/html5gum
[`syntect`]: https://github.com/trishume/syntect
[anon-struct]: {% link _ramblings/2026-10-19-anonymous-struct-parameters.md %}
[ratatui#2853]: https://github.com/ratatui/ratatui/pull/2853
[ratatui#2854]: https://github.com/ratatui/ratatui/issues/2854
[ratatui-cells]: https://github.com/WeirdBrains/ratatui/tree/multi-char-cells
[ratzilla-fork]: https://github.com/nixpulvis/ratzilla
[ratzilla#192]: https://github.com/ratatui/ratzilla/issues/192
[ratzilla#194]: https://github.com/ratatui/ratzilla/pull/194
[rz-hyperlinks]: https://github.com/nixpulvis/ratzilla/tree/hyperlinks
[rz-input]: https://github.com/nixpulvis/ratzilla/tree/dom-input-after-resize
[rz-text]: https://github.com/nixpulvis/ratzilla/tree/dom-text-in-place
[rz-parent]: https://github.com/nixpulvis/ratzilla/tree/dom-parent-element
[rz-select]: https://github.com/nixpulvis/ratzilla/tree/dom-selectable-area
[rz-mouse]: https://github.com/nixpulvis/ratzilla/tree/dom-mouse-cell-size
[rz-palette]: https://github.com/nixpulvis/ratzilla/tree/dom-palette

NOTE: fill in the PR links below once they're up.

[rz-hyperlinks-pr]: https://github.com/ratatui/ratzilla/pull/TODO
[rz-mouse-pr]: https://github.com/ratatui/ratzilla/pull/TODO
[rz-text-pr]: https://github.com/ratatui/ratzilla/pull/TODO
[rz-parent-pr]: https://github.com/ratatui/ratzilla/pull/TODO
[rz-select-pr]: https://github.com/ratatui/ratzilla/pull/TODO
[rz-palette-pr]: https://github.com/ratatui/ratzilla/pull/TODO
[tui-release]: https://github.com/nixpulvis/nixpulvis.github.io/releases/tag/tui
