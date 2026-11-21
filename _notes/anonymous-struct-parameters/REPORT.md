# mathstuf's closure question, tested against the prototype

Source: post #6 in the IRLO thread "Named arguments are just structs". mathstuf asks whether
`crop` (with a defaulted anonymous struct parameter) is compatible with

```rust
let closure = |aggregate: struct { width: u32, height: Option<u32>, x: u32, y: u32 }| crop(img, aggregate);
```

Compiler: `build/host/stage1/bin/rustc` on `struct-args+type-elision` (HEAD `8a84abe71ad`, built Oct 16).
Every program shares this prelude, with features `struct_args, struct_args_pattern_elision,
named_arguments, default_field_values, parameter_type_elision`:

```rust
struct Image;
fn crop(img: &Image, struct { width: u32, height: Option<u32> = None, x: u32 = 0, y: u32 = 0 }) -> Image { .. }
```

Programs are in `/private/tmp/claude-501/-Users-nixpulvis-Code-rust/1ef79975-2500-4704-959c-d3fdd8b6265b/scratchpad/mathstuf/`.

## Short answer

No, as written. Two independent reasons:

1. `struct { .. }` is rejected in a closure parameter type ("only allowed as parameter types" of fn items).
2. Even where it is allowed (a wrapper `fn`), records are nominal: an identically shaped
   `struct { .. }` is a distinct type from `crop`'s record, so it can't be passed through, and FRU
   (`_ { ..a }`) across the two doesn't work either.

What does work: leave the closure parameter unannotated. It then infers `crop`'s own record type,
defaults included, so `closure(_ { width: 200, .. })` and even `closure(width: 200)` compile and run.

## Results

| # | Case | Result |
|---|------|--------|
| 01 | `crop(&img, _ { width: 200, .. })` and `crop(&img, width: 200)` | compiles, runs |
| 02 | mathstuf's closure, `\|aggregate: struct { .. }\|` | **error**: anonymous structs only allowed as parameter types |
| 03 | `\|aggregate\| crop(&img, aggregate)` called with all fields | compiles, runs |
| 04 | same closure called with `_ { width: 200, .. }` (defaults) | compiles, runs |
| 05 | wrapper `fn crop2(img, a: struct { same fields })` forwarding `a` | **error** E0308: distinct record types |
| 06 | wrapper that destructures and rebuilds, `crop(img, _ { width, height, x, y })` | compiles, runs |
| 07 | wrapper forwarding with FRU, `crop(img, _ { ..a })` | **error** E0308: distinct record types |
| 08 | `crop` passed as `impl Fn(&Image, A) -> Image`, called with `_ { width: 200, .. }` | compiles, runs |
| 09 | `let f: fn(&Image, _) -> Image = crop;` then `f(&img, _ { width: 200, .. })` | compiles, runs |
| 10 | unannotated closure defined but never called | compiles |
| 11 | `let a = _ { width: 200, .. }; crop(&img, a);` | **error** E0282: type annotations needed |
| 12 | `fn run(cb: impl Fn(struct { width: u32 }))` | **error**: anonymous structs only allowed as parameter types (known TODO item) |
| 13 | unannotated closure called with named args `closure(width: 200)` | compiles, runs |
| 13 | same closure called as `closure()` (left-out record) | **error** E0057: 1 argument expected |
| 14 | `\|_ { width, height, .. }\| crop(&img, _ { width, height, .. })` | **error** E0282: type annotations needed |

## Compiler output for the failures

### 02: mathstuf's closure

```
error: anonymous structs are only allowed as parameter types
  --> 02_mathstuf_closure.rs:13:31
   |
13 |     let closure = |aggregate: struct { width: u32, height: Option<u32>, x: u32, y: u32 }| crop(&img, aggregate);
   |                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not allowed here
```

### 05: forwarding an identically shaped record

```
error[E0308]: mismatched types
  --> 05_wrapper_fn_same_fields.rs:12:15
   |
12 |     crop(img, a)
   |     ----      ^ expected `{record@...}`, found a different `{record@...}`
   |     |
   |     arguments to this function are incorrect
   |
   = note: `{record@...}` and `{record@...}` have similar names, but are actually distinct types
note: `{record@...}` is defined in module `crate` of the current crate
  --> 05_wrapper_fn_same_fields.rs:11:26
note: `{record@...}` is defined in module `crate` of the current crate
  --> 05_wrapper_fn_same_fields.rs:4:22
note: function defined here
```

The "have similar names" note with `{record@...}` elided in the main label is unhelpful: the
span-qualified names appear in 07 but not here.

### 07: FRU across records

```
error[E0308]: mismatched types
  --> 07_wrapper_fn_fru.rs:12:21
   |
12 |     crop(img, _ { ..a })
   |                     ^ expected `{record@07_wrapper_fn_fru.rs:4:22}`, found `{record@07_wrapper_fn_fru.rs:11:26}`
   |
   = note: `{record@07_wrapper_fn_fru.rs:11:26}` and `{record@07_wrapper_fn_fru.rs:4:22: 4:28}` have similar names, but are actually distinct types
```

### 11: record literal with no expected type

```
error[E0282]: type annotations needed
  --> 11_record_in_let.rs:13:13
   |
13 |     let a = _ { width: 200, .. };
   |             ^ cannot infer type
```

### 12: anonymous struct in an `Fn` bound

```
error: anonymous structs are only allowed as parameter types
  --> 12_impl_fn_bound.rs:11:20
   |
11 | fn run(cb: impl Fn(struct { width: u32 })) { cb(_ { width: 1 }) }
   |                    ^^^^^^^^^^^^^^^^^^^^^ not allowed here
```

### 13: left-out record through a closure

```
error[E0057]: this function takes 1 argument but 0 arguments were supplied
  --> 13_closure_named_args.rs:14:5
   |
14 |     closure();
   |     ^^^^^^^-- argument #1 of type `{record@13_closure_named_args.rs:4:22: 4:28}` is missing
help: provide the argument
   |
14 |     closure(/* {record@13_closure_named_args.rs:4:22} */);
```

Named arguments do reach the closure's record (13b), but leaving out the whole record does not.
I haven't checked why; my guess is that `omittable_record` only handles fn-item callees.

### 14: struct pattern in a closure parameter

```
error[E0282]: type annotations needed
  --> 14_closure_destructure.rs:12:20
   |
12 |     let closure = |_ { width, height, .. }| crop(&img, _ { width, height, .. });
   |                    ^^^^^^^^^^^^^^^^^^^^^^^ cannot infer type
```

Inferred, not traced: the pattern is checked before the body, so the record type `crop` would
give it isn't known yet. Unlike 03, there's no plain binding to defer the type through.

## Takeaways for the thread

- The answer to "why not" is nominality plus the parameter-position restriction. Making 02 work
  needs both closure-parameter anonymous structs and structural equivalence between records, which
  TODO.md already lists as a separate design ("Records stay nominal").
- The idiomatic forms that do work today: an unannotated closure (03, 04, 13b), passing `crop`
  itself where a closure is expected (08, 09), or destructure-and-rebuild in a wrapper (06).
- Defaults belong to `crop`'s record type, so anything that carries that type carries the
  defaults; mathstuf's annotated record has none, which would matter even under structural typing.
