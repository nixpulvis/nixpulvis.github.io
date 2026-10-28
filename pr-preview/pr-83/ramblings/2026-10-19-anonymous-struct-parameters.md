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

But the point stands, Rust already has a named, reorderable, defaultable
construct... the `struct`. I'm not the only one who thinks so. In the 2020
[named arguments pre-RFC][pre-rfc-2020], CAD97 [argued][cad97-2020] that making
options structs more ergonomic was a better short-term goal than full named
arguments, and the [RFC][rfc-2964] it became was [closed][rfc-2964-close], with
the lang team asking for a roadmap first. scottmcm has since [pointed
out][scottmcm-2023-irlo] that named arguments only need an inferred struct
literal and a way to declare an anonymous struct in a function signature. This
follows that direction, directly. [RFC 3444][rfc-3444] already proposes an
inferred literal, but goes further than we need and also doesn't cover anonymous
structs in function signatures.

First we'll show [`#![feature(anonymous_struct_parameters)]`](#1-struct-args):

- `fn foo(a: struct { x: i32, y: i32 = 0 })`{:.new-syntax}: the type of an
  anonymous struct, usable in a parameter's type position

Which requires [`#![feature(struct_pattern_inference)]`](#struct-pattern-inference) to then call:
- `_ { x: 1, .. }`{:.new-syntax}: a struct literal or pattern whose type is inferred

These two features form the core of the struct argument proposal.

Then [`#![feature(anonymous_struct_pattern_elision)]`](#elided-struct-patterns) gives:
- `fn foo(struct { ref x: i32, y: i32 = 0 })`{:.new-syntax}: where the anonymous
  struct both defines the type and the pattern

And optionally, [`#![feature(parameter_type_elision)]`](#elided-parameter-types)
adds corresponding elision of types that are described by their pattern:
- `fn bar(Point { x, y })`{:.new-syntax}: binds `x` and `y` without writing `: Point`

Later we'll show [`#![feature(named_arguments)]`](#3-named-argument-sugar) for a
proposed syntax of what a full named argument sugar might look like:
- `fn baz(x: i32; named: &str)`{:.new-syntax}
- `baz(1, named: "hi");`{:.new-syntax}

---
* TOC
{:toc}
---

## 1. Struct Args

Today in Rust you can pass struct arguments in a number of ways. Given the
following struct:

```rust
struct Point {
    x: i32,
    y: i32,
}
```

You can use it in a function definition:

1. As any other parameter: `fn f(p: Point) {}`
2. With a pattern: `fn f(Point { x, y }: Point) {}`

The first form is just standard argument passing, which doesn't destructure the
named fields at all, while the second form destructures the named fields but
names the struct type twice.

I'll be introducing new syntax in stages, and noting all new syntax inside
highlighted code blocks.

### Anonymous structs in params

For a function argument without a named struct type to use, an anonymous struct
type can be declared inline with `struct { .. }`. This is the base feature,
`#![feature(anonymous_struct_parameters)]`, and it's an ordinary type in the
`pat: ty` parameter position:

<div class="highlight-block" markdown="1">
```rust
fn g(p: struct { x: i32, y: i32 }) {}
```
</div>

This only binds `p`, exactly as `p: Point` would. These new structs are
anonymous, just like anonymous closure types, e.g. `{closure@foo.rs:2:13:
2:21}`.

Other than lacking a name though, an anonymous struct parameter **is just a
`struct`**. The rules for their evaluation order, borrowing, patterns, and
defaults ([RFC 3681][rfc-3681]) are the ones Rust already has for structs, not
new ones.

Anonymous struct types have come up on internals [since
2016][pre-rfc-unnamed-struct-types], usually as *structural* types, with `{ red:
25i8, .. }` as both a type and an expression, and named arguments as one of the
motivations. When it came up [again in 2023][irlo-record-types], scottmcm
[replied][scottmcm-2023-irlo] with the point that named arguments don't need
structural types at all, just an inferred struct literal and "sugar to create an
anonymous type in a function definition", which is what's proposed here. [As
recently as 2025][irlo-anon-struct-2025], Josh Triplett [separated][josh-2025]
three ideas that usually get conflated. The first is structural anonymous types,
which seem unlikely to make progress soon, though we'll explore some of their
motivations in [record types (in the upcoming follow-up
post)][follow-up-records]. The second is [RFC 2102][rfc-2102]'s unnamed fields
([tracking issue][rust-49804]), which are about `#[repr(C)]` layout and not
directly related to this. Their syntax does line up though, RFC 2102 also
spells an anonymous struct type `struct { .. }`.

### Struct pattern inference

Since the struct has no name, `#![feature(struct_pattern_inference)]` lets you
bind its fields with `_ { .. }`, which is a struct pattern that infers its type.
It works anywhere a pattern does, so the fields can also be renamed, borrowed,
or skipped:

<div class="highlight-block" markdown="1">
```rust
fn g(_ { x, y }: struct { x: i32, y: i32 }) {}
fn g(_ { x: horizontal, .. }: struct { x: i32, y: i32 = 0 }) {}
let _ { x, y } = point;
```
</div>

Calling a function with an anonymous struct argument is just like calling it
with a named struct argument, except you can only use an `_` to refer to it, the
literal form of the same `_ { .. }`:

<div class="highlight-block" markdown="1">
```rust
g(_ { x: 1, y: 2 });
```
</div>

But nothing is stopping you from inferring the type of a named struct argument either.

<div class="highlight-block" markdown="1">
```rust
// recall: fn f(p: Point) {}
f(_ { x: 1, y: 2 });
```
</div>

Note that `_` never fills in the type of a parameter. `fn g(_ { x })` is an
error for the same reason `fn g(x: _)` is. This is because function signatures
don't infer their types.

Josh's third idea is passing a struct to a function without naming it. The `f(_
{ x: 1 })` form is the call half of what's proposed here. This idea is
materialized in the open [RFC 3444][rfc-3444], which proposes inferring a struct
literal's path (spelled `.{ .. }`). `binrw`'s maintainers [point
out][rfc-3444-binrw] that this plus field defaults (RFC 3681) would replace
[their `args!` macro][binrw-named-args] and its inference hacks. However, RFC
3444 proposes a more general "path inference" and focuses on inferring the path
of an `enum`, which we aren't concerned with to satisfy ergonomic struct
arguments.

<div class="aside" markdown="1">
I personally don't accept that `.{ }` is better, since `_` is already the
convention for inferring types, and I have no issue with `_::Variant` naming an
enum's variant. Maybe this point will prove to be contentious, but I don't see
how `.{ .. }` really fixes it smoothly. This is something to consider now, since
it might impact the broader design, but nothing we introduce here touches `enum`
today. I'm not convinced inferred `enum` variants couldn't be solved in a way
that works naturally with `_ { }`.
</div>

With inferred literals, default field values ([RFC 3681][rfc-3681]) carry over
directly:

<div class="highlight-block" markdown="1">
```rust
fn h(a: i32, _ { i, log }: struct { i: i32 = 1, log: bool = false }) {}
```
</div>

<div class="highlight-block" markdown="1">
```rust
h(1, _ { .. });
h(2, _ { i: 2, .. });
h(3, _ { i: set_log(), log: get_log() });
```
</div>

So while this doesn't give us default positional arguments, it does give us a way to define default fields for anonymous struct arguments.

### Elided struct patterns

So far `h` has had to name `i` and `log` twice, once to bind them in the
pattern and again to declare them in the type. Since we almost always want the
fields themselves, we can leave the pattern off.

`#![feature(anonymous_struct_pattern_elision)]` lets an anonymous struct type
act as its own pattern, binding each of its fields by name:

<div class="highlight-block" markdown="1">
```rust
fn h(a: i32, struct { i: i32 = 1, log: bool = false }) {}
```
</div>

This is the form we'll use for most of the rest of this post.

Fields are written just like parameters here, so they can use the same binding
modes, `mut`, `ref`, and `ref mut`:

<div class="highlight-block" markdown="1">
```rust
fn g(struct { mut x: i32, ref name: String }) {}
```
</div>

These only make sense when the type is also the pattern, so
`p: struct { mut x: i32 }` is an error.

A field can also be an anonymous struct itself, though how an elided pattern
should bind the nested fields is still an open question, which the upcoming
[follow-up post][follow-up-nested] will go into.

### Elided parameter types

Anonymous struct arguments don't actually need elided parameter types, and
neither does the named argument sugar later on, so this could just as well be
its own proposal. But once a type can imply its pattern, it's a natural time to
let a pattern imply its type too.

This also removes the duplication of `Point` from the start of this section.
With `#![feature(parameter_type_elision)]`, a parameter's type can be left out
when the pattern already names it.

```rust
fn f(Point { x, y }: Point) {}
```

With an elided struct type, it could be written as:

<div class="highlight-block" markdown="1">
```rust
fn f(Point { x, y }) {}
```
</div>

The rule is that a pattern which spells out a type can stand for both the
pattern and that type. Struct and tuple struct patterns contain the type in
their path names, a reference pattern has its referenced type, and tuples and
arrays work if their members spell out their types.

<div class="highlight-block" markdown="1">
```rust
fn f(p @ Point { x, .. })   // p @ Point { x, .. }: Point
fn f(Meters(m))             // Meters(m): Meters
fn f(&Point { x, y })       // &Point { x, y }: &Point
fn f((Point { x, y }, Meters(m)))  // (Point, Meters)
fn f([Meters(a), m])        // [Meters(a), m]: [Meters; 2]
```
</div>

This elision is purely syntactic since the type is basically just the pattern's
path copied over. Lifetimes can still be elided, e.g. `&Point { .. }` still
works just like `&Point` does, and generic arguments are written in the pattern
the way they would be in the type, e.g. `fn f<T>(Pair<T> { a, b })` (or with a
turbofish, `Pair::<T>`). A variant pattern like `Shape::Square(s)` names a
variant, not a type, so it can't elide anything.

### API evolution and SemVer

Since an anonymous struct's fields are named at every call, its field names are
part of the function's API, just like a public struct's. That changes which
edits to a function are breaking:

- **Adding a parameter** is always breaking for positional parameters, however
  adding a field *with a default* to an anonymous struct isn't breaking for
  callers that wrote `..`, with or without the named argument
  [sugar](#defaults-fa-x-1-)
- **Reordering** positional parameters is breaking, however reordering fields
  isn't since they're passed by name
- **Renaming** a positional parameter is free, since its name isn't part of its
  API, however renaming a field is breaking
- **Removing** either, or adding a field *without* a default, is breaking

For named structs, `#[non_exhaustive]` makes adding fields non-breaking, but
rather bluntly, since it forbids struct literals outside the defining crate
entirely, and [RFC 3681][rfc-3681] doesn't allow it on structs with default
field values. So a library has to choose between defaults that callers can use
with `..`, and adding fields without a breaking change. Anonymous structs have
no struct item to put `#[non_exhaustive]` on, so they don't even get that
choice. If the two are ever allowed together, anonymous structs could opt in, or
be non-exhaustive by default, meaning every literal from outside the crate has
to write `..`, even one that lists every field. Then adding a field with a
default would never be a breaking change.

## 2. Examples

Now let's look at some realistic examples of this new syntax. Later we'll see
how to make it look even more like named arguments with some [syntactic
sugar](#3-named-argument-sugar). I have a hacked-together prototype
implementation for every feature in this post, which the upcoming [follow-up
post][follow-up] will cover, but for now we'll stick to the ones from Section 1
and leave `named_arguments` for later.

### Unambiguous calls

Consider the [`std::fs::copy`][fs-copy] function, which has the following
signature:

```rust
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(
    from: P,
    to: Q,
) -> io::Result<u64>
```

Calling it can be ambiguous, since both arguments are just paths:

```rust
let a = Path::new("notes.txt");
let b = Path::new("notes.bak");
copy(a, b)?;
copy(b, a)?;  // oops, overwrites notes.txt
```

With an anonymous struct instead:

<div class="highlight-block" markdown="1">
```rust
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(struct {
    from: P,
    to: Q,
}) -> io::Result<u64>
```
</div>

Each call names its fields, so it's clear which path is `from` and which is
`to`:

<div class="highlight-block" markdown="1">
```rust
let a = Path::new("notes.txt");
let b = Path::new("notes.bak");
copy(_ { from: a, to: b })?;
copy(_ { from: b, to: a })?;  // clearly wrong
```
</div>

### Existing structs

`_ { .. }` isn't tied to anonymous structs. It works wherever the expected type
is a struct, so it works for every existing struct without changing anything:

<div class="highlight-block" markdown="1">
```rust
struct Rgb { red: u8, green: u8, blue: u8 }

fn fill(shape: &mut Shape, color: Rgb) {
    unimplemented!()
}

fill(&mut shape, _ { red: 0xA8, green: 0x3C, blue: 0x09 });

let accent = Rgb { red: 0x1E, green: 0x90, blue: 0xFF };
fill(&mut shape, _ { red: 0xFF, ..accent });

// wgpu's `InstanceDescriptor`
Instance::new(_ {
    backends: Backends::VULKAN,
    ..InstanceDescriptor::new_without_display_handle()
});
```
</div>

Keep in mind that `..` and `..Default::default()` are different things. A plain
`..` takes each omitted field's declared default ([RFC 3681][rfc-3681]), which
must be const, see [non-const defaults](#non-const-defaults), while
`..Default::default()` builds the struct's whole `Default` value and takes the
omitted fields from it. Neither is new here, they are just how structs already
work.

### Options structs

With default field values ([RFC 3681][rfc-3681]), an options struct no longer
needs the builder pattern, since `..` fills in every field the caller leaves
out:

```rust
pub struct CropOptions {
    pub width: u32,
    pub height: Option<u32> = None,
    pub x: u32 = 0,
    pub y: u32 = 0,
}

pub fn crop(img: &Image, options: CropOptions) -> Image {
    unimplemented!()
}

let img = Image::from_png("cat.png");
crop(&img, CropOptions { width: 200, .. });
crop(&img, CropOptions { width: 200, height: Some(200), .. });
crop(&img, CropOptions { y: 25, x: 15, width: 200, .. });
```

Today's calls name `CropOptions` every time, and with `_ { .. }` they wouldn't
have to. But it's still a whole separate type, with its own name and `pub` on
every field, just to pass a few arguments to one function. An anonymous struct
keeps the same fields and defaults, but puts them right in the signature:

<div class="highlight-block" markdown="1">
```rust
pub fn crop(img: &Image, struct {
    width: u32,
    height: Option<u32> = None,
    x: u32 = 0,
    y: u32 = 0,
}) -> Image {
    unimplemented!()
}

let img = Image::from_png("cat.png");
crop(&img, _ { width: 200, .. });
crop(&img, _ { width: 200, height: Some(200), .. });
crop(&img, _ { y: 25, x: 15, width: 200, .. });
```
</div>

If the options do end up needing a name, switching back to `CropOptions` doesn't
change the calls, since `_ { .. }` still works to build a named struct.

### Generics and lifetimes

Fields can use anything a parameter's type can, including generic parameters,
elided lifetimes, `impl Trait`, `Self`, and associated types:

<div class="highlight-block" markdown="1">
```rust
fn map<T, U>(v: Vec<T>, struct {
    f: impl Fn(&T) -> U,
    limit: usize = 10,
}) -> Vec<U> {
    unimplemented!()
}

map(vec![1, 2, 3], _ { f: |n| n * 2, .. });
map(vec![1, 2, 3], _ { f: |n| n.to_string(), limit: 2 });
```
</div>

This works the way closures do. A closure written inside `map` gets a type
that's generic over `map`'s generic parameters, and so does the anonymous
struct. Each instantiation of `map`, including the hidden type parameter that
`impl Fn` introduces, gets its own instantiation of the struct. Nothing about
the struct needs declaring, it just uses what the signature already has.

A lifetime in a field is the same as it would be in a positional parameter,
whether it's named like `'a` here, or elided like `&str` would be in `x: &str`.
Unlike a named struct, which would need `struct PickArgs<'a>`, the anonymous
struct parameter never declares lifetimes itself:

<div class="highlight-block" markdown="1">
```rust
fn pick<'a>(x: &'a str, struct {
    fallback: &'a str = "none",
}) -> &'a str {
    unimplemented!()
}

pick("", _ { .. });
pick("", _ { fallback: "empty" });
```
</div>

### Traits

Trait methods can take anonymous struct parameters just like any other method.
The trait owns the struct and its defaults, and each `impl` repeats the fields
with the same types, just like positional parameters. Calls through generics and
`dyn` work like any other method call:

<div class="highlight-block" markdown="1">
```rust
trait Writer {
    fn write(&mut self, struct {
        data: &'static str,
        times: usize = 1,
    });
}

impl Writer for Buffer {
    fn write(&mut self, struct {
        times: usize,
        data: &'static str,
    }) {
        unimplemented!()
    }
}

buffer.write(_ { data: "hi", .. });
let w: &mut dyn Writer = &mut buffer;
w.write(_ { data: "hi", times: 2 });
```
</div>

## 3. Named Argument Sugar

Everything so far is struct syntax in new places. The proposed syntax sugar here
is what looks like named arguments.

This is also the part I expect the most bikeshedding about. However, none of
this is required since everything in Sections 1 and 2 works without it. Most of
it is borrowed from other languages. The `name: value` arguments look like
[Swift][swift-labels]'s argument labels, and like Swift,
[Python][python-kwargs], and [Julia][julia-kwargs], named arguments can have
defaults. [Separating positional and keyword parameters with a
`;`](#signature-syntax-fn-fa-i32-x-i32) comes from Julia.

It does have costs though. Every call can be written two ways, and a `;`
signature obscures that the last parameter is a struct. Only the last argument
can be written with names, and `:` in argument lists is taken for good. I think
it's worth it, but these are fair concerns.

### Signature syntax: `fn f(a: i32; x: i32)`

A `struct { .. }` as the last parameter is noisy, especially in the common case
where it holds the function's options. Borrowing from [Julia][julia-kwargs]'s
keyword parameters, a `;` can end the positional parameters instead, and every
parameter after it becomes a field of one anonymous struct, which is passed as
the last argument. The `;` has its drawbacks though, which [`;` vs `*`](#-vs-)
gets into, and there may be a better syntax here I just can't think of.

With named argument sugar, `crop` and `copy` become:

<div class="highlight-block" markdown="1">
```rust
fn crop(
    img: &Image;
    width: u32,
    height: Option<u32> = None,
    x: u32 = 0,
    y: u32 = 0,
) -> Image

fn copy<P: AsRef<Path>, Q: AsRef<Path>>(; from: P, to: Q) -> io::Result<u64>
```
</div>

- The parameters after the `;` are fields, exactly as in a bare `struct { .. }`
  parameter. They're written `name: Type` with an optional `= default`, and can
  take the same binding modes, `mut`, `ref`, and `ref mut`. They're just names
  and not patterns, since a field's name is its API.
- There's at most one `;` and it declares the last struct parameter. Anonymous
  struct parameters anywhere else can still use `struct { .. }`.
- It works in traits and impls too, as in `fn write(&mut self; data: &'static
  str, times: usize = 1);`.
- It's purely syntax, so switching between `;` and `struct { .. }` changes
  nothing for callers.

### Call syntax: `f(a, x: 1)`

Named arguments are the other half of the `;`, though they fill a final struct
argument regardless of how it's declared. They are just sugar for a `_ { .. }`
literal as the last argument. Once an argument is written `name: value`, it and
every argument after it form the struct. The calls from [Section 2](#2-examples)
become:

<div class="highlight-block" markdown="1">
```rust
copy(from: a, to: b)?;

crop(&img, width: 200, ..);
crop(&img, width: 200, x: 15, y: 25, ..);

fill(&mut shape, red: 0xA8, green: 0x3C, blue: 0x09);
```
</div>

The `..` at the end of `crop`'s calls fills in the defaults. There are some more
details and questions about [Defaults](#defaults-fa-x-1-) below.

- Positional arguments come first, then named ones.
- There's no field shorthand. In a struct literal `Point { x, y }` means `Point
  { x: x, y: y }`, but `f(x, y)` is already two positional arguments, so it's
  `f(x: x, y: y)`, or `f(_ { x, y })`. That is, unless we make the `;` required
  before the struct argument at the call site as well, which would allow
  `f(; x, y)`.
- Named arguments are always the last argument, so they fill the last
  parameter. An anonymous struct anywhere else is passed with `_ { .. }`.

Positional parameters never become labels in this proposal. A parameter is a
pattern that may not have a name, like `(x, y): (i32, i32)` or `_: u8`, and
turning its name into API would make every rename a breaking change. So `foo(a:
1)` with `fn foo(a: i32)` is intentionally an error. The argument is either
passed by position, or the parameter moves into a struct.

The syntax happens to be free, since `expr: Type` in expressions (type
ascription) was removed, and `macro_rules!` can't write `$e:expr :`. It does
claim `:` in argument lists. `f(x: 1)` always means "the field `x` of the final
parameter struct of `f`".

### `;` vs `*`

Python's bare `*` for [keyword-only parameters][pep-3102], as in
`def f(a, *, b=1)`, is similar. Unlike Julia's and Python's though, the `;`
doesn't make a second kind of parameter, it's just another way to write a
struct.

Python's `*` does have one real advantage though... It's its own item in the
list, so in a long signature it's hard to miss, and adding or removing a
parameter on either side of it in a multi-line parameter list only touches that
parameter's line. The `;` takes the place of a comma instead, so in a signature
that doesn't fit on one line, like `crop` above, it ends the last positional
parameter's line. A short one stays on one line, `fn f(a: i32; x: i32)`.

I personally like `;` when used in a single line, but named arguments are likely
to be numerous, and split across lines. `*` works in languages like Python and
Ruby, which don't have the strong association of `*` with deref, but in Rust I
don't like it. I'm not sure what's better however.

### Defaults: `f(a, x: 1, ..)`

Named arguments end with `..` to fill in the defaults of any fields left out,
the same as a struct literal, or with `..base` to take them from another value.
Without the `..`, every field has to be given:

<div class="highlight-block" markdown="1">
```rust
crop(&img, width: 200, ..);
crop(&img, width: 200, height: None, x: 0, y: 0);

fill(&mut shape, red: 0xFF, ..accent);
```
</div>

The `..` is required, since it's what shows which fields were defaulted. Leaving
it off, so that named arguments imply it, would be closer to how other languages
do it, at the cost of visibility, which Rust tends to prefer.

A `..` on its own isn't a named argument though. `f(..)` and `f(..base)` are
already a `RangeFull` and a `RangeTo`, as in `v.drain(..)`. We could pick the
meaning by the type of the parameter it fills, so `f(..)` would mean
`f(_ { .. })` when that parameter is a struct, and stay a range otherwise.
Passing a range where a struct is expected is an error today, so no existing
call would change meaning, but `..` could no longer be read without knowing the
signature. This is a somewhat unfortunate existing conflation in Rust's syntax.

My upcoming [follow-up post][follow-up-implied-defaults] will go further into
implying `..` instead, whether named structs should get it too, and leaving out
a last struct whose fields all have defaults.

## 4. Relating to Recent Posts

The recent posts from the intro each take a different position on named
arguments in Rust. Endler argues structs already do the job, Botahamec wants
`pub` parameters with defaults, and Klabnik is open to labels but not defaults.
Each is worth answering on its own terms, so here's an overview of how this
design compares to them, with more details on each below.

|  | This design | Structs today ([Endler][endler]) | `pub` parameters ([Botahamec][botahamec]) | Names only ([Klabnik][klabnik]) |
| --- | --- | --- | --- | --- |
| Named parameters | Fields of a `struct { .. }` parameter, or parameters after a [`;`](#signature-syntax-fn-fa-i32-x-i32) | Fields of a named struct | Parameters marked `pub` | Labels on existing parameters |
| Reordering | Yes | Yes | Yes | Not settled |
| Defaults | `..` with RFC 3681 field defaults, or `..Default::default()` for named structs. Still undecided: [implying the `..`][follow-up-implied-defaults], and [optional fields][follow-up-optional], which I'm skeptical of | `..Default::default()` | May be non-const, omission implicit | No |
| Evaluation order | As written | As written | — | Raised as a problem |
| Patterns as parameters | Struct pattern | — | `pub name @ pat` | Raised as a problem |
| `fn` pointers | Names and defaults kept, distinct per function ([sharing](#function-types) when the fields match is an open question) | Names kept in the struct type | Names erased | Raised as a problem |
| Adding a parameter | Non-breaking if defaulted, for callers that wrote `..` | Non-breaking if defaulted, for callers that wrote `..Default::default()` | Non-breaking if defaulted | — |
| Per-function boilerplate | None | A named struct per argument bundle | None | None |
| New syntax | `struct { .. }` as a parameter type, `_ { .. }` as an expression and a pattern, elided struct patterns and parameter types, and the optional [sugar](#3-named-argument-sugar) for `name: value` arguments and `;` parameters | None | `pub` parameters, `name: value` arguments, defaults | `name: value` arguments |

Klabnik's post is a position post and not a proposal, so its column records the
problems it raises rather than answers. Macro crates like [bon] and [structx]
fill the same gap today, with a builder or a macro at each call.

<details id="botahamec-names-defaults-and-pub-apis">
  <summary markdown="1">
### Botahamec: names, defaults, and `pub` parameters
  </summary>
  <div markdown="1">
In ["Named and Optional Arguments are Awesome"][botahamec], Botahamec tours
Dart, C#, and TypeScript, then looks at what Rust does without them. `HashMap`
has eight constructors to cover three optional parameters.
`print(text, bold, italics, underline)` is easy to call with the flags swapped.
The options-struct workaround needs a type, a `Default` impl, and
`..Default::default()` at every call, and it falls apart when only some fields
have defaults.

Their proposal resembles [a 2016 pre-RFC][pre-rfc-2016]. A parameter marked
`pub` can be passed by name, and parameters may have defaults, which callers
can omit. A `pub` parameter can still be passed by position too.

`pub` is a poor fit for this, since it has nothing to do with visibility (a
private `fn` can have `pub` parameters, and `pub(crate) x: u32` has no sensible
meaning), and the post itself flags the keyword as an open question. But that's
a small point about a single keyword.

<div class="highlight-block red" markdown="1">
```rust
// Botahamec's proposal
fn print_labeled_measurement(pub value: i32, pub unit_label: char = 'm') { .. }

print_labeled_measurement(5);
print_labeled_measurement(5, 'h');
print_labeled_measurement(value: 5, unit_label: 'h');
print_labeled_measurement(unit_label: 'h', value: 5);
```
</div>

With an anonymous struct, the same function's named parameters become fields,
which are only ever passed by name.

<div class="highlight-block" markdown="1">
```rust
// anonymous struct
fn print_labeled_measurement(struct { value: i32, unit_label: char = 'm' }) { .. }

print_labeled_measurement(_ { value: 5, .. });
print_labeled_measurement(_ { value: 5, unit_label: 'h' });
print_labeled_measurement(_ { unit_label: 'h', value: 5 });

// with the named argument sugar
print_labeled_measurement(value: 5, ..);
print_labeled_measurement(unit_label: 'h', value: 5);
```
</div>

There's no equivalent of `print_labeled_measurement(5, 'h')`, which is
intentional. Like C#, Botahamec's proposal lets a named parameter still be
passed by position. With anonymous structs, a parameter is either positional or
a field, and a field can only be passed by name, whether that's in a `_ { .. }`
literal or with the named argument sugar.

That's why existing functions can't be backported, which Botahamec counts
against struct name inference. A function's positional parameters stay
positional, and taking names means moving them into a struct, which is a
breaking change like adding any other parameter.

Botahamec also forbids defaults on trait methods, since it's unclear whether
the trait or the impl should own them, and only the trait's would work through
`dyn`. Here the trait owns them, and calls through generics and `dyn` use them
like any other ([Section 2](#traits)).

Their `HashMap` example replaces the eight constructors with one function,
quoted here as written. It doesn't address generic parameters yet. `S` and `A`
are never declared, and a default like `A = Global` only typechecks when `A` is
`Global`, so the post sets the problem aside for its "Future possibilities"
section.

<div class="highlight-block red" markdown="1">
```rust
// Botahamec's proposal, generic parameters ignored
pub fn new(
    pub capacity: usize = 0,
    pub alloc: A = Global,
    pub hasher: S = RandomState,
);
```
</div>

An anonymous struct has to address that problem now. It can still replace the
eight constructors with one function, though the hasher and the allocator
become `Option`s that the body fills in:

<div class="highlight-block" markdown="1">
```rust
// anonymous struct
fn new_map<K, V, S: BuildHasher + Default, A: Allocator + Default>(struct {
    capacity: usize = 0,
    hasher: Option<S> = None,
    alloc: Option<A> = None,
}) -> HashMap<K, V, S, A> {
    HashMap::with_capacity_and_hasher_in(
        capacity,
        hasher.unwrap_or_default(),
        alloc.unwrap_or_default(),
    )
}

let m: HashMap<u8, u8> = new_map(_ { capacity: 16, .. });
```
</div>

The bigger difference is how a parameter opts in. Botahamec's proposal needs a
keyword for it and anonymous structs don't. A parameter opts in by being a
field, and visibility isn't overloaded to mean a parameter is a named argument.

</div>
</details>

<details>
  <summary markdown="1">
### Klabnik: open-minded concerns
  </summary>
  <div markdown="1">
In ["Arguing about arguments"][klabnik], Steve Klabnik is open to labels on
existing parameters, `foo(x: 5, y: 6)` for `fn foo(x: i32, y: i32)`, but not to
optional or default arguments, because they hide what a call passes. They also
list what makes named arguments hard in Rust specifically. Parameters are
patterns and may not have a name, `fn` pointers erase names, trait impls can
rename parameters, reordering arguments raises the question of evaluation order,
and every parameter name quietly becomes part of the API.

Klabnik's objection to defaults is fair, and the answer is that making names
explicit is what lets defaults be safe. Fields are named only because the
function put them in a struct, and they can only be passed by name, so there's
no positional form to swap and no argument left out of the middle of a list,
`f(a, c)` for `f(a, b = 0, c)`. Defaults are owned by the function
(by the trait, for a trait method), and a call that relies on them says so
with `..`, with or without the [sugar](#defaults-fa-x-1-). Adding a parameter
is also safe, since a new field with a default doesn't break callers that wrote
`..`.

Where Botahamec opts parameters in with `pub`, Klabnik doesn't opt them in at
all, and that's the bigger difference. If every parameter can be labeled, the
function's author can no longer decide how an argument is passed. Labels would
have to be optional for existing calls to keep compiling, so `copy(a, b)` still
compiles, and there's no way to require names for arguments that are confusing
by position, like `from` and `to`. That's the whole point of naming them. With
anonymous structs the author decides. Positional parameters are passed by
position, fields are passed by name, and the signature shows where one ends and
the other begins, so `copy(_ { from: a, to: b })`, or `copy(from: a, to: b)`
with the sugar, is the only way to call `copy`.

The other problems are secondary. Every parameter name in every published crate
would become API after the fact, including names chosen only for the function
body, like `buf`, `_x`, hygienic macro names, and bindgen's `arg1`, so renaming
any of them would break someone. Each parameter would also get two APIs, since
reordering breaks positional callers and renaming breaks labeled ones, and
mixed calls would need new rules for which slot a positional argument fills
after a label.

The rest of Klabnik's list already has answers, because each has an existing
struct rule. None of these are new to Rust. Here's how anonymous structs handle
each one.

#### Evaluation order
{:.no_toc}

Fields are evaluated in the order written at the call site, as in any struct
literal. So this compiles:

<div class="highlight-block" markdown="1">
```rust
fn consume(struct { data: Vec<u8>, length: usize }) { .. }

consume(_ { length: data.len(), data: data });
```
</div>

and swapping the two is a use-after-move error, exactly as it would be for a
named struct:

<div class="highlight-block" markdown="1">
```rust
consume(_ { data: data, length: data.len() });
// error[E0382]: borrow of moved value: `data`
```
</div>

#### Patterns as parameters
{:.no_toc}

Klabnik points out that a Rust parameter is a pattern, not a name.
`fn foo((x, y): (i32, i32))` introduces two bindings, so there's no single name
to label the argument with, short of inventing separate external and internal
names.

Anonymous structs don't need either. Only fields are passed by name, and a
field always has one, which is its label at the call. With an [elided
pattern](#elided-struct-patterns) it's also the binding in the body, and when
the body wants a different name, an explicit `_ { .. }` pattern renames it like
any struct pattern. Positional parameters keep their patterns, since they're
never passed by name.

<div class="highlight-block" markdown="1">
```rust
fn foo((x, y): (i32, i32), struct { scale: i32 = 1 }) -> i32 { .. }
fn bar((x, y): (i32, i32), _ { scale: s }: struct { scale: i32 = 1 }) -> i32 { .. }

foo((1, 2), _ { scale: 2 });
foo((1, 2), _ { .. });
```
</div>

#### Defaults
{:.no_toc}

Defaults are RFC 3681 defaults. They're declared on the field, and used when
the caller writes `..`. They can use the function's type and const parameters
(`fn foo<const A: usize>(struct { bar: usize = A })`), and associated consts
through their bounds (`fn foo<T: HasN>(struct { n: usize = T::N })`).

#### Renaming hazards
{:.no_toc}

A field's name is part of the function's API, the same as a public struct field
([Section 1](#api-evolution-and-semver)). Positional parameters are unaffected,
so a function opts in, one parameter at a time, by moving it into the struct.
Nothing becomes API by accident, and a typo like `widht` gets the same error,
with the same suggestion of `width`, as it would in any struct literal.

#### Trait impls
{:.no_toc}

An impl can't rename fields. It repeats the trait's fields by name, in any
order, and a renamed field is an error ([Section 2](#traits)). It has to be
that way, since callers through generics and `dyn` only see the trait, so the
trait's field names are the ones they write.

#### `fn` pointers
{:.no_toc}

A function used as a value keeps its anonymous structs as parameter types, so
names survive through the pointer, and so do defaults:

<div class="highlight-block" markdown="1">
```rust
fn foo(struct { a: i32, b: i32 = 10 }) -> i32 { .. }

let g: fn(_) -> i32 = foo;
g(_ { b: 2, a: 4 });
g(_ { a: 5, .. });
```
</div>

Two functions with the same fields don't share a pointer type ([Section
5](#function-types)), though the upcoming [follow-up
post][follow-up-function-types] will sketch one way they might.

  </div>
</details>

<details>
  <summary markdown="1">
### Endler: structs already do this
  </summary>
  <div markdown="1">

In ["We Have Named Arguments at Home"][endler], Matthias Endler argues the job
is already done. A struct gives you names, any order, typo checking, per-field
docs, and defaults, and it's a real type you can store, validate, and pass
along. Names belong to a type, not to the calling convention.

That's the same premise anonymous structs start from, that named arguments are
just structs. Anonymous structs are convenience on top of Endler's model, not a
replacement for it. The function still takes a struct, with all of those
properties, and the anonymous struct and the inferred literal only remove the
ceremony of declaring it and naming it at every call. It's the
have-your-cake-and-eat-it-too approach, with the semantics of structs and
without the boilerplate.

Endler's `crop_imm` example, borrowed from Klabnik, declares a struct just for
this one function:

<div class="highlight-block other" markdown="1">
```rust
// Endler
struct Crop {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn crop_imm<I: GenericImageView>(
    image: &I,
    crop: Crop,
) -> SubImage<&I> { .. }

crop_imm(&img, Crop { x: 10, y: 20, width: 200, height: 100 });
```
</div>

That's the case anonymous structs were made for:

<div class="highlight-block" markdown="1">
```rust
// anonymous struct
fn crop_imm<I: GenericImageView>(image: &I, struct {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}) -> SubImage<&I> { .. }

crop_imm(&img, _ { x: 10, y: 20, width: 200, height: 100 });
```
</div>

When the arguments need to be a real type, for invariants, for storing, or for
passing along, declare one and use the elided type from [Section
1](#elided-parameter-types), as in
`fn crop_imm<I: GenericImageView>(image: &I, Crop { x, y, width, height })`.
Callers don't change, since `_ { .. }` builds either one. The call Endler's
post [imagines][endler-composition] a pile of language features for gets close
without any sugar. It basically just needs `_ { .. }` around the options and
real syntax for the duration and headers:

<div class="highlight-block" markdown="1">
```rust
fn request(url: impl Into<Url>, struct {
    timeout: Option<Duration> = None,
    redirects: bool = true,
    headers: Vec<Header> = Vec::new(),
}) -> Result<Response, Error> { .. }

request("/hello", _ {
    timeout: Some(Duration::from_secs(5)),
    redirects: false,
    headers: vec![
        Header("Accept", "application/json"),
        Header("X-Foo", "bar"),
    ],
})?;
request("/hello", _ { .. })?;
```
</div>

With the named argument [sugar](#call-syntax-fa-x-1) from Section 3, the
`_ { .. }` goes away too, and the call has the shape Endler imagines.

Endler also credits field-init shorthand for removing the `foo=foo` repetition
Klabnik points out. `_ { timeout, redirects }` keeps it, though the named
argument sugar has no shorthand of its own.

  </div>
</details>

### Where they agree

Side by side, the posts agree on more than it might seem at first. Everyone is
trying to make names at the call site work. Botahamec's objection to the options
struct isn't its meaning but its ceremony. In their words, "I don't see much
reason to name a new struct if it's only going to be used for the one function."
Endler's structs already answer most of Klabnik's hard problems, because struct
literals already have rules for names, order, and evaluation.

That leaves one cost, declaring and naming a struct for every function. It's why
the pattern shows up in public APIs and almost never in private code. Anonymous
structs take Endler's model and remove that cost, which is mostly what Botahamec
is asking for. Klabnik's objection to defaults is the one real potential
disagreement, and making the named parameters explicit helps, I think.

That's why I think structs are the right direction, rather than a separate named
argument feature. Everything here is built from parts Rust already has, struct
literals, struct patterns, and RFC 3681 defaults, with the struct's name
inferred and its declaration moved into the signature. Each post gets what it
asks for without giving up what the others care about. Endler's structs stay
real types, Botahamec loses the boilerplate, and Klabnik can see the defaults at
the call, since struct literals and named arguments mark them with `..`. Names,
order, evaluation, and API stability all follow rules Rust users already know
from struct literals, instead of a second set of rules for arguments. And it
doesn't close any doors, since the sugar from Section 3 is optional and can wait
until the base feature is accepted and tested. I also think it will gracefully
continue into structural records, if/when we ever want to tackle that.

## 5. Limitations and Open Questions

These will be covered in more detail, along with the prototype's own gaps, in
the upcoming [follow-up post][follow-up-limitations].

### Non-const defaults

Field defaults are RFC 3681 defaults and must be const. A default can't call
`RandomState::new()`, or `S::default()` through a bound, so a field like that
becomes an `Option` the body fills in. This is a limit of RFC 3681, not of
anonymous structs, and its [tracking issue][rust-132162] already lists non-const
values as an open question.

### Function types

A function used as a value keeps its anonymous struct as a parameter type, so
two functions whose structs have the same fields still have different types, and
`if c { returns_anon_a } else { returns_anon_b }` doesn't compile.

### Macros

Most procedural macros parse with [`syn`][syn], which expects every parameter to
be `pat: ty`. The core features keep that shape, so `syn` just needs the new
syntax, as it got `= default` fields in [3.0][syn-3]. The elision features and
`named_arguments` change the shape, which means macros that read each parameter,
like `#[tracing::instrument]`, need updating too.

`macro_rules!` is subtler. `_ { .. }` changes what `pat`, `expr`, and `stmt`
fragments match, so existing macros can pick a different arm, even in crates
that don't enable `struct_pattern_inference`. That makes it the only part of
the proposal that needs an edition. The upcoming [follow-up][follow-up-macros]
will have the details.

### Tooling

rust-analyzer and tree-sitter-rust have their own parsers and need updating
separately. rustfmt uses rustc's parser, so it mostly works already.

## 6. What's Next

- **An MCP or RFC.** The 2020 named arguments RFC was [closed][rfc-2964-close]
  because the lang team wanted to see "what the roadmap looks like towards the
  eventual full solution" before an incremental step. This is meant to be that
  roadmap, with struct literals and inferred struct names. We can add named
  argument sugar on top too.
- **The prototype.** Every feature here is implemented in a rustc prototype,
  which the upcoming [follow-up post][follow-up] will walk through, along with
  the open questions and the more speculative ideas, optional fields and record
  types, that this post intentionally leaves out.
- **Follow-ups to [RFC 3681][rfc-3681].** These are limits of default field
  values ([tracking issue][rust-132162]), not of anonymous structs, so they
  belong in follow-ups to that RFC, and anonymous and named structs keep the
  same rules.
  - **Non-const defaults.** `hasher: S = S::default()` needs a non-const
    default. The tracking issue already has it as an open question ([Section
    5](#non-const-defaults)).
  - **API evolution.** RFC 3681 doesn't allow `#[non_exhaustive]` on structs
    with default field values. If the two are ever allowed together, anonymous
    structs could opt into `#[non_exhaustive]`, or be non-exhaustive by default,
    with every literal from outside the crate required to write `..`. Adding a
    field with a default would then never be breaking ([Section
    1](#api-evolution-and-semver)).
- **Follow-up posts.** Several of the threads cited here are subsumed by this
  proposal or depend on how it goes, and each should get a comment pointing to
  it.
  - **[rfcs#323][rfcs-323], named arguments.** The wishlist issue is still
    open. Anonymous struct parameters with the [named argument
    sugar](#3-named-argument-sugar) would subsume it.
  - **[RFC 3444][rfc-3444], inferred types.** `_ { .. }` is the struct half of
    its path inference, spelled `_` instead of `.`. The two need one syntax, and
    this post's struct half doesn't need to wait for enums to be implemented.
  - **[RFC 2584][rfc-2584], structural records.** It's closed, but a revival
    would inherit the `let` record conversion rule from [record
    types][follow-up-records] in the upcoming follow-up, and anonymous struct
    parameters could become structural records without changing any caller (I
    think).

<div class="highlight-block" markdown="1">
```rust
fn send_feedback(struct {
    from: impl TryInto<User, Error = InvalidEmail>,
    to: impl TryInto<User, Error = InvalidEmail>,
    message: &str,
    mood: Mood,
}) -> Result<(), InvalidEmail>;

send_feedback(_ {
    from: "you@example.com",
    to: "nathan@nixpulvis.com",
    message: "
      Named arguments are dumb, Rust shouldn't even allow
      multiple arguments. Everything should be curried:
      
      λ(x, y).e := λx.λy.e
    ",
    mood: Mood::Angry,
})?;
```
</div>

Seriously though, I'd like to hear what you think, where this breaks, what APIs
it doesn't fit, or rules that surprise you, over on the [internals.rust-lang.org
thread][internals-thread].

<!-- References -->

<!-- Discussion -->
[internals-thread]: https://internals.rust-lang.org/t/named-arguments-are-just-structs-anonymous-struct-parameters/24683

<!-- Blog posts -->
[botahamec]: https://botahamec.dev/named-optional-args
[klabnik]: https://steveklabnik.com/writing/arguing-about-arguments/
[endler]: https://corrode.dev/blog/named-arguments-at-home/
[endler-composition]: https://corrode.dev/blog/named-arguments-at-home/#composition-is-a-superpower

<!-- RFC pull requests -->
[rfc-2584]: https://github.com/rust-lang/rfcs/pull/2584
[rfc-2964]: https://github.com/rust-lang/rfcs/pull/2964
[rfc-3444]: https://github.com/rust-lang/rfcs/pull/3444
[rfc-3681]: https://github.com/rust-lang/rfcs/pull/3681

<!-- RFC texts -->
[rfc-2102]: https://rust-lang.github.io/rfcs/2102-unnamed-fields.html

<!-- Follow-up post -->
[follow-up]: 2026-09-29-anonymous-struct-parameters-prototype
[follow-up-nested]: 2026-09-29-anonymous-struct-parameters-prototype#nested-anonymous-structs
[follow-up-macros]: 2026-09-29-anonymous-struct-parameters-prototype#macros
[follow-up-limitations]: 2026-09-29-anonymous-struct-parameters-prototype#2-limitations-and-open-questions
[follow-up-function-types]: 2026-09-29-anonymous-struct-parameters-prototype#function-types
[follow-up-implied-defaults]: 2026-09-29-anonymous-struct-parameters-prototype#implied-defaults
[follow-up-optional]: 2026-09-29-anonymous-struct-parameters-prototype#optional-fields
[follow-up-records]: 2026-09-29-anonymous-struct-parameters-prototype#record-types

<!-- Comments on RFC issues and pull requests -->
[rfc-2964-close]: https://github.com/rust-lang/rfcs/pull/2964#issuecomment-671545486
[rfc-3444-binrw]: https://github.com/rust-lang/rfcs/pull/3444#issuecomment-5626022100
[nixpulvis-2016]: https://github.com/rust-lang/rfcs/issues/323#issuecomment-213065086

<!-- Issues and tracking issues -->
[rfcs-323]: https://github.com/rust-lang/rfcs/issues/323
[rust-49804]: https://github.com/rust-lang/rust/issues/49804
[rust-132162]: https://github.com/rust-lang/rust/issues/132162

<!-- Internals forum threads -->
[pre-rfc-2016]: https://internals.rust-lang.org/t/pre-rfc-named-arguments/3831
[pre-rfc-2020]: https://internals.rust-lang.org/t/pre-rfc-named-arguments/12730
[cad97-2020]: https://internals.rust-lang.org/t/pre-rfc-named-arguments/12730/47
[pre-rfc-unnamed-struct-types]: https://internals.rust-lang.org/t/pre-rfc-unnamed-struct-types/3872
[irlo-record-types]: https://internals.rust-lang.org/t/record-types/18258
[irlo-anon-struct-2025]: https://internals.rust-lang.org/t/is-the-anonymous-struct-unnamed-struct-still-in-progress/23592
[scottmcm-2023-irlo]: https://internals.rust-lang.org/t/record-types/18258/11
[josh-2025]: https://internals.rust-lang.org/t/is-the-anonymous-struct-unnamed-struct-still-in-progress/23592/3

<!-- Crates and crate source -->
[syn]: https://docs.rs/syn
[syn-3]: https://github.com/dtolnay/syn/releases/tag/3.0.0
[bon]: https://docs.rs/bon
[structx]: https://docs.rs/structx
[binrw-named-args]: https://github.com/jam1garner/binrw/blob/585d48134e7b71be74d8b81b54cdbd07f6f1a1c4/binrw/src/named_args.rs

<!-- Standard library -->
[fs-copy]: https://doc.rust-lang.org/std/fs/fn.copy.html

<!-- Other languages -->
[swift-labels]: https://docs.swift.org/swift-book/documentation/the-swift-programming-language/functions/#Function-Argument-Labels-and-Parameter-Names
[python-kwargs]: https://docs.python.org/3/tutorial/controlflow.html#keyword-arguments
[julia-kwargs]: https://docs.julialang.org/en/v1/manual/functions/#Keyword-Arguments
[pep-3102]: https://peps.python.org/pep-3102/
