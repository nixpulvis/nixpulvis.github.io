---
layout: project
references:
- https://github.com/nixpulvis/lalrpop-lambda
- https://github.com/nixpulvis/lambash
---

This project both parses and reduces the λ-calculus. For example `(λx.x) 1`
will reduce to `1`. in addition to the string parser, a set of Rust macros:
`var!`, `abs!` (`λ!`), and `app!` (`γ!`) are provided.

The λ-calculus was originally proposed as a tool for use in the construction
of mathematics, however it's be proven equivalent to all known programming
languages [1]. The practical implications of this are called the "Church-Turing
thesis" or simply the **computability thesis** (_complexity_ is another story).
