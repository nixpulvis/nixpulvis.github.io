# cratesite

A static site generator where the site is a crate.

A site's pages are Markdown files in its crate. The site's build script
hands them to `cratesite::BuildScript`, which compiles every page into the
crate as a module, documented with the page itself, so `cargo doc` is another
way to read the site. The crate then builds its HTML from the same pages with
`Theme::build`, wrapping them in layouts the site writes itself, usually as
askama templates.

- Markdown with pulldown-cmark, code highlighted with syntect, and
  ```` ```tikz ```` code drawn to SVG with LaTeX and dvisvgm.
- Collections, like `_posts/`, published under a permalink like
  `/posts/:name`.
- Every page's Markdown published next to its HTML, like `/about.md`.
- A command line with `build` and `serve`, which rebuilds on changes.

## Features

- `build` for the build script helper.
- `generate` for the generator.
- `kramdown` to read Markdown written for kramdown: `{:.class}` attribute
  lists, kramdown's heading ids, and `$$` for all math.

See [nixpulvis.com](https://github.com/nixpulvis/nixpulvis.github.io) for a
site built with it, and the crate docs for how to lay one out.
