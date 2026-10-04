---
layout: rambling
title: Anonymous Struct Parameters
draft: true
---

There have been a few [recent posts](#3-relating-to-recent-posts) on how Rust
should adopt _named arguments_, and I even had some thoughts on the matter back
in [a comment I left on the issue in 2016][nixpulvis-2016]. My original idea was
never really something I expected to be adopted, but my point was always that
named arguments are just structs, which is even more true now than back then
given [default field values (RFC 3681)][rfc-3681]. It tried to show a direct
correspondence between positional arguments with tuples and named arguments with
structs, which was cute, but ultimately not good syntax.

But the point stands, Rust already has a named, reorderable, defaultable
construct... the `struct`. I'm not the only one who thinks so. scottmcm has
[pointed out][scottmcm-2023-irlo] that named arguments only need an inferred
struct literal and a way to declare an anonymous struct in a function signature,
and [RFC 3444][rfc-3444] already proposes an inferred literal, but goes further
than we need.

So let's talk about struct args instead of named arguments specifically for a
minute. For this idea, I'll be introducing new syntax in stages, and noting all
new syntax inside highlighted code blocks.

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
2. With a pattern: `fn f(p @ Point { x, y }: Point) {}`

The first form is just standard argument passing, which doesn't destructure the named fields at all. While the second form destructures the named fields but requires both using a somewhat lesser known `@` pattern and a named struct type (twice).

### Elided struct types

We could avoid the duplication of `Point` in the second example simply by allowing it to be elided entirely when it can be inferred from the pattern.

E.g.

```rust
fn f(p @ Point { x, y }: Point) {}
```

Could be written as:

<div class="highlight-block" markdown="1">
```rust
fn f(p @ Point { x, y }) {}
```
</div>

Technically elided struct types in function signatures aren't needed for the
following anonymous struct feature, but they feel like a natural extension of
the language at this moment since both require letting the type be inferred from
the pattern.


### Anonymous structs

Furthermore, for a function argument without a struct type to use yet, an anonymous struct could be defined inline.

E.g.

<div class="highlight-block" markdown="1">
```rust
fn g(p: _ { x: i32, y: i32 }) {}
```
</div>

Which could be thought of essentially as:

```rust
fn g(p @ {struct@...} { x, y }: {struct@...}) {}
```

These new structs are anonymous, just like anonymous closure types, e.g.
`{closure@foo.rs:2:13: 2:21}`.

There's already some existing motivation for `p @ Point` to be equal to `p:
Point` since `rustdoc` will report the previous `f` as `f(p: Point)`.

The binding is optional. Without it, the fields are bound directly, which is the
form the rest of this post mostly uses:

<div class="highlight-block" markdown="1">
```rust
fn g(_ { x: i32, y: i32 }) {}
```
</div>

Which could be thought of as:

```rust
fn g({struct@...} { x, y }: {struct@...}) {}
```

Calling a function with an anonymous struct argument is just like calling it
with a named struct argument, except you can only use an `_` to refer to it:

<div class="highlight-block" markdown="1">
```rust
g(_ { x: 1, y: 2 });
```
</div>

But nothing is stopping you from inferring the type of a named struct argument either.

E.g.

<div class="highlight-block" markdown="1">
```rust
// recall: fn f(p: Point) {}
f(_ { x: 1, y: 2 });
```
</div>

Anonymous struct types have come up on internals [since 2016][pre-rfc-unnamed-struct-types],
usually as *structural* types, with `{ red: 25i8, .. }` as both a type and an
expression, and named arguments as one of the motivations. When it came up
[again in 2023][irlo-record-types], scottmcm [replied][scottmcm-2023-irlo] with
the point mentioned in the intro, that named arguments don't need structural
types at all, just an inferred struct literal and "sugar to create an anonymous
type in a function definition", which is what's proposed here. [As recently as
2025][irlo-anon-struct-2025], Josh Triplett [separated][josh-2025] three ideas
that usually get conflated. The first is structural anonymous types, which seem
unlikely to make progress. The second is [RFC 2102][rfc-2102]'s unnamed fields
([tracking issue][rust-49804]), which are about `#[repr(C)]` layout and
unrelated to this. The third is passing a struct to a function without naming
it, `f(_ { x: 1 })`, which is the call half of what's proposed here.

That last idea is [RFC 3444][rfc-3444] from the intro, which proposes inferring a
struct literal's path (currently spelled `.{ .. }`), and binrw's maintainers [point
out][rfc-3444-binrw] that it plus field defaults would replace [their `args!`
macro][binrw-named-args] and its inference hacks. However, that RFC proposes a
more general "path inference" and focuses on inferring the path of an `enum`
which we aren't concerned with to satisfy ergonomic struct arguments.

<div class="aside" markdown="1">
I personally don't accept that `.{ }` is better, since `_` is already the
convention for inferring types, and I also have no issue with `_::Variant`
naming an enum's variant. Maybe this point will prove to be contentious, but I
don't see how `.{ .. }` really fixes it smoothly. This is a point to consider
now, since it might impact the broader design, but nothing we introduce here
touches `enum` today. I'm not convinced inferred `enum` variants couldn't be
solved in a way that works naturally with `_ { }`.
</div>

Regardless of the exact syntax though, an anonymous struct parameter **is just a
`struct`**. Evaluation order, borrowing, patterns, and defaults ([RFC
3681][rfc-3681]) follow rules Rust already has instead of new ones.

So we get this for free:

<div class="highlight-block" markdown="1">
```rust
fn h(a: i32, _ { i: i32 = 1, log: bool = false }) {}
h(1, _ { .. });
h(2, _ { i: 2, .. });
h(3, _ { i: set_log(), log: get_log() });
```
</div>

So while this doesn't give us default positional arguments, it does give us a way to define default fields for anonymous struct arguments.

### API evolution and SemVer

Since an anonymous struct's fields are named at every call, its field names are
part of the function's API, just like a public struct's. That changes which
edits to a function are breaking:

- **Adding a parameter** is always breaking for positional parameters, however
  adding a field *with a default* to an anonymous struct isn't breaking for
  callers that wrote `..`
- **Reordering** positional parameters is breaking, however reordering fields
  isn't since they're passed by name
- **Renaming** a positional parameter is free, since its name isn't part of its
  API, however renaming a field is breaking
- **Removing** either, or adding a field *without* a default, is breaking

For named structs, `#[non_exhaustive]` makes adding fields non-breaking, but
bluntly, since it forbids struct literals outside the defining crate entirely,
and [RFC 3681][rfc-3681] doesn't allow it on structs with default field values.
So a library has to choose between defaults that callers can use with `..`, and
adding fields without a breaking change. Anonymous structs have no struct item
to put `#[non_exhaustive]` on, so they don't even get that choice. If the two are
ever allowed together, anonymous structs could opt in, or be non-exhaustive by
default, meaning every literal from outside the crate has to write `..`, even
one that lists every field, as named argument [sugar](#call-syntax-fa-x-1)
already implies. Then adding a field with a default would never be breaking
([Section 8](#8-whats-next)).

## 2. Examples

Now let's look at some realistic examples of using this new syntax. Later we'll
see if/how we can make it even shorter with some syntactic sugar in [Section
4](#4-sugar). I have a hacked together [prototype
implementation](#5-the-prototype) for both the `struct_args` and
`struct_args_sugar` features, but for now we'll focus on the core
`struct_args` feature.

### Unambiguous calls

Consider the [`std::fs::copy`][fs-copy] function, which has the following signature:

```rust
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(
    from: P,
    to: Q,
) -> io::Result<u64>
```

Calling this function can be ambiguous, as both parameters have the same type.

```rust
let a = Path::new("notes.txt");
let b = Path::new("notes.bak");
copy(a, b);
copy(b, a);  // oops, overwrites notes.txt
```

Now consider if we used an anonymous struct instead:

<div class="highlight-block" markdown="1">
```rust
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(_ {
    from: P,
    to: Q,
}) -> io::Result<u64>
```
</div>

Now we can call `copy` with an anonymous struct which names the fields and makes it clear which argument is `from` and which is `to`:

<div class="highlight-block" markdown="1">
```rust
let a = Path::new("notes.txt");
let b = Path::new("notes.bak");
copy(_ { from: a, to: b })?;
copy(_ { from: b, to: a })?;  // clearly wrong
```
</div>

### Options without a builder

Anonymous struct parameters with defaults can avoid the builder pattern.
Sometimes your function needs some options, but you don't want to write a
complete builder implementation for it. Anonymous struct with `..` filling in
the remaining fields from their defaults can be a simpler solution for these use
cases.

<div class="highlight-block" markdown="1">
```rust
fn crop(img: &Image, _ {
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

### Existing structs

`_ { .. }` isn't tied to anonymous structs. It works wherever the expected type is a struct, so every existing options-struct API gets it without changing:

<div class="highlight-block" markdown="1">
```rust
struct Rgb { red: u8, green: u8, blue: u8 }

fn fill(shape: &mut Shape, color: Rgb) {
    unimplemented!()
}

fill(&mut shape, Rgb { red: 0xA8, green: 0x3C, blue: 0x09 });
fill(&mut shape, _ { red: 0xA8, green: 0x3C, blue: 0x09 });

let accent = Rgb { red: 0x1E, green: 0x90, blue: 0xFF };
fill(&mut shape, _ { red: 0xFF, ..accent });

// wgpu's `InstanceDescriptor`
Instance::new(_ { backends: Backends::VULKAN, ..InstanceDescriptor::new_without_display_handle() });
```
</div>

Keep in mind that `..` and `..Default::default()` are different things. A plain
`..` takes each omitted field's declared default (RFC 3681, which must be const,
see [non-const defaults](#non-const-defaults)). While
`..Default::default()` builds the struct's whole `Default` value and takes the
omitted fields from it. This is not new to this proposal, it's simply how
structs work.

### Generics and lifetimes

Fields can use anything in scope in the signature: including generic parameters,
elided lifetimes, `impl Trait`, `Self`, and associated types. A lifetime in a
field means what it would in a positional parameter. An elided one, like
`&str`, is elided as it would be in `x: &str`. Unlike a named struct, which
would need `struct PickArgs<'a>`, the anonymous struct never declares lifetimes
itself.

<div class="highlight-block" markdown="1">
```rust
fn map<T, U>(v: Vec<T>, _ {
    f: impl Fn(&T) -> U,
    limit: usize = 10,
}) -> Vec<U> { .. }

map(vec![1, 2, 3], _ { f: |n| n * 2, .. });
map(vec![1, 2, 3], _ { f: |n| n.to_string(), limit: 2 });

fn pick<'a>(x: &'a str, _ {
    fallback: &'a str = "none",
}) -> &'a str { .. }

pick("", _ { .. });
pick("", _ { fallback: "empty" });
```
</div>

### Traits

Methods defined on a `trait` can take anonymous struct parameters just like any
other method. The trait owns the struct and its defaults, and each `impl` method
repeats the fields with the same types, just like positional parameters, but in
any order. Calls through generics and `dyn` work like any other method call.

<div class="highlight-block" markdown="1">
```rust
trait Writer {
    fn write(&mut self, _ {
        data: &'static str,
        times: usize = 1,
    });
}

impl Writer for Buffer {
    fn write(&mut self, _ {
        times: usize,
        data: &'static str,
    }) { .. }
}

buffer.write(_ { data: "hi", .. });
let w: &mut dyn Writer = &mut buffer;
w.write(_ { data: "hi", times: 2 });
```
</div>

Leaving the defaults out of the `impl` keeps them in one place, the trait, but
it means someone reading the impl can't see that `times` defaults to `1`.
Requiring the impl to repeat the default instead, as `times: usize = 1`, checked
against the trait's, might be preferable, at the cost of keeping the two in sync
by hand.

Unlike positional parameters, the struct field names must match the trait's
field names. An `impl` that renames `data` to `bytes` is an error, since `bytes`
isn't a field of the trait's struct.

## 3. Relating to Recent Posts

TODO

<!--
So far this has all been about structs, but the feature people keep asking for
is *named arguments*. They've been [a wishlist issue][rfcs-323] since before
Rust 1.0, the issue is still open, and languages like [Swift][swift-labels],
[Ruby][ruby-kwargs], [Python][python-kwargs], and [Julia][julia-kwargs] already
have them. Anonymous structs get most of the way there, with names, any order,
and defaults, but `copy(_ { from: a, to: b })` still isn't quite
`copy(from: a, to: b)`. Others have been circling similar ideas, so before
closing that gap, it's worth seeing how this compares.

Three recent posts pull in different directions, and each is worth answering on
its own terms. Here's an overview of how this design compares to them, with more
details on each below.

|  | This design | Structs today ([Endler][endler]) | `pub` parameters ([Botahamec][botahamec]) | Names only ([Klabnik][klabnik]) |
| --- | --- | --- | --- | --- |
| Named parameters | Fields of a `_ { .. }` parameter, or parameters after `;` with [possible sugar](#signature-syntax-fn-fa-i32-x-i32) | Fields of a named struct | Parameters marked `pub` | Every parameter, with no marker |
| Reordering | Yes | Yes | Yes | Not settled |
| Defaults | `..` with RFC 3681 field defaults, and `..Default::default()` for named structs, with [possible optional fields](#optional-fields) as a follow-up I'm skeptical of | `..` with RFC 3681 field defaults, and `..Default::default()` | May be non-const, omission implicit | No |
| Evaluation order | As written | As written | Not addressed | Raised as a problem |
| Patterns as parameters | Struct pattern | Struct pattern | `pub name @ pat` | Raised as a problem |
| `fn` pointers | Distinct per function (sharing when the fields match is an open question) | Rejected unless the struct is shared | Names erased | Raised as a problem |
| Adding a parameter | Non-breaking if defaulted, for callers that wrote `..` | Non-breaking if defaulted, for callers that wrote `..` or `..Default::default()` | Non-breaking if defaulted | Breaking |
| Per-function boilerplate | None | One struct per function | None | None |
| New syntax | `_ { .. }` as a parameter and as an expression, struct patterns without their type, and [possible sugar](#4-sugar) for `name: value` arguments and `;` parameters | None | `pub` parameters, `name: value` arguments, defaults | `name: value` arguments |

Klabnik's post is a position rather than a proposal, so its column records the problems it raises rather than answers. Macro crates like [bon] and [structx] fill the same gap today, with a builder or a macro at each call.

<details id="botahamec-names-defaults-and-pub-apis">
  <summary>Botahamec: names, defaults, and <code>pub</code> parameters</summary>
  <div markdown="1">
In ["Named and Optional Arguments are Awesome"][botahamec], Botahamec tours Dart, C#, and TypeScript, then looks at what Rust does without them. `HashMap` has eight constructors to cover three optional parameters. `print(text, bold, italics, underline)` is easy to call with the flags swapped. The options-struct workaround needs a type, a `Default` impl, and `..Default::default()` at every call, and it falls apart when only some fields have defaults.

Their proposal builds on [a 2016 pre-RFC][pre-rfc-2016]. A parameter marked `pub` can be passed by name, and parameters may have defaults, which callers leave out without a trace. A `pub` parameter can still be passed by position too. `pub` is a poor fit for this, since it has nothing to do with visibility (a private `fn` can have `pub` parameters, and `pub(crate) x: u32` has no sensible meaning), and the post itself flags the keyword as an open question. But that's a small point about a single keyword.

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

With an anonymous struct, the same function's named parameters become fields, which are only ever passed by name.

<div class="highlight-block" markdown="1">
```rust
// anonymous struct
fn print_labeled_measurement(_ { value: i32, unit_label: char = 'm' }) { .. }

print_labeled_measurement(_ { value: 5, .. });
print_labeled_measurement(_ { value: 5, unit_label: 'h' });
print_labeled_measurement(_ { unit_label: 'h', value: 5 });
```
</div>

There's no equivalent of `print_labeled_measurement(5, 'h')`, and that's on purpose. Botahamec's proposal, like C#, where every parameter can be named, lets the same parameter be passed by name or by position. With anonymous structs, a parameter is positional or a field, never both. That's what keeps `coordinates(y, x)` from silently meaning `coordinates(y: x, x: y)` after someone drops the labels.

Their `HashMap` example folds the eight constructors into one function, quoted here as written. It doesn't address generic parameters yet. `S` and `A` are never declared, and a default like `A = Global` only typechecks when `A` is `Global`, so the post sets the problem aside for its "Future possibilities" section.

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

An anonymous struct has to face that problem now. It folds the constructors too, but only partway, and this is where const-only defaults hurt. `RandomState::new()` isn't const, and a default can't name a value of a generic type like `S` or `A`, so a default can't be `Global` or call `S::default()` through a bound. The hasher and the allocator both become `Option`s.

<div class="highlight-block" markdown="1">
```rust
// anonymous struct
fn new_map<K, V, S: BuildHasher + Default, A: Allocator + Default>(_ {
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

The bigger difference is how a parameter opts in. Botahamec's proposal needs a keyword for it. Anonymous structs don't. A parameter opts in by being a field, and the only visibility involved is the real one, since the struct's fields take the function's.

  </div>
</details>

<details id="klabnik-names-yes-defaults-no">
  <summary>Klabnik: names yes, defaults no</summary>
  <div markdown="1">
In ["Arguing about arguments"][klabnik], Steve Klabnik is open to labels on existing parameters, `foo(x: 5, y: 6)` for `fn foo(x: i32, y: i32)`, but not to optional or default arguments, because they hide what a call passes. They also list what makes named arguments hard in Rust specifically. Parameters are patterns and may not have a name, `fn` pointers erase names, trait impls can rename parameters, reordering arguments raises the question of evaluation order, and every parameter name quietly becomes API.

Klabnik's objection to defaults is fair, and the answer is that making names explicit is what lets defaults be safe. Fields are named only because the function put them in a struct, and they can only be passed by name, so there's no positional form to swap and no argument left out of the middle of a list, `f(a, c)` for `f(a, b = 0, c)`. Defaults are const and owned by the function (by the trait, for a trait method), and a call that relies on them says so with `..`. That's what makes adding a parameter safe, since a new field with a default doesn't break callers that wrote `..`.

Where Botahamec opts parameters in with `pub`, Klabnik doesn't opt them in at all, and that's the bigger difference. If every parameter can be labelled, the function's author can no longer decide how an argument is passed. Labels are optional, so `copy(a, b)` still compiles, and there's no way to require names for arguments that are confusing by position, like `from` and `to`. That's the whole point of naming them. With anonymous structs the author decides. Positional parameters are passed by position, fields are passed by name, and the signature shows where one ends and the other begins, so `copy(_ { from: a, to: b })` is the only way to call `copy`.

The other problems are secondary. Every parameter name in every published crate would become API after the fact, including names chosen only for the function body, like `buf`, `_x`, hygienic macro names, and bindgen's `arg1`, so renaming any of them would break someone. Each parameter would also get two APIs, since reordering breaks positional callers and renaming breaks labelled ones, and mixed calls would need new rules for which slot a positional argument fills after a label.

The rest of Klabnik's list already has answers, because each has an existing struct rule. None of these are new. Here's how anonymous structs handle each one.

#### Evaluation order

Fields are evaluated in the order written at the call site, as in any struct literal. So this compiles:

<div class="highlight-block" markdown="1">
```rust
fn consume(_ { data: Vec<u8>, length: usize }) { .. }

consume(_ { length: data.len(), data: data });
```
</div>

and swapping the two is a use-after-move error, exactly as it would be for a named struct:

<div class="highlight-block" markdown="1">
```rust
consume(_ { data: data, length: data.len() });
// error[E0382]: borrow of moved value: `data`
```
</div>

#### Patterns as parameters

Klabnik points out that a Rust parameter is a pattern, not a name. `fn foo((x, y): (i32, i32))` introduces two bindings, so there's no single name to label the argument with, short of inventing separate external and internal names.

Anonymous structs don't need either. Only fields are passed by name, and a field is always written `name: Type`, so its name is both the label at the call and the binding in the body. Positional parameters keep their patterns, since they're never passed by name.

<div class="highlight-block" markdown="1">
```rust
fn foo((x, y): (i32, i32), _ { scale: i32 = 1 }) -> i32 { .. }

foo((1, 2), _ { scale: 2 });
foo((1, 2), _ { .. });
```
</div>

#### Defaults

Defaults are RFC 3681 defaults. They're const, declared on the field, and used when the caller writes `..`. They can use the function's type and const parameters (`fn foo<const A: usize>(_ { bar: usize = A })`), and associated consts through their bounds (`fn foo<T: HasN>(_ { n: usize = T::N })`). Const-only is a real limit, and [Section 7](#7-limitations-and-open-questions) covers it.

#### Renaming hazards

A field's name is part of the function's API, the same as a public struct field. Positional parameters are unaffected, so a function opts in, one parameter at a time, by moving it into the struct. Nothing becomes API by accident, and a typo like `widht` gets the same error, with the same suggestion of `width`, as it would in any struct literal.

#### Trait impls

An impl can't rename fields. It repeats the trait's fields by name, in any order, and a renamed field is an error ([Section 2](#traits)). It has to be that way, since callers through generics and `dyn` only see the trait, so the trait's field names are the ones they write.

#### `fn` pointers

A function used as a value keeps its anonymous structs as parameter types, so names survive through the pointer, and so do defaults:

<div class="highlight-block" markdown="1">
```rust
fn foo(_ { a: i32, b: i32 = 10 }) -> i32 { .. }

let g: fn(_) -> i32 = foo;
g(_ { b: 2, a: 4 });
g(_ { a: 5, .. });
```
</div>

Two functions with the same fields don't share a pointer type, though [Section 7](#function-types) sketches one way they might.

  </div>
</details>

<details id="endler-structs-already-do-this">
  <summary>Endler: structs already do this</summary>
  <div markdown="1">
In ["We Have Named Arguments at Home"][endler], Matthias Endler argues the job is already done. A struct gives you names, any order, typo checking, per-field docs, and defaults, and it's a real type you can store, validate, and pass along. Names belong to a type, not to the calling convention.

That's the same premise anonymous structs start from, that named arguments are just structs. Anonymous structs are convenience on top of Endler's model, not a replacement for it. The function still takes a struct, with all of those properties, and the anonymous struct and the inferred literal only remove the ceremony of declaring it and naming it at every call. It's the have-your-cake-and-eat-it-too approach, with the semantics of structs and without the boilerplate.

Endler's `crop_imm` example, borrowed from Klabnik, declares a struct just for this one function:

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
fn crop_imm<I: GenericImageView>(image: &I, _ {
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
1](#elided-struct-types), as in `fn crop_imm<I: GenericImageView>(image: &I,
Crop { x, y, width, height })`. Callers don't change, since `_ { .. }` builds
either one. The call Endler's post [wishes it could write][endler-composition]
at the end gets close without any sugar. It basically just needs `_ { .. }`
around the options and real syntax for the duration and headers:

<div class="highlight-block" markdown="1">
```rust
fn request(url: impl Into<Url>, _ {
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

With the named-argument [sugar](#call-syntax-fa-x-1) from Section 4, the `_ { ..
}` goes away too, and the call has exactly the shape Endler wants. 

  </div>
</details>

### Where they agree

Side by side, the posts agree on more than it looks. Everyone wants names at the call site. Botahamec's objection to the options struct isn't its meaning but its ceremony. In their words, "I don't see much reason to name a new struct if it's only going to be used for the one function." Endler's structs already answer most of Klabnik's hard problems, because struct literals already have rules for names, order, and evaluation.

That leaves one cost, declaring and naming a struct for every function. It's why the pattern shows up in public APIs and almost never in private code. Anonymous structs take Endler's model and remove that cost, which is what Botahamec is asking for. Klabnik's objection to defaults is the one real disagreement, and making the named parameters explicit is what answers it.

That's why I think structs are the right direction, rather than a separate named-argument feature. Everything here is built from parts Rust already has: struct literals, struct patterns, and RFC 3681 defaults, with the struct's name inferred and its declaration moved into the signature. Each post gets what it asks for without giving up what the others care about. Endler's structs stay real types, Botahamec loses the boilerplate, and Klabnik can see every default at the call, since the core form marks them with `..`. Names, order, evaluation, and API stability all follow rules Rust users already know from struct literals, instead of a second set of rules for arguments. And it doesn't close any doors, since sugar can come later if it earns its place, and structural records after that.

-->

## 4. Sugar

TODO

<!--
Everything so far is struct syntax in new places. The sugar here is the part that looks like named arguments, and like the elided struct type in Section 1, it only removes something from what you write. The rule to keep in mind is that **the sugar never adds meaning.** There is one deliberate exception, which is that named arguments to an anonymous struct imply `..`.

I'm less sure about this part than about Sections 1 and 2. Everything they offer works without it, and sugar has costs that structs don't. It gives every call two spellings, so codebases will split on style. It claims `:` in argument lists for good, and ties named arguments to the last parameter, which makes adding a parameter after it a breaking change. The implied `..` hides which fields took their defaults, which is exactly what Klabnik worries about. And the `;` signature syntax hides that the parameter is a struct at all, which can surprise someone who passes the function as a value. These are reasons to add the sugar later, if at all, once the struct parts have proven themselves.

### Call syntax: `f(a, x: 1)`

Named arguments are sugar for a `_ { .. }` literal as the last argument. Once an argument is written `name: value`, it and the ones after it form the struct. The calls from [Section 2](#2-examples) become:

<div class="highlight-block" markdown="1">
```rust
copy(from: a, to: b)?;
// = copy(_ { from: a, to: b })?

crop(&img, width: 200);
// = crop(&img, _ { width: 200, .. })
crop(&img, width: 200, x: 15, y: 25);

fill(&mut shape, red: 0xA8, green: 0x3C, blue: 0x09);
fill(&mut shape, red: 0xFF, ..accent);
```
</div>

- Positional arguments come first, then named ones, then optionally `..` or `..base`, last.
- Against an anonymous struct, named arguments imply `..`, so the fields they leave out take their defaults. A field without a default must still be given. Against a named struct they don't, so `Instance::new(backends: Backends::VULKAN, ..InstanceDescriptor::new_without_display_handle())` still spells out where the rest comes from. The reason is evolution, since a library adding field defaults to its public struct mustn't change which of its callers' calls compile.
- `f(..)` on its own is still a `RangeFull`, as in `v.drain(..)`. A call taking every default writes `f(_ { .. })`, or leaves the struct out (below).
- There's no shorthand, since `f(x, y)` is positional. `_ { x, y, .. }` still has it.
- Named arguments fill the last parameter, so adding a parameter after it is a breaking change for callers that use them. `crop(&img, width: 200)` would name fields of the new parameter instead, and fail to compile. New options belong in the existing struct, as defaulted fields. An anonymous struct parameter anywhere else is passed with an explicit `_ { .. }`.

Positional parameters never become labels. A parameter is a pattern that may not have a name (`(x, y): (i32, i32)`, `_: u8`), and turning its name into API would make every rename a breaking change. So `foo(a: 1)` against `fn foo(a: i32)` is an error. The argument is passed by position, or the parameter moves into a struct.

The syntax happens to be free, since `expr: Type` in expressions (type ascription) was removed, and `macro_rules!` can't write `$e:expr :`. It does claim `:` in argument lists for good, which is the point. `f(x: 1)` always means "the field `x` of the struct the last parameter expects".

Note that `crop(&img, width: 200, height: 200)` doesn't compile against `height: Option<u32>`. It's `height: Some(200)`. Named arguments are sugar for a struct literal, and struct literals don't wrap values in `Some`. [Section 6](#optional-fields) has a more speculative idea that would change that.

A last anonymous struct parameter whose fields all have defaults can also be left out of the call entirely:

<div class="highlight-block" markdown="1">
```rust
fn text(text: &str, _ { font_size: u32 = 16 }) -> Widget { .. }

text("hi");              // text("hi", _ { .. })
text("hi", font_size: 24);
```
</div>

This only works for anonymous structs, never for named ones. `v.push()` mustn't build a default element, and, again, a library adding defaults to its struct mustn't change which calls compile.

There's one caveat. Leaving the struct out works for calls, including method calls and calls through `fn` pointers, but a function used as a value still has its anonymous struct parameter. Changing `fn f()` to `fn f(_ { a: i32 = 0 })` keeps every `f()` compiling, but breaks `opt.unwrap_or_else(f)`.

### Signature syntax: `fn f(a: i32; x: i32)`

[Julia][julia-kwargs] separates keyword parameters from positional ones with a `;`, and the same syntax works here. The call syntax has a mirror in the signature, which is easier to miss. `_ { .. }` as the last parameter is noisy, especially in the common case where it holds the function's options, so a `;` in the parameter list can start it instead. Every parameter after the `;` is a field of one anonymous struct, which is the last parameter.

<div class="highlight-block" markdown="1">
```rust
fn crop(
    img: &Image;
    width: u32,
    height: Option<u32> = None,
    x: u32 = 0,
    y: u32 = 0,
) -> Image
// = fn crop(img: &Image, _ {
//       width: u32,
//       height: Option<u32> = None,
//       x: u32 = 0,
//       y: u32 = 0,
//   }) -> Image

fn copy<P: AsRef<Path>, Q: AsRef<Path>>(; from: P, to: Q) -> io::Result<u64>
// = fn copy<P: AsRef<Path>, Q: AsRef<Path>>(_ { from: P, to: Q }) -> io::Result<u64>
```
</div>

- The parameters after the `;` are fields, exactly as in `_ { .. }`, written `name: Type`, optionally with `mut name` and `= default`. They're names, not patterns, since a field's name is its API.
- There's at most one `;`, and it declares the last parameter, which is the one the call sugar fills. So `fn f(a: i32; x: i32)` is called `f(1, x: 2)`, and the two sides read the same. Anonymous struct parameters anywhere else still use `_ { .. }`.
- It works in traits and impls too, as in `fn write(&mut self; data: &'static str, times: usize = 1);`.
- It's purely syntax, so switching between `;` and `_ { .. }` changes nothing for callers.

`;` already appears inside parameter types, as in `[u8; 4]`, but only inside brackets, so a `;` at the top level of a parameter list is unambiguous. Python's bare `*` for [keyword-only parameters][pep-3102] (`def f(a, *, b=1)`) is similar. Unlike Julia's and Python's, the `;` doesn't make a second kind of parameter. It's just another way to write a struct.

The `;` declares an anonymous struct. When the argument struct should have a name instead, for reuse, forwarding, or traits, declare an ordinary struct and take it with the elided type from [Section 1](#elided-struct-types):

<div class="highlight-block" markdown="1">
```rust
struct CropArgs {
    width: u32,
    height: Option<u32> = None,
    x: u32 = 0,
    y: u32 = 0,
}

fn crop(
    img: &Image,
    CropArgs { width, height, x, y },
) -> Image { .. }
// = fn crop(
//     img: &Image,
//     CropArgs { width, height, x, y }: CropArgs,
// ) -> Image

crop(&img, _ { width: 200, .. });
crop(&img, width: 200, ..);
crop(&img, CropArgs { width: 200, .. });   // also fine
// forwarding, `opts: CropArgs`
crop(&img, _ { width: 50, ..opts });
crop(&img, width: 50, ..opts);
```
</div>

This makes the named form as short as an anonymous one, and spelled the same way with the struct's name in place of `_`. Callers use `_ { .. }` either way, so switching between the two forms doesn't break them, except for calls that rely on an anonymous struct's implied defaults, like `f()` and named arguments without `..`.

Since signatures don't infer, a generic struct's parameters go in the pattern (`Wrapper::<'_, T> { s, v }`). Only struct patterns get this, and tuple-struct patterns (`Point(x, y)`) still need their type.
-->

## 5. The Prototype

### The core feature

```rust
#![feature(struct_args, default_field_values)]
```

TODO: describe the `struct_args` feature.

<!--### How it works

Most of the work happens in the parser, which turns the new syntax into ordinary structs, struct patterns, and struct literals, so name resolution, privacy, borrow checking, and MIR building see nothing new.

- `_ {` in a parameter list becomes an anonymous struct named `‹record·N›` (`‹` can't appear in an identifier, so the name can't be written), placed next to the function with its visibility and `#[cfg]`s, plus the struct pattern `‹record·N› { x, y, .. }: ‹record·N›<..>`. Each field without a default gets its own type parameter, whose argument is the field's written type, so the signature gives it its meaning. A field with a default keeps its type, which the default is checked against. Types print like closures do, as in `{record@src/main.rs:6:9: 6:10}`.
- `_ { .. }` in an expression becomes the struct literal `<_>::_ { .. }`, and named arguments become the same literal. Type checking fills in the struct from the expected type, or defers the literal until inference knows it.
- A `let` record gets its own generic struct deriving `Clone` and `Copy`, and type checking gets one new coercion, a record conversion, which THIR turns back into a struct literal reading the record's fields. Borrow checking then sees ordinary field moves, which gives partial moves and use-after-move errors for free.
- Trait methods get a hidden generic associated type for each record parameter, so an impl's record normalizes to the trait's.
- A left-out record and named arguments' implied `..` are both filled in when building THIR.

Code that doesn't enable the feature sees one change. `_ { .. }` and `f(x: 1)` now report the feature-gate error instead of their old parse errors.
-->

### The sugar

```rust
#![feature(struct_args, struct_args_sugar, default_field_values)]
```

TODO: describe the `struct_args_sugar` feature.

<!--Everything above is implemented in a rustc prototype behind `#![feature(struct_args)]` and `#![feature(struct_args_sugar)]`, marked incomplete. Defaults and `..` also need `#![feature(default_field_values)]`, whose implementation landed in [rust-lang/rust#129514][rust-129514]. The prototype isn't published yet, so its results can't be reproduced for now, but the examples that compile come from it.-->

## 6. Going Further

<div class="aside" markdown="1">
This section is optional reading. It's less developed than the rest of the
post, and the main proposal doesn't depend on it. Neither optional fields nor
`let` records are in the prototype. Both are collapsed below, so feel free to
skip ahead to [Section 7](#7-limitations-and-open-questions).
</div>

<details id="optional-fields">
  <summary>Optional fields: passing <code>T</code> for an <code>Option&lt;T&gt;</code> field</summary>
<!--
  <div markdown="1">
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
// = fn crop(img: &Image, _ { width: u32, height: Option<u32> = None }) -> Image

crop(&img, width: 200);
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

Like the implied `..` from [Section 4](#4-sugar), this would be a deliberate
exception to "the sugar never adds meaning", and limited the same way, to named
arguments to an anonymous struct. `_ { height: 100 }` would still be an error,
and so would `height: 100` against a named struct. Otherwise every struct
literal would change, and a library switching between a named struct and an
anonymous one would change which of its callers' calls compile. The catch is
that a call's arguments can't always move into a [`let` record](#record-types)
unchanged. `crop(&img, width: 200, height: 100)` compiles, but the same fields
in a record don't.

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
A *record* is a struct type whose compatibility with other types is decided by its fields, not its name. Named structs deliberately aren't like this. Two structs with the same fields are still different types, because the name carries meaning, which is what makes newtypes work. A record is intrinsically unnamed, so there's no meaning to protect and it doesn't act like a newtype. Records can relate in two ways. With strict field equality, two records with the same fields are the same type, which is what structural records propose. With subsets and supersets, a record converts, where the target type is known, into any struct whose fields are a subset or a superset of its own. This section is mostly about the second.

#### Why: subsets and supersets

Everything in Sections 1 to 4 ties a struct to one function. That's fine at a single call, but arguments rarely live at a single call. Options get built once and passed to several functions, or passed through a wrapper to the function it wraps, and those functions each want a different slice of them. Without records, you either share one named struct between the functions, which couples their APIs to each other, or copy the fields across by hand at every call.

Records solve this with two kinds of compatibility:

- **Superset.** A record may have more fields than the struct it's passed as. The extra fields stay behind, available to the next call.
- **Subset.** A record may have fewer fields than the struct it's passed as. The missing fields take the struct's defaults, as if the call had written `..`.

Most real uses need both at once:

<div class="highlight-block" markdown="1">
```rust
fn crop(img: &Image, _ {
    width: u32,
    height: Option<u32> = None,
    x: u32 = 0,
    y: u32 = 0,
}) -> Image { .. }

fn encode(img: &Image, _ {
    quality: u8 = 80,
    progressive: bool = false,
}) -> Vec<u8> { .. }

let opts = _ { width: 200, height: Some(100), quality: 90 };
let thumb = crop(&img, opts);  // `quality` stays, `x` and `y` default
encode(&thumb, opts);          // `width` and `height` stay, `progressive` defaults
```
</div>

Neither function knows about the other, and neither has to agree on a shared options type. The record is the only thing that sees all the fields, and each call takes what it needs by name.

#### Records built before the call: `let args = _ { ... };`

A record bound by `let`, without a type annotation or `..`, gets a struct of its own, made from its field names. It converts by field name into whatever struct it's passed as. Fields the target lacks stay behind in the record (the record is a superset), and fields the record lacks take the target's defaults (the record is a subset). It's `Copy` when all its fields are, so one record can go to several functions:

<div class="highlight-block" markdown="1">
```rust
fn greet(_ { name: &'static str, times: usize = 1 }) { .. }
fn greet_prefix(_ {
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

A record isn't tied to one struct, named or anonymous. The same record can fill both `fill`'s `Rgb` and `crop`'s anonymous struct from [Section 2](#2-examples):

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

Its field values are checked without the field types and coerced once the struct is known. That has limits. A closure that needs its signature from the field type only works when the struct literal is written at the call, and reading `args.x` before `crop(&img, args)` needs a type annotation, which an anonymous struct can't have.

#### Structural records

So what about *structural records* in general? [RFC 2584][rfc-2584] proposed `{ red: 0xA8, green: 0x3C, blue: 0x09 }` as both an expression and a type, where any two records with the same fields are the same type. The lang team [closed it][rfc-2584-close] not because it was wrong but because it was not a priority, and scottmcm cautioned against a new fundamental kind of type that [reaches the type system, MIR, Miri, derives, proc macros, and every backend][scottmcm-2023]. Their suggestion was to take the lower-impact pieces first, [struct literals whose type is inferred from use][scottmcm-2021], borrowed from [Zig][zig-anon-struct], and unnameable *Voldemort types* instead of structural ones.

Anonymous structs and records follow these two pieces of advice. `_ { .. }` is an inferred struct literal, and an anonymous struct parameter or a `let` record is an unnameable struct (like a closure's type), so **none of them needs a new kind of type**. The only new rule is that a `let` record converts into other structs by field name. A bare `{ x: 1 }` is still an error, left free for a structural literal, and if records with the same fields someday become the same type, anonymous struct parameters can be redefined as structural records without changing how anyone calls them.

Records, and their conversions, cover a good part of what RFC 2584 wanted structural records for. They still aren't structural records. Each record is its own unnameable struct, and it only meets another struct through a conversion, which happens where the target type is known and moves the fields into a new value. That leaves gaps:

- **No type identity.** Two `let` records with the same fields are different types. `if c { r1 } else { r2 }` and `[r1, r2]` have no single type to convert into, and `r1 == r2` has no impl between them.
- **No way to refer to a record type.** `_ { .. }` in a parameter list declares a new anonymous struct every time it's written, and never names an existing one. So one function's anonymous struct can't be used as a struct field, a return type, or another function's parameter, and a library can only hand one out by converting it into a named struct.
- **Conversion moves, it doesn't borrow.** `&args` can't be passed as `&Rgb`, since the two structs' layouts aren't guaranteed to match, and a `Vec` of one record doesn't convert into a `Vec` of another. Generic code never reaches a conversion at all.
- **Few traits.** A `let` record derives only `Clone` and `Copy`. It has no `Debug`, `PartialEq`, `Hash`, or `Default`, and no user `impl`s, since there's no type to name.

None of these block structural records later, because each is something records leave out, not something they decide:

- The bare `{ x: 1 }` syntax, for expressions, patterns, and types, is still free.
- A record's type can't be named, so, like a closure's, its identity is unspecified. Two records with the same fields can become the same type later without changing what any caller writes, and any `fn` pointer shim like the one sketched in [Section 7](#function-types) could be retired the same way.
- Deriving more traits for records is additive.

#### Converting by field name

What records do decide is the conversion itself. Once `let` records convert by field name, leaving behind fields the target lacks and filling in its defaults, structural records either inherit that rule or break it. [RFC 2584][rfc-2584-rendered] listed both larger-to-smaller and record-to-struct coercions as future possibilities, and warned that they "could also reduce robustness".

The robustness risk is a field that silently goes nowhere. A misspelled field in a `let` record, `_ { width: 200, hieght: Some(50) }` passed to `crop`, stays behind, and `height` takes its default. The plan is to treat that the way Rust already treats a variable nobody reads, so a field of a `let` record that no conversion takes is unused, and gets a warning. That catches the typo. It also catches a library renaming `height` to `h`, which is already a breaking change since field names are API, because the caller's `height` no longer goes anywhere. A field that some call does take isn't unused, so one record can still feed both `fill` and `crop`.

Ruby went through a harder version of this. Its automatic conversion between a trailing hash and keyword arguments had enough corner cases, like a hash landing in a positional parameter or in the keywords depending on whether that parameter had a default, that [Ruby 3.0 separated them][ruby-3-kwargs] and made callers write `**` to pass a hash as keywords. Records don't have that ambiguity. A record only ever fills the parameter in its own position, and field names are checked at compile time. The unused-field warning covers the silent part that's left.

The conversion rule, not the records' types, is what has to be settled together with structural records.

#### Named records?

Every record so far is unnameable, which is what leaves the second gap above. A function can't return one, a struct can't hold one, and two functions can't share one except through a named struct, which doesn't convert by field name. Should there be syntax for declaring a named record, a type that has a name but still converts by field name like a `let` record?

<div class="highlight-block" markdown="1">
```rust
// possible spellings
type CropArgs = _ { width: u32, height: Option<u32> = None };
#[record] struct CropArgs { width: u32, height: Option<u32> = None }
```
</div>

A named record gives up what a named struct's name protects, so it has to be the definer's choice, never something a caller can do to an existing struct. Whether it's worth new syntax, or should wait for structural records to make the type writable directly, is open.
  </div>
-->
</details>


## 7. Limitations and Open Questions

### Non-const defaults

TODO

<!--Field defaults are RFC 3681 defaults, so they have to be const. A default can't call `RandomState::new()`, and it can't reach a generic type's value through its bound, like `S::default()`. A field that needs one becomes an `Option` that the body fills in, as in the `HashMap` example in [Section 3](#botahamec-names-defaults-and-pub-apis). Named structs can fall back on `..Default::default()`, which runs ordinary code, but an anonymous struct has no `Default` impl to call. This is a limit of RFC 3681 rather than of records, so lifting it belongs in a follow-up to that RFC ([Section 8](#8-whats-next)).

RFC 3681 makes defaults const on purpose. Its [rationale][rfc-3681-const] is that `Foo { .. }` stays deterministic and cheap, and that a crate can't break a downstream `const fn` by quietly changing a default to a non-const one. Its [future possibilities][rfc-3681-non-const] leave non-const values open, "potentially allowed but linted against", and expect `Default` impls to become const where they can. The [tracking issue][rust-132162] lists "Expand support to non-const values?" as an open question that doesn't block stabilization, with the author's position that "we shouldn't do that, particularly seeing how powerful const eval is becoming".

That question is where the discussion is. [cart argued][cart-2026], from Bevy, that `bar: Bar = Default::default()`, `String`s, and `seed: usize = rand()` are exactly the defaults people want, and asked why a struct literal can't just be const when all of its defaults are. [scottmcm replied][scottmcm-2026] that [const traits][const-traits-goal] are a 2026 project goal, and asked which cases truly *can't* be const, and how `seed: usize = rand()` compares to `seed: Option<usize> = None`. That last comparison is the `HashMap` example, and this [reply][vultix-2026] applies here too: `Option` says the hasher is optional, when every map has one and only passing it is optional. Const traits would cover `S::default()` through a `[const] Default` bound, but not `RandomState::new()`, which seeds itself from the OS at runtime.-->

### Function types

TODO

<!--It would be nice if two functions whose anonymous structs have the same fields, with the same names and types, could unify as values, so that `if c { resize } else { grow }` compiles, and `type Callback = fn(_ { width: u32, height: u32 })` accepts both. The prototype doesn't do this. The generated structs are distinct types whose layouts aren't guaranteed to match, so one possible approach is a conversion shim, like the existing reify shims, that moves the fields from one struct into the other. Limiting it to the same fields would keep `if`/`else` symmetric, never drop a field behind the caller's back, and leave room for structural records to replace the shim later.

The subset and superset conversions of [record types](#record-types) would stretch this further. A function taking `_ { width: u32, height: u32, x: u32 = 0 }` could stand in for a `Callback`, with the shim filling in `x` from its default, and a shim going the other way would drop `x`, which is the silent loss the same-fields limit avoids. Which functions unify would then depend on which conversions apply in which direction, not just on their fields, so the same-fields rule is the conservative place to start. None of this is designed in detail yet.

Generic records are harder. In `fn higher_order<A>(f: impl Fn(A)) { f(a: 4, b: 2) }`, the body can't build an `A`, since the caller picks it. Today that needs a named struct (`impl Fn(FooArgs)`).-->

### `Option` fields as optional arguments

TODO

<!--Whether the [optional fields](#optional-fields) idea from Section 6, an implied `= None` and a plain `T` wrapped in `Some`, is worth its costs is open. Wrapping makes inference depend on the field's type, needs a rule for `Option<Option<T>>` fields, and would be Rust's first implicit `T` to `Option<T>` conversion. I don't expect this to be accepted, frankly, but it has some attractive qualities.-->

<!-- TODO: settle how `Option` fields should work. -->

### Tooling

TODO

<!--Tools that parse Rust themselves don't get any of the compiler's changes. rust-analyzer's parser treats `_ {` as an error and loses highlighting for the rest of the signature, and semantic support needs about as much work as the prototype did. tree-sitter-rust, which Neovim, Helix, Zed, and GitHub use, needs about ten lines of grammar, and currently lacks RFC 3681's default field values too. rustfmt uses rustc's parser, so it formats `_ { .. }` literals and the code around them, but keeps anonymous struct parameters, `;` parameter lists, and named arguments as written.-->

### Prototype gaps

TODO

<!--These are gaps in the prototype, not the design:

- Attribute macros built on `syn` reject anonymous struct parameters, so `#[tokio::main]` and `#[tracing::instrument]` can't be used on such functions.
- Diagnostics name anonymous structs by location, as `{record@src/main.rs:6:9: 6:10}`, and a few suggestions name a struct you can't write.-->

## 8. What's Next

- **An RFC.** The 2020 named-arguments RFC was [closed][rfc-2964-close] because the lang team wanted to see "what the roadmap looks like towards the eventual full solution" before an incremental step. This is meant to be that roadmap, with struct literals with an inferred name now, named-argument sugar on top, and structural records later if they're wanted.
- **Follow-ups to [RFC 3681][rfc-3681].** These are limits of default field values ([tracking issue][rust-132162]), not of anonymous structs, so they belong in follow-ups to that RFC, and anonymous and named structs keep the same rules.
  - **Non-const defaults.** `hasher: S = RandomState::new()` needs a non-const default. The tracking issue already has it as an open question ([Section 7](#non-const-defaults)).
  - **API evolution.** RFC 3681 doesn't allow `#[non_exhaustive]` on structs with default field values. If the two are ever allowed together, anonymous structs could opt into `#[non_exhaustive]`, or be non-exhaustive by default, with every literal from outside the crate required to write `..`. Adding a field with a default would then never be breaking ([Section 1](#api-evolution-and-semver)).
- **Follow-up posts.** Several of the threads cited here are subsumed by this proposal or depend on how it goes, and each should get a comment pointing to it.
  - **[rfcs#323][rfcs-323], named arguments.** The wishlist issue is still open. Anonymous struct parameters with the [named-argument sugar](#4-sugar) would subsume it.
  - **[RFC 3444][rfc-3444], inferred types.** `_ { .. }` is the struct half of its path inference, spelled `_` instead of `.`. The two need one spelling, and the struct half doesn't need to wait for enums.
  - **[RFC 2584][rfc-2584], structural records.** It's closed, but a revival would inherit the `let` record [conversion rule](#converting-by-field-name), and anonymous struct parameters could become structural records without changing any caller.

<div class="highlight-block" markdown="1">
```rust
fn send_feedback(_ {
    from: &User,
    to: &User,
    message: &str,
    mood: Mood,
});

send_feedback(_ {
    from: "you@example.com",
    to: "nathan@nixpulvis.com",
    message: """
      Named arguments are dumb, Rust shouldn't even allow
      multiple arguments. Everything should be curried:
      
      λ(x, y).e := λx.λy.e
    """,
    mood: Mood::Angry,
});
```
</div>

Seriously though, I'd like to hear what you think, where this breaks, what APIs it doesn't fit, or rules that surprise you.

TODO: users.rust-lang.org link.

<!-- References -->

<!-- Blog posts -->
[botahamec]: https://botahamec.dev/named-optional-args
[klabnik]: https://steveklabnik.com/writing/arguing-about-arguments/
[endler]: https://corrode.dev/blog/named-arguments-at-home/
[endler-composition]: https://corrode.dev/blog/named-arguments-at-home/#composition-is-a-superpower

<!-- RFC pull requests -->
[rfc-2584]: https://github.com/rust-lang/rfcs/pull/2584
[rfc-3444]: https://github.com/rust-lang/rfcs/pull/3444
[rfc-3681]: https://github.com/rust-lang/rfcs/pull/3681

<!-- RFC texts -->
[rfc-2102]: https://rust-lang.github.io/rfcs/2102-unnamed-fields.html
[rfc-2584-rendered]: https://github.com/Centril/rfcs/blob/rfc/structural-records/text/0000-structural-records.md
[rfc-3681-const]: https://rust-lang.github.io/rfcs/3681-default-field-values.html#on-const-contexts
[rfc-3681-non-const]: https://rust-lang.github.io/rfcs/3681-default-field-values.html#non-const-values

<!-- Comments on RFC issues and pull requests -->
[rfc-2584-close]: https://github.com/rust-lang/rfcs/pull/2584#issuecomment-810313148
[rfc-2964-close]: https://github.com/rust-lang/rfcs/pull/2964#issuecomment-671545486
[scottmcm-2021]: https://github.com/rust-lang/rfcs/pull/2584#issuecomment-794508491
[scottmcm-2023]: https://github.com/rust-lang/rfcs/pull/2584#issuecomment-1477166822
[rfc-3444-binrw]: https://github.com/rust-lang/rfcs/pull/3444#issuecomment-5626022100
[nixpulvis-2016]: https://github.com/rust-lang/rfcs/issues/323#issuecomment-213065086
[cart-2026]: https://github.com/rust-lang/rust/issues/132162#issuecomment-4930535333
[scottmcm-2026]: https://github.com/rust-lang/rust/issues/132162#issuecomment-4930787638
[vultix-2026]: https://github.com/rust-lang/rust/issues/132162#issuecomment-4931304399

<!-- Issues and tracking issues -->
[rfcs-323]: https://github.com/rust-lang/rfcs/issues/323
[rfcs-2416]: https://github.com/rust-lang/rfcs/issues/2416
[rust-49804]: https://github.com/rust-lang/rust/issues/49804
[rust-132162]: https://github.com/rust-lang/rust/issues/132162
[rust-42764]: https://github.com/rust-lang/rust/issues/42764

<!-- Project goals -->
[const-traits-goal]: https://rust-lang.github.io/rust-project-goals/2026/const-traits.html

<!-- rustc pull requests -->
[rust-129514]: https://github.com/rust-lang/rust/pull/129514

<!-- Internals forum threads -->
[pre-rfc-2016]: https://internals.rust-lang.org/t/pre-rfc-named-arguments/3831
[pre-rfc-unnamed-struct-types]: https://internals.rust-lang.org/t/pre-rfc-unnamed-struct-types/3872
[irlo-record-types]: https://internals.rust-lang.org/t/record-types/18258
[irlo-anon-struct-2025]: https://internals.rust-lang.org/t/is-the-anonymous-struct-unnamed-struct-still-in-progress/23592
[scottmcm-2023-irlo]: https://internals.rust-lang.org/t/record-types/18258/11
[josh-2025]: https://internals.rust-lang.org/t/is-the-anonymous-struct-unnamed-struct-still-in-progress/23592/3

<!-- Crates and crate source -->
[bon]: https://docs.rs/bon
[structx]: https://docs.rs/structx
[binrw-named-args]: https://github.com/jam1garner/binrw/blob/585d48134e7b71be74d8b81b54cdbd07f6f1a1c4/binrw/src/named_args.rs

<!-- Standard library -->
[fs-copy]: https://doc.rust-lang.org/std/fs/fn.copy.html
[ref-coercions]: https://doc.rust-lang.org/reference/type-coercions.html

<!-- Other languages -->
[swift-labels]: https://docs.swift.org/swift-book/documentation/the-swift-programming-language/functions/#Function-Argument-Labels-and-Parameter-Names
[ruby-kwargs]: https://docs.ruby-lang.org/en/master/syntax/calling_methods_rdoc.html#label-Keyword+Arguments
[ruby-3-kwargs]: https://www.ruby-lang.org/en/news/2019/12/12/separation-of-positional-and-keyword-arguments-in-ruby-3-0/
[python-kwargs]: https://docs.python.org/3/tutorial/controlflow.html#keyword-arguments
[zig-anon-struct]: https://ziglang.org/documentation/master/#Anonymous-Struct-Literals
[julia-kwargs]: https://docs.julialang.org/en/v1/manual/functions/#Keyword-Arguments
[pep-3102]: https://peps.python.org/pep-3102/
