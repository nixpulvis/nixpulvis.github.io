---
layout: rambling
title: "Anonymous Struct Parameters: The Prototype"
date: "2026-09-29 00:00:01"
draft: true
---

This is a follow-up to [Anonymous Struct Parameters][first-post], covering the
rustc prototype behind it, and the details and open questions that didn't fit
there.

---
* TOC
{:toc}
---

## 1. The Prototype

Everything in Sections 1 to 3 of the [first post][first-post] is implemented in
a [rustc prototype][struct-args-branch] as five incomplete features,
`anonymous_struct_parameters`, `struct_pattern_inference`,
`anonymous_struct_pattern_elision`, `parameter_type_elision`, and
`named_arguments`, each gated separately so they can be judged separately.
Defaults and `..` also need `#![feature(default_field_values)]`, which landed in
[rust-lang/rust#129514][rust-129514]. The examples in both posts that compile
come from it, and [Prototype gaps](#prototype-gaps) lists where it falls short.

<!-- TODO(voice): "about 2,000 lines", long "so most of the compiler, like ..." sentence, "leaves the door open" -->

It's about 2,000 lines of changes to the compiler and tools. The idea is to turn the new
syntax into an ordinary struct, struct pattern, or struct literal as early as
possible, so most of the compiler, like privacy, borrow checking, and MIR
building, doesn't see anything new. Internally an anonymous struct is called a
*record*, which is where names like `TyKind::Record` and `{record@..}` come
from. The name also leaves the door open for extending them later, toward the
[record types](#record-types) of [Section 3](#3-going-further).

### The core feature

<div class="highlight-block" markdown="1">
```rust
#![feature(anonymous_struct_parameters, struct_pattern_inference, anonymous_struct_pattern_elision, default_field_values)]

fn f(a: i32, struct { x: i32, y: i32 = 0 }) {}
f(0, _ { x: 1, .. });
```
</div>

<!-- TODO(voice): five bold-label bullets, spec register with no walk-through. Parsing: "the type its pattern spells out, so". Lowering: "also when it's nested ..., which is how" chain. Types: "tells it apart in the few places that care", ", since" tail -->

- **Parsing.** `struct { .. }` is a new AST type, `TyKind::Record`, which holds
  the fields. It's only parsed as a parameter's type or a field's type inside
  another one, and anywhere else it's an error. A parameter that's just the type
  gets the pattern `<_>::_ { x, y }`, which `_ { x, y }` also parses to. An
  elided parameter type is the type its pattern spells out, so `Point { x, y }`
  becomes `Point { x, y }: Point`, and `&T(x)` becomes `&T(x): &T`. And `_ { .. }`
  in an expression is a struct literal whose path gets filled in later by type
  checking.
- **Lowering.** Name resolution gives each anonymous struct its own `DefId`, and
  AST lowering turns it into a real struct item. The struct shares its
  function's generics, also when it's nested in another one's field, which is
  how its fields can use the function's generic parameters, `impl Trait`,
  `Self`, and lifetimes without declaring any of them. Lifetimes elided in fields
  become early-bound parameters of the function, so the struct can use them too.
- **Types.** After that a record is an ordinary ADT, its struct applied to the
  function's own generic arguments. `tcx.is_record` tells it apart in the few
  places that care, like printing it the way closures are printed,
  `{record@src/main.rs:6:9: 6:15}`, and skipping it in item lints and dead code
  checks, since it's documented and used through its function.
- **Type checking.** The `<_>::_` pattern takes its struct from the type it
  matches, and an inferred literal takes its struct from the expected type, which
  is also how `_ { .. }` builds and matches named structs.
- **Traits.** In a trait impl, the parameter's type isn't the impl's own
  anonymous struct. It's the trait method's struct, with the impl's generic
  arguments filled in, and the fields the impl writes out are just checked
  against the trait's by name and type, nested ones included. A method stays dyn
  compatible unless one of its fields mentions `Self`.

Rustdoc, Clippy, and rustfmt only needed small changes to match
([Tooling](#tooling)).

### The sugar

<div class="highlight-block" markdown="1">
```rust
#![feature(anonymous_struct_parameters, named_arguments, default_field_values)]

fn f(a: i32; x: i32, y: i32 = 0) {}
f(0, x: 1, ..);
```
</div>

<!-- TODO(voice): "The sugar is smaller still, and almost all of it is in the parser." -->

The sugar is smaller still, and almost all of it is in the parser.

<!-- TODO(voice): Named arguments item: "its span (its location in the source)", two long chained sentences -->

- **Named arguments.** Named arguments at the end of a call are parsed into one
  inferred struct literal, flagged so the pretty printer and rustfmt can print
  them back the way they were written. AST lowering adds the implied `..` and
  tags the literal's span (its location in the source) as coming from the named
  argument desugaring, the same way the code for `for` loops, `?`, and `a..b`
  gets tagged. Type checking uses the tag to skip the `default_field_values`
  feature gate for the implied `..`, and error messages use it so a missing
  field is reported as "missing named argument `width`" instead of pointing at a
  `..` nobody wrote.
- **`;` parameters.** These are handled entirely by the parser, which turns
  everything after the `;` into the fields of a last `TyKind::Record` parameter,
  the same as writing `struct { .. }`.
- **Left-out structs.** This is the only part that reaches type checking. When a
  call has one argument fewer than the function takes, and the last parameter is
  an anonymous struct whose fields all have defaults, the argument check skips
  that parameter and makes a note of the call. When building THIR, each of those
  calls gets a `_ { .. }` argument added, so MIR building and borrow checking see
  the call as if it had been written out.

## 2. Limitations and Open Questions

### Nested anonymous structs

A field could also be an anonymous struct itself. Nesting isn't new, with named
structs it's already valid Rust today:

```rust
struct Marker { at: Point }

fn i(Marker { at: Point { x, y } }: Marker) {}
i(Marker { at: Point { x: 1, y: 2 } });
```

This only binds `x` and `y`, the `at` field is taken apart but never bound.

Without elided patterns, an anonymous field is just another type, so it's built
and taken apart the same way:

<div class="highlight-block" markdown="1">
```rust
fn j(_ { at: _ { x, y } }: struct { at: struct { x: i32, y: i32 } }) {}
j(_ { at: _ { x: 1, y: 2 } });
```
</div>

To keep `at` as well as its fields, bind it with an `@`. Since `x` and `y` are
`Copy`, `at` and its fields can each have their own copy:

```rust
fn k(Marker { at: at @ Point { x, y } }: Marker) {}
```

The anonymous version is the same pattern with `_` in place of the names:

<div class="highlight-block" markdown="1">
```rust
fn l(_ { at: at @ _ { x, y } }: struct { at: struct { x: i32, y: i32 } }) {}
```
</div>

With fields that aren't `Copy`, like a `String`, `at` and its fields can't both
own the same values, so the best you can do is borrow both:

```rust
struct Label { name: String }
struct Tagged { at: Label }

fn m(Tagged { at: ref at @ Label { ref name } }: Tagged) {}
```

And again anonymously:

<div class="highlight-block" markdown="1">
```rust
fn n(_ { at: ref at @ _ { ref name } }: struct { at: struct { name: String } }) {}
```
</div>

Once the pattern is elided though, it's not exactly clear how binding should
work:

<div class="highlight-block" markdown="1">
```rust
fn j(struct { at: struct { x: i32, y: i32 } }) {}
```
</div>

Elided fields can already take binding modes like `ref`, so the inner fields
could too. Borrowing everything gives you `at` and its fields at once, just
like `ref at @ _ { ref x, ref y }` did:

<div class="highlight-block" markdown="1">
```rust
fn j(struct { ref at: struct { ref x: i32, ref y: i32 } }) {}
```
</div>

The biggest issue is name collisions. Binding fields at every level puts them
all in the same scope, so two inner structs with the same field name clash:

<div class="highlight-block red" markdown="1">
```rust
fn o(struct { from: struct { x: i32 }, to: struct { x: i32 } }) {}
```
</div>

Binding only the outer fields never has this problem, since a struct's field
names are already unique. But I don't think that's the right call, it's just
the easy one. A collision like this could be an error, fixed by writing the
pattern out and renaming them, like `_ { from: _ { x: from_x }, to: _ { x: to_x } }`.

### Implied defaults

The first post requires named arguments to end with `..` to use defaults
([Defaults][first-post-defaults]). Implying it instead, which is what the
prototype does, would be closer to how Swift, Python, and Julia leave out named
arguments, but then a call never shows which fields took their defaults. For an
anonymous struct, at least the defaults would be the function's own, since the
struct only exists for it.

<!-- TODO(voice): "on top of calls that leave the struct out" -->

Named structs are less clear. If they also get the implied `..`, a library
adding a default to a field of a public struct changes which calls compile, so
it depends on how `#[non_exhaustive]` ends up working with RFC 3681 defaults
([API evolution and SemVer][first-post-semver]). Writing the `..` yourself, as
in `Instance::new(backends: Backends::VULKAN, ..InstanceDescriptor::new_without_display_handle())`,
works either way. If they don't get it, switching a parameter from an anonymous
struct to a named one breaks named arguments that leave out fields, on top of
calls that leave the struct out.

<!-- TODO(voice): "settles the named struct question"; run of ", so ..., but ..." sentences on a judgment call with no "I" -->

Requiring the `..`, as the first post does, keeps the defaults visible
and settles the named struct question. But a call that only wants the defaults
then needs a `..` on its own, and `f(..)` is already a `RangeFull`, as in
`v.drain(..)`, just like `f(..base)` is a `RangeTo`. One way out is to decide
what `..` means by the type of the parameter it fills. If it's a struct,
`f(..)` means `f(_ { .. })` and `f(..base)` means `f(_ { ..base })`. Otherwise,
for `RangeFull`, `RangeTo`, or a generic `R: RangeBounds<usize>` like
`drain`'s, it means what it does today. Passing a range where a struct is
expected is an error today, so no existing call would change meaning, but `..`
could no longer be read without knowing the signature. A `;` before the named
arguments, which the first post [mentions as an option][first-post-call], would
avoid the question, since `f(a; ..)` can't be a range.

<!-- TODO(voice): four-clause first sentence (", but ..., since ..., and ..."), "The catch is" -->

Leaving the struct out has an API evolution upside. Since adding a parameter is
breaking, new options should usually go in the existing struct as fields with
defaults, but a new last anonymous struct where every field has a default can
be added too, since calls can leave it out, and `crop(&img, width: 200, ..)`
still fills the old one. The catch is that a function used as a value still
takes the struct, so changing `fn f()` to `fn f(struct { a: i32 = 0 })` keeps
every `f()` compiling, but breaks `opt.unwrap_or_else(f)` ([Function
types](#function-types)).

### Defaults in trait impls

<!-- TODO(voice): densest paragraph in the post; "what the trait already says" -->

The `impl` can't repeat the defaults, since they come from the trait. A call
fills in the defaults before the method is dispatched, and through generics or
`dyn` the caller can't know which impl it's calling, so only the trait's
defaults can be used consistently. Letting the impl repeat them, checked
against the trait's, would mean comparing values like `Self::N` after filling
in the impl's types, which isn't worth it just to repeat what the trait already
says. Instead, writing one in the impl is an error that points at the trait's
default, so a reader of the impl can find it.

### Non-const defaults

TODO

<!-- TODO(voice): first draft paragraph, "a limit of RFC 3681 rather than of records, so lifting it belongs in" -->

<!--Field defaults are RFC 3681 defaults, so they have to be const. A default can't call `RandomState::new()`, and it can't reach a generic type's value through its bound, like `S::default()`. A field that needs one becomes an `Option` that the body fills in, as in the `HashMap` example in the [first post][first-post-botahamec]. Named structs can fall back on `..Default::default()`, which runs ordinary code, but an anonymous struct has no `Default` impl to call. This is a limit of RFC 3681 rather than of records, so lifting it belongs in a follow-up to that RFC ([What's Next][first-post-whats-next]).

RFC 3681 makes defaults const on purpose. Its [rationale][rfc-3681-const] is that `Foo { .. }` stays deterministic and cheap, and that a crate can't break a downstream `const fn` by quietly changing a default to a non-const one. Its [future possibilities][rfc-3681-non-const] leave non-const values open, "potentially allowed but linted against", and expect `Default` impls to become const where they can. The [tracking issue][rust-132162] lists "Expand support to non-const values?" as an open question that doesn't block stabilization, with the author's position that "we shouldn't do that, particularly seeing how powerful const eval is becoming".

TODO(voice): "That question is where the discussion is." opener; reports the thread with no take of your own

That question is where the discussion is. [cart argued][cart-2026], from Bevy, that `bar: Bar = Default::default()`, `String`s, and `seed: usize = rand()` are exactly the defaults people want, and asked why a struct literal can't just be const when all of its defaults are. [scottmcm replied][scottmcm-2026] that [const traits][const-traits-goal] are a 2026 project goal, and asked which cases truly *can't* be const, and how `seed: usize = rand()` compares to `seed: Option<usize> = None`. That last comparison is the `HashMap` example, and this [reply][vultix-2026] applies here too: `Option` says the hasher is optional, when every map has one and only passing it is optional. Const traits would cover `S::default()` through a `[const] Default` bound, but not `RandomState::new()`, which seeds itself from the OS at runtime.-->

### Function types

TODO

<!-- TODO(voice): first draft paragraph, "Limiting it to the same fields would keep ..., never drop ..., and leave room for ..." -->

<!--It would be nice if two functions whose anonymous structs have the same fields, with the same names and types, could unify as values, so that `if c { resize } else { grow }` compiles, and `type Callback = fn(struct { width: u32, height: u32 })` accepts both. The prototype doesn't do this. The generated structs are distinct types whose layouts aren't guaranteed to match, so one possible approach is a conversion shim, like the existing reify shims, that moves the fields from one struct into the other. Limiting it to the same fields would keep `if`/`else` symmetric, never drop a field behind the caller's back, and leave room for structural records to replace the shim later.

TODO(voice): "the same-fields rule is the conservative place to start. None of this is designed in detail yet."

The subset and superset conversions of [record types](#record-types) would stretch this further. A function taking `struct { width: u32, height: u32, x: u32 = 0 }` could stand in for a `Callback`, with the shim filling in `x` from its default, and a shim going the other way would drop `x`, which is the silent loss the same-fields limit avoids. Which functions unify would then depend on which conversions apply in which direction, not just on their fields, so the same-fields rule is the conservative place to start. None of this is designed in detail yet.

Leaving out a struct raises a similar question for a single function. `fn f(struct { a: i32 = 0 })` can be called as `f()`, but as a value it still takes one argument, so `opt.unwrap_or_else(f)` doesn't compile. One option is for `f` to implement `FnOnce()` as well as `FnOnce(_)`, filling in `_ { .. }` when it's called without the struct. That's pretty close to overloading by arity, which Rust has avoided so far, and it would make inference ambiguous anywhere the argument types aren't known yet. It should probably be worked out together with the other `Fn` trait questions records raise, like the generic case below.

Generic records are harder. In `fn higher_order<A>(f: impl Fn(A)) { f(a: 4, b: 2) }`, the body can't build an `A`, since the caller picks it. Today that needs a named struct (`impl Fn(FooArgs)`).-->

### Macros

Procedural macros get the tokens as written, and most of them parse those tokens
with [`syn`][syn], which expects every parameter to be `pat: ty`, separated by
commas. So `syn` rejects anonymous struct parameters, elided parameter types,
and `;` parameter lists, which means `#[tokio::main]` or `#[tracing::instrument]`
can't be used on functions that have them. Macros that parse the whole function
body also fail on named arguments, since `f(x: 1)` isn't an expression `syn`
knows about. This isn't unique to anonymous structs though. `syn` only started
parsing RFC 3681's `= default` fields in [3.0][syn-3], so derives still built on
syn 2 fail on structs that use them. syn 3.0 also anticipates several in-flight
RFCs, so new syntax doesn't have to wait until it's close to stable to show up.

<!-- TODO(voice): "a `pat: ty` spelling that fits what `syn` already has", "can be read as what it stands for" -->

Every new form has a `pat: ty` spelling that fits what `syn` already has.
`p: struct { x: i32 }` is already `pat: ty`, with `struct { .. }` as an opaque
type. A bare `struct { x: i32 }` is `_ { x }: struct { x: i32 }`, using the
inferred struct pattern. An elided parameter type can be read as what it stands
for, `Point { x, y }: Point`, and a `;` parameter list as a last
`struct { .. }` parameter. They're all just sugar, so a macro that prints the
expanded form back out doesn't change what the function means. That was a goal
of the design, every parameter form should have a `pat: ty` version someone
could write by hand.

Even once `syn` can parse these, some macros will need updating. A macro that
just passes the signature through, as `#[tokio::main]` mostly does, would work
right away. But `#[tracing::instrument]` records each parameter by its binding
name, so it would need to know that an anonymous struct's fields are its
bindings.

<!-- TODO(voice): "The fix is to ..." stated as settled, colon plus ", which" chain -->

`macro_rules!` has a subtler problem. Today a `$p:pat`, `$e:expr` (2024) or
`$s:stmt` fragment at the end of a macro arm, given `_ { x }`, parses just the
`_` and leaves `{ x }` behind, so the arm fails and a later arm, like
`(_ { $($t:tt)* })`, can match. With `_ { .. }` patterns and literals the
fragment takes the whole thing, so the earlier arm matches instead, which breaks
existing macros even in crates that don't use the feature. The fix is to keep
the old behavior in those fragments: `_` followed by `{` at the fragment's
outer level stays a lone `_`, which matches exactly what they did before.
Nested inside brackets it was always an error, so it's fine to allow there.
Rust has changed what a fragment matches before, with `pat_param` in 2021 and
`expr_2021` in 2024, so a later edition could let them match `_ { .. }` too.

### Tooling

TODO

<!-- TODO(voice): draft below, "about as much work", "about ten lines", five-clause rustfmt sentence -->

<!--Tools that parse Rust themselves don't get any of the compiler's changes. rust-analyzer's parser treats `struct {` in a parameter list as an error and loses highlighting for the rest of the signature, and semantic support needs about as much work as the prototype did. tree-sitter-rust, which Neovim, Helix, Zed, and GitHub use, needs about ten lines of grammar, and currently lacks RFC 3681's default field values too. rustfmt uses rustc's parser, so it formats `_ { .. }` literals and patterns, elided parameter types, and `;` parameter lists, putting the `;` on its own line when the signature doesn't fit on one, but keeps anonymous struct parameters and named arguments as written.-->

### Prototype gaps

These are gaps in the prototype, or places where it had to pick something the
design hasn't decided yet:

<!-- TODO(voice): item 5 "An elided parameter type is purely syntactic." -->

- Named arguments imply `..`, which the first post's design doesn't
  ([Implied defaults](#implied-defaults)). The prototype also implies it for
  every struct, named or anonymous, because it adds the `..` during AST
  lowering, before it knows which struct the call is for.
- A function whose last anonymous struct can be left out still can't be used as
  a value without it. For `fn f(struct { a: i32 = 0 })`, `opt.unwrap_or_else(f)` is
  an error. Making it work would mean `f` implementing `FnOnce()` as well as
  `FnOnce(_)` ([Function types](#function-types)).
- The `macro_rules!` fragments above still take `_ { .. }`, which needs the
  fix described in [Macros](#macros) before the prototype is compatible.
- An elided parameter type is purely syntactic. A variant pattern like `E::V(x)`
  can't elide its type even when `E` has a single variant and the pattern is
  irrefutable.
- Rustdoc shows an anonymous struct parameter as `_`, without its fields.
- Some error messages could be better, especially for mistakes in named
  arguments.

## 3. Going Further

<div class="aside" markdown="1">
This section is optional reading. It's less developed than the rest of the
post, and the main proposal doesn't depend on it. Neither optional fields nor
`let` records are in the prototype. Both are collapsed below, so feel free to
skip them.
</div>

<details id="optional-fields">
  <summary>Optional fields: passing <code>T</code> for an <code>Option&lt;T&gt;</code> field</summary>
<!--
  <div markdown="1">
TODO(voice): "That takes two rules, which come as a pair."

So far a field is optional because it has a default, and `Option` is just
another type a field can have. We could consider going much further, and make
`Option` a first-class part of what makes an optional field optional. An
`Option<T>` field would be optional by its type alone, and callers could pass a
plain `T` instead of wrapping it. That takes two rules, which come as a pair.

- A field of type `Option<T>` without a default gets `= None` implicitly.
- A named argument of type `T` for a field of type `Option<T>` is wrapped in
  `Some`.

<div class="highlight-block" markdown="1">
```rust
fn crop(img: &Image; width: u32, height: Option<u32>) -> Image
// = fn crop(img: &Image, struct { width: u32, height: Option<u32> = None }) -> Image

crop(&img, width: 200, ..);
// = crop(&img, _ { width: 200, height: None })
crop(&img, width: 200, height: 100);
// = crop(&img, _ { width: 200, height: Some(100) })
crop(&img, width: 200, height: None);
// = crop(&img, _ { width: 200, height: None })
```
</div>

That makes the call most people write first, `crop(&img, width: 200, height:
100)`, compile. It also makes changing a field from `height: u32 = 0` to
`height: Option<u32>` non-breaking, since callers' `height: 100` still compiles.

TODO(voice): "Whether it should apply to named structs is open, same as ..., and ..., so that ..." chain; "The catch is"

Like an [implied `..`](#implied-defaults), this would be a deliberate
exception to "the sugar never adds meaning", and it would only apply to named
arguments. `_ { height: 100 }` would still be an error, otherwise every struct
literal would change. Whether it should apply to named structs is open, same as
the implied `..`, and it should probably be decided the same way, so that a
library switching between a named struct and an anonymous one doesn't change
which of its callers' calls compile. The catch is that a call's arguments can't always
move into a [`let` record](#record-types) unchanged. `crop(&img, width: 200,
height: 100)` compiles, but the same fields in a record don't.

TODO(voice): clipped "It isn't free." opener; "Trying the field's own type first ..." nested qualifier sentence

It isn't free. Wrapping depends on the argument's type, but the argument's type
often depends on the field's, so in `height: s.parse()?` the field no longer
tells the compiler whether `parse` should produce `u32` or `Option<u32>`. Trying
the field's own type first, and wrapping only if that fails, keeps every call
that compiles today meaning what it does today, including `height: x.into()`,
but makes whether a call wraps depend on what the compiler can infer. Wrapping
only when the argument's type is already known is easier to predict, but
rejects calls that look like they should work.

Nesting needs a rule too. An `Option<Option<T>>` field, for "not set" versus
"explicitly cleared", makes `None` ambiguous between `None` and `Some(None)`.
Never wrapping into a field whose `T` is itself an `Option` avoids that, but
then a generic `Option<T>` field wraps or doesn't depending on `T` at each call.

TODO(voice): "This would be one, if a narrow one.", "the conservative path is"

And Rust has avoided implicit `T` to `Option<T>` conversions so far. It isn't
one of the [coercions][ref-coercions], a [request for it][rfcs-2416] was closed
as unlikely to happen, and the compiler [suggests][rust-42764] wrapping in
`Some` instead. This would be one, if a narrow one. It isn't in the prototype,
and the conservative path is to keep writing `Some` and `= None` explicitly,
adding wrapping later only with an inference rule that can't change what an
existing call means.
  </div>
-->
</details>

<details id="record-types">
  <summary>Record types: type identity based on fields</summary>
<!--
  <div markdown="1">
TODO(voice): opens on a dictionary-style definition, "decided by its fields, not its name", six spec sentences with no "I"/"we", "where the target type is known" wedged mid-clause

A *record* is a struct type whose compatibility with other types is decided by its fields, not its name. Named structs deliberately aren't like this. Two structs with the same fields are still different types, because the name carries meaning, which is what makes newtypes work. A record is intrinsically unnamed, so there's no meaning to protect and it doesn't act like a newtype. Records can relate in two ways. With strict field equality, two records with the same fields are the same type, which is what structural records propose. With subsets and supersets, a record converts, where the target type is known, into any struct whose fields are a subset or a superset of its own. This section is mostly about the second.

#### Why: subsets and supersets

TODO(voice): "arguments rarely live at a single call", ", which couples their APIs" inside an either/or

Everything in the [first post][first-post] ties a struct to one function. That's fine at a single call, but arguments rarely live at a single call. Options get built once and passed to several functions, or passed through a wrapper to the function it wraps, and those functions each want a different slice of them. Without records, you either share one named struct between the functions, which couples their APIs to each other, or copy the fields across by hand at every call.

Records solve this with two kinds of compatibility:

TODO(voice): bold-label Superset/Subset bullets

- **Superset.** A record may have more fields than the struct it's passed as. The extra fields stay behind, available to the next call.
- **Subset.** A record may have fewer fields than the struct it's passed as. The missing fields take the struct's defaults, as if the call had written `..`.

Most real uses need both at once:

<div class="highlight-block" markdown="1">
```rust
fn crop(img: &Image, struct {
    width: u32,
    height: Option<u32> = None,
    x: u32 = 0,
    y: u32 = 0,
}) -> Image { .. }

fn encode(img: &Image, struct {
    quality: u8 = 80,
    progressive: bool = false,
}) -> Vec<u8> { .. }

let opts = _ { width: 200, height: Some(100), quality: 90 };
let thumb = crop(&img, opts);  // `quality` stays, `x` and `y` default
encode(&thumb, opts);          // `width` and `height` stay, `progressive` defaults
```
</div>

TODO(voice): balanced pair of summing-up sentences that restate the example

Neither function knows about the other, and neither has to agree on a shared options type. The record is the only thing that sees all the fields, and each call takes what it needs by name.

#### Records built before the call: `let args = _ { ... };`

TODO(voice): parenthetical definitions "(the record is a superset)", repeats the Superset/Subset bullets

A record bound by `let`, without a type annotation or `..`, gets a struct of its own, made from its field names. It converts by field name into whatever struct it's passed as. Fields the target lacks stay behind in the record (the record is a superset), and fields the record lacks take the target's defaults (the record is a subset). It's `Copy` when all its fields are, so one record can go to several functions:

<div class="highlight-block" markdown="1">
```rust
fn greet(struct { name: &'static str, times: usize = 1 }) { .. }
fn greet_prefix(struct {
    name: &'static str,
    times: usize = 1,
    prefix: &'static str,
}) { .. }

let args = _ { name: "hello", times: 2, prefix: "-- " };
// `prefix` stays behind
greet(args);
// `args` is `Copy`, so it can be used again
greet_prefix(args);
```
</div>

A record isn't tied to one struct, named or anonymous. The same record can fill both `fill`'s `Rgb` and `crop`'s anonymous struct from the [first post's examples][first-post-examples]:

<div class="highlight-block" markdown="1">
```rust
let args = _ {
    red: 0xA8, green: 0x3C, blue: 0x09,
    width: 200,
};
fill(&mut shape, args);
crop(&img, args);
```
</div>

Any other `_ { .. }` whose struct isn't known yet, including a `let` record with `..`, takes the struct of its first use, without a conversion:

<div class="highlight-block" markdown="1">
```rust
let args = _ { width: 200, x: 15, .. };
crop(&img, args);
```
</div>

TODO(voice): clipped "That has limits." opener, ", and ..., which ..." chain

Its field values are checked without the field types and coerced once the struct is known. That has limits. A closure that needs its signature from the field type only works when the struct literal is written at the call, and reading `args.x` before `crop(&img, args)` needs a type annotation, which an anonymous struct can't have.

#### Structural records

So what about *structural records* in general? [RFC 2584][rfc-2584] proposed `{ red: 0xA8, green: 0x3C, blue: 0x09 }` as both an expression and a type, where any two records with the same fields are the same type. The lang team [closed it][rfc-2584-close] not because it was wrong but because it was not a priority, and scottmcm cautioned against a new fundamental kind of type that [reaches the type system, MIR, Miri, derives, proc macros, and every backend][scottmcm-2023]. Their suggestion was to take the lower-impact pieces first, [struct literals whose type is inferred from use][scottmcm-2021], borrowed from [Zig][zig-anon-struct], and unnameable *Voldemort types* instead of structural ones.

TODO(voice): bolded thesis, "left free for a structural literal", "without changing how anyone calls them" run-on

Anonymous structs and records follow these two pieces of advice. `_ { .. }` is an inferred struct literal, and an anonymous struct parameter or a `let` record is an unnameable struct (like a closure's type), so **none of them needs a new kind of type**. The only new rule is that a `let` record converts into other structs by field name. A bare `{ x: 1 }` is still an error, left free for a structural literal, and if records with the same fields someday become the same type, anonymous struct parameters can be redefined as structural records without changing how anyone calls them.

Records, and their conversions, cover a good part of what RFC 2584 wanted structural records for. They still aren't structural records. Each record is its own unnameable struct, and it only meets another struct through a conversion, which happens where the target type is known and moves the fields into a new value. That leaves gaps:

TODO(voice): bold-label limitation bullets, "Conversion moves, it doesn't borrow."

- **No type identity.** Two `let` records with the same fields are different types. `if c { r1 } else { r2 }` and `[r1, r2]` have no single type to convert into, and `r1 == r2` has no impl between them.
- **No way to refer to a record type.** `struct { .. }` in a parameter list declares a new anonymous struct every time it's written, and never names an existing one. So one function's anonymous struct can't be used as a struct field, a return type, or another function's parameter, and a library can only hand one out by converting it into a named struct.
- **Conversion moves, it doesn't borrow.** `&args` can't be passed as `&Rgb`, since the two structs' layouts aren't guaranteed to match, and a `Vec` of one record doesn't convert into a `Vec` of another. Generic code never reaches a conversion at all.
- **Few traits.** A `let` record derives only `Clone` and `Copy`. It has no `Debug`, `PartialEq`, `Hash`, or `Default`, and no user `impl`s, since there's no type to name.

TODO(voice): "something records leave out, not something they decide"; item 2 "could be retired the same way"

None of these block structural records later, because each is something records leave out, not something they decide:

- The bare `{ x: 1 }` syntax, for expressions, patterns, and types, is still free.
- A record's type can't be named, so, like a closure's, its identity is unspecified. Two records with the same fields can become the same type later without changing what any caller writes, and any `fn` pointer shim like the one sketched in [Function types](#function-types) could be retired the same way.
- Deriving more traits for records is additive.

#### Converting by field name

TODO(voice): "What records do decide is the conversion itself." framing opener, "either inherit that rule or break it"

What records do decide is the conversion itself. Once `let` records convert by field name, leaving behind fields the target lacks and filling in its defaults, structural records either inherit that rule or break it. [RFC 2584][rfc-2584-rendered] listed both larger-to-smaller and record-to-struct coercions as future possibilities, and warned that they "could also reduce robustness".

TODO(voice): "The robustness risk is ..." abstract opener, impersonal "The plan is", clipped "That catches the typo."

The robustness risk is a field that silently goes nowhere. A misspelled field in a `let` record, `_ { width: 200, hieght: Some(50) }` passed to `crop`, stays behind, and `height` takes its default. The plan is to treat that the way Rust already treats a variable nobody reads, so a field of a `let` record that no conversion takes is unused, and gets a warning. That catches the typo. It also catches a library renaming `height` to `h`, which is already a breaking change since field names are API, because the caller's `height` no longer goes anywhere. A field that some call does take isn't unused, so one record can still feed both `fill` and `crop`.

TODO(voice): "Records don't have that ambiguity.", "covers the silent part that's left"

Ruby went through a harder version of this. Its automatic conversion between a trailing hash and keyword arguments had enough corner cases, like a hash landing in a positional parameter or in the keywords depending on whether that parameter had a default, that [Ruby 3.0 separated them][ruby-3-kwargs] and made callers write `**` to pass a hash as keywords. Records don't have that ambiguity. A record only ever fills the parameter in its own position, and field names are checked at compile time. The unused-field warning covers the silent part that's left.

TODO(voice): one-sentence summary paragraph, "X, not Y", "settled"

The conversion rule, not the records' types, is what has to be settled together with structural records.

#### Named records?

Every record so far is unnameable, which is what leaves the second gap above. A function can't return one, a struct can't hold one, and two functions can't share one except through a named struct, which doesn't convert by field name. Should there be syntax for declaring a named record, a type that has a name but still converts by field name like a `let` record?

<div class="highlight-block" markdown="1">
```rust
// possible spellings
type CropArgs = struct { width: u32, height: Option<u32> = None };
#[record] struct CropArgs { width: u32, height: Option<u32> = None }
```
</div>

TODO(voice): "never something a caller can do", ends on "is open."

A named record gives up what a named struct's name protects, so it has to be the definer's choice, never something a caller can do to an existing struct. Whether it's worth new syntax, or should wait for structural records to make the type writable directly, is open.
  </div>
-->
</details>

For where this goes from here, see [What's Next][first-post-whats-next] in the
first post.

<!-- References -->

<!-- First post -->
[first-post]: 2026-10-19-anonymous-struct-parameters
[first-post-examples]: 2026-10-19-anonymous-struct-parameters#2-examples
[first-post-semver]: 2026-10-19-anonymous-struct-parameters#api-evolution-and-semver
[first-post-call]: 2026-10-19-anonymous-struct-parameters#call-syntax-fa-x-1
[first-post-defaults]: 2026-10-19-anonymous-struct-parameters#defaults-fa-x-1-
[first-post-botahamec]: 2026-10-19-anonymous-struct-parameters#botahamec-names-defaults-and-pub-apis
[first-post-whats-next]: 2026-10-19-anonymous-struct-parameters#6-whats-next

<!-- RFC pull requests -->
[rfc-2584]: https://github.com/rust-lang/rfcs/pull/2584

<!-- RFC texts -->
[rfc-2584-rendered]: https://github.com/Centril/rfcs/blob/rfc/structural-records/text/0000-structural-records.md
[rfc-3681-const]: https://rust-lang.github.io/rfcs/3681-default-field-values.html#on-const-contexts
[rfc-3681-non-const]: https://rust-lang.github.io/rfcs/3681-default-field-values.html#non-const-values

<!-- Comments on RFC issues and pull requests -->
[rfc-2584-close]: https://github.com/rust-lang/rfcs/pull/2584#issuecomment-810313148
[scottmcm-2021]: https://github.com/rust-lang/rfcs/pull/2584#issuecomment-794508491
[scottmcm-2023]: https://github.com/rust-lang/rfcs/pull/2584#issuecomment-1477166822
[cart-2026]: https://github.com/rust-lang/rust/issues/132162#issuecomment-4930535333
[scottmcm-2026]: https://github.com/rust-lang/rust/issues/132162#issuecomment-4930787638
[vultix-2026]: https://github.com/rust-lang/rust/issues/132162#issuecomment-4931304399

<!-- Issues and tracking issues -->
[rfcs-2416]: https://github.com/rust-lang/rfcs/issues/2416
[rust-132162]: https://github.com/rust-lang/rust/issues/132162
[rust-42764]: https://github.com/rust-lang/rust/issues/42764

<!-- Project goals -->
[const-traits-goal]: https://rust-lang.github.io/rust-project-goals/2026/const-traits.html

<!-- rustc pull requests -->
[rust-129514]: https://github.com/rust-lang/rust/pull/129514

<!-- Prototype -->
[struct-args-branch]: https://github.com/nixpulvis/rust/tree/struct-args+type-elision

<!-- Crates and crate source -->
[syn]: https://docs.rs/syn
[syn-3]: https://github.com/dtolnay/syn/releases/tag/3.0.0

<!-- Standard library -->
[ref-coercions]: https://doc.rust-lang.org/reference/type-coercions.html

<!-- Other languages -->
[ruby-3-kwargs]: https://www.ruby-lang.org/en/news/2019/12/12/separation-of-positional-and-keyword-arguments-in-ruby-3-0/
[zig-anon-struct]: https://ziglang.org/documentation/master/#Anonymous-Struct-Literals
