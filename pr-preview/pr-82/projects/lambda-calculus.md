---
title: The λ-calculus
status: incomplete
references:
- https://github.com/nixpulvis/lalrpop-lambda
- https://github.com/nixpulvis/lambash
collection: projects
---

λ-calculus one of the most fundamental programing languages, being made up of
only **three primative types** of objects. It's the basis for functional
programming, and is just as powerful as any other programming language. I've
created a parser and reducer for this language which, for example, will reduce
`(λx.x) 1` to `1`. In addition to a parser, which consumes arbitrary text, a
set of Rust macros: `var!`, `abs!` (`λ!`), and `app!` (`γ!`) are provided.