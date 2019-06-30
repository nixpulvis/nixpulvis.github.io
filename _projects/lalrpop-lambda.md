---
layout: project
references:
- https://github.com/nixpulvis/lalrpop-lambda
- https://github.com/nixpulvis/lambash
---

This project both parses and reduces the λ-calculus. For example `(λx.x) 1`
will reduce to `1`. in addition to the string parser, a set of Rust macros:
`var!`, `abs!` (`λ!`), and `app!` (`γ!`) are provided.

The λ-calculus was originally proposed as a tool for use in the construction of
mathematics, however it's be proven equivalent to all known programming
languages [[1]][1]. The practical implications of this are called the
"Church-Turing thesis" or simply the **computability thesis** (_complexity_ is
another story).

For more detailed documentation of both this project, and the λ-calculus, read
the [documentation][documentation]. This page serves as a more personal account
of my development, and both a (slightly) more approachable introduction to the
λ-calculus, and some of the flexibility of Rust.

I probably first heard about the λ back in grade school in some passing
physics equation, almost certainly as wavelength. Despite how cool light is, it
wasn't until took my first class at Northeastern University and I learned about
the _function_ that I've started holding the λ above all.

The λ-calculus is **so very interesting** to me, because like [e^{\pi i + 1 =
0}][eulers], it is incredibly fundamental. Unlike Euler's identity however,
this calculus is actually at it's core, very easy to understand.

All you have are three components:

- Names (i.e. variables or bindings)
- Abstractions (i.e. functions like f(x))
- Applications (e.g. f(g(x)))

### Basic Syntax

There are a few grammars for the lambda calculus floating around out there, and
we are also specifically aiming to support the simply typed λ-calculus, so we
have our own parser ([LALRPOP][lalrpop]) with an unambiguous grammar to support
this.

Below are some basic examples covering the different forms available in this
grammar. Here the left hand side of `->` _parses_ into the right hand side.

```
x       -> x
x y     -> (x y)
(y x)   -> (y x)
λ       -> (λ_._)
λx      -> (λx._)
λx.x    -> (λx.x)
λx.(x)  -> (λx.x)
λx.x x  -> (λx.(x x))
(a b c) -> ((a b) c)
```

### Reductions

Here the left hand side of `->` _reduces_ to the right hand side. The beta
reduction corresponds to calling a function to get it's result. The simplest
example is applying a single argument to the identity function, seen below:

```
(λx.x) y -> y
```

Functions are themselves values in the λ-calculus, making it what we call a
_higher order language_, allowing us to write more complex reductions.

```
(λf.λg.λx.(f (g x))) (λx.x) (λx.x) y
->
(λg.λx.((λx.x) (g x))) (λx.x) y
->
(λx.((λx.x) ((λx.x) x))) y
->
(λx.((λx.x) x)) y
->
(λx.x) y
->
y

TODO: Visualize the multiple paths to `y`, reduction strategies.
```

- alpha
- beta
- eta

#### alpha

#### beta

###### Small Step Semantics
###### Large Step Semantics
###### Reduction Strategies

- Normal order
- Applicative Order
- Call by name
- Call by value
- ...


### Currying

Our grammar also supports currying, which allows us to write multi-argument
functions on a language that fundamentally only allows one. This is done with
the following scheme:

```
λx y -> λx.λy
```

This reads nicely, lambda x y is the same as lambda x, lambda y. To "call" a
curried function simply call the result of each function call with the arguments
in order.

```
(λx y.(add x y)) 1 2
-> curry
((λx.λy.(add x y)) 1) 2
-> beta
(λy.(add 1 y)) 2
-> beta
(add 1 2)
```

This works in any reasonable programming language with functions and arguments.
In Ruby for example, you can write an `add` function like this if you're so
inclined.

```ruby
def c_add(a)
  lambda { |b| a + b }
end

c_add(1).(2)
=> 3
```


[1]: ...
[documentation]: ...
[lalrpop]: ...
[eulers]: /math/01-eulers
