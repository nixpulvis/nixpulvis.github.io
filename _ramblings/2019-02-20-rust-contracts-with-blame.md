---
layout: rambling
---

Blimey!

```rust

add1("foo");
```
```rust
fn add1(n: isize) -> isize {
    n
}
```
```
error[E0308]: mismatched types
 --> foo.rs:6:10
  |
6 |     add1("foo");
  |          ^^^^^ expected isize, found reference
  |
  = note: expected type `isize`
             found type `&'static str`
```


