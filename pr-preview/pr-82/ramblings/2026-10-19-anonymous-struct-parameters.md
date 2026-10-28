---
title: 'Anonymous Struct Parameters: The Idea'
date: '2026-10-19'
collection: ramblings
---

There have been a few [recent posts](#4-relating-to-recent-posts) on how Rust
should adopt _named arguments_, and I even had some thoughts on the matter back
in [a comment I left on the issue in 2016][nixpulvis-2016]. My original idea was
never really something I expected to be adopted, but my point was always that
named arguments are just structs, which is even more true now than back then
given [default field values (RFC 3681)][rfc-3681]. It tried to show a direct
correspondence between positional arguments with tuples and named arguments with
structs, which was cute, but ultimately not good syntax.

[rfc-3681]: https://github.com/rust-lang/rfcs/pull/3681
[nixpulvis-2016]: https://github.com/rust-lang/rfcs/issues/323#issuecomment-213065086