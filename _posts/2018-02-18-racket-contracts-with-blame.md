---
layout: post
---

Throughout my undergraduate computer science education contracts have been
heavily stressed. However, to many people who didn't study at Northeastern this
concept may be foriegn. The idea of a contract isn't all that different from a
type, since both are used to describe the shape of data. The two main
differences to me are that:

1. Contracts are dynamic, telling you what went wrong at run-time.
2. Contracts are opt-in. Without getting into gradual type systems, this is
general not the case with type systems.

In learning to program Racket you may not write a single contract, but you will
definetly use them. This is because most of the functions (and data) you'll
build with have contracts, and when something goes wrong it's the contracts
that will tell you where. This is because contracts in Racket know how to blame
the correct party, providing you with very helpful error messages.  Consider
the following, where I try to add one to a string:

```racket
(add1 "foo")
```

I get a run-time error, much like a type error.

```racket
; add1: contract violation
;   expected: number?
;   given: "hi"
```

This is because `add1` has the following contract assigned to it:

```racket
(-> number? number?)
```

This looks like a more *normal* type if you remember that Racket is a prefix
notation language. If it helps you, think of that contract as `number? ->
number?`. In the rest of this post, I want to dive into contracts in more
depth.

### Defining Data with a Contract

Typically contracts are attached at module bounderies. A module in Racket can
`provide` an item for use in other modules, and attach a contract to each item
in the process. However, it is also posible to create data with contracts
within a module. These are contracts with *nested contract bounderies*. We'll
be using these in this post, to avoid talking about modules too much.

We'll be using `define/contract` to define an item with a contract directly. We
can define items just like with `define` but with an additional `contract-expr`
as it's 2nd argument. For example:

```racket
; Define `add2` with a contract of it's own.
(define/contract (add2 n)
  (-> number? number?)
  (+ n 2))

(add2 0)
2

(add2 "foo")
; add2: contract violation
;   expected: number?
;   given: "foo"
;   in: the 1st argument of
;       (-> number? number?)
;   contract from: (function add2)
;   blaming: top-level
;    (assuming the contract is correct)
```

Compared to a `define` implementation of `add2`:

```racket
; Define a plain old `add2` without it's own contract.
(define (add2 n)
  (+ n 2))

(add2 0)
2

(add2 "foo")
; +: contract violation
;   expected: number?
;   given: "foo"
;   argument position: 1st
```

As you can see, both definitions of `add2` correctly operate on `0`, and both
definitions correctly error when given `"foo"`, the important difference is
which party is blamed. In the `define` example there is no new contract, and
it's the contract of `+` which detects the error. In our definition with
`define/contract` it's the attached `(-> number? number?)` contract which
catches our error.

### Flat Contracts

Before we look at function contracts any more, let's first start by looking at
the simplest kind of contract. A flat contract is one of:

- Symbols, booleans, keywords, `null`.
- Strings, Characters.
- Numbers
- Regular Expressions
- Predicate functions, i.e. `(-> any/c boolean?)`.

These are the *trival* cases of contracts. Let's take a quick look at creating
data using flat contracts.

```racket
(define/contract happy? #t (or #t #f))
happy?
#t

(define/contract happy? #t (and #t #f))
; happy?: broke its own contract
;   promised: #t
;   produced: #f
;   in: #t
;   contract from: (definition happy?)
;   blaming: (definition happy?)
;    (assuming the contract is correct)
```

As you can see, the contract system caught us when we tried to say that
`happy?` was contractually obligated to be `#t`, but we assigned `(and #t
false)`. Strings, numbers, and regexs all work as you'd expect. Predicate
functions are also pretty straightforward, and provide an easy way to check
whatever condition you'd like.

```racket
; Define our own contract predicate.
(define (natural+? n) (> n 0))

(define/contract one natural+? 1)
(define/contract zero natural+? 0)
; zero: broke its own contract
;   promised: natural+?
;   produced: 0
;   in: natural+?
;   contract from: (definition zero)
;   blaming: (definition zero)
;    (assuming the contract is correct)
```

Pretty simple, and it's nice to be able to talk about our assumptions in code.

### Chaperone and Impersonator Contracts

The most important kind of contract left out of the flat contracts is `->`. This
form of contract is also known as a contract combinator, since it takes many
contracts as arguments and returns a new contract. Any contract created with
`->` (for example `(-> number? number?)`) is a chaperone contract.

Before I get into chaperone and impersonator contracts, first let me explain
impersonators and chaperones beifly. Both are a way to wrap data, where a
chaperone is allowed to view the data, and an impersonator is allowed to modify
it.

```racket
(define impersonated-add1
  (impersonate-procedure add1 (lambda (n) 41)))

; Notice how we update the value that gets passed to `add1`.
(impersonated-add1 0)
42

(define chaperoned-add1
  (chaperone-procedure add1 (lambda (n) 41)))

; A chaperone is not allowed to modify the input to `add1`.
(chaperoned-add1 0)
; procedure chaperone: non-chaperone result;
;  received a argument that is not a chaperone of the original argument
;   original: 0
;   received: 41
```

An important fact about these kinds of wrappers is that they are `equal?` to
thier wrapped functions, whereas most functions aren't `equal?` to anything
else.

```racket
(equal? (lambda (x) x) (lambda (x) x))
#f
(equal? impersonated-add1 add1)
#t
(equal? chaperoned-add1 add1)
#t
```

Below I create a chaperone of the `add1` function that prints it's inputs
and outputs (taken from the Racket documentation).

```racket
(define (post-wrap n)
  (printf "returned: ~s\n" n) n)

(define (wrap n)
  (printf "given: ~s\n" n)
  (values post-wrap n))

(define print-add1 (chaperone-procedure add1 wrap))
```

As you can see `chaperone-procedure` can wrap both before and after the
original function is called. This pattern is perfect for creating a contract on
functions (and other kinds of composite data), since a contract for a function
must check the contracts for it's arguments and results.

```racket
(flat-contract number?)
#t
(flat-contract (-> number? number?))
#f
(chaperone-contract? (-> number? number?))
#t
```
