---
layout: rambling
draft: true
---

Blimey!

```rust
add1("foo");
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

```rust
fn add1(n: isize) -> isize {
    n
}
```

TODO:

Contracts are good at protecting dynamic behavior. Instead of type errors, that
describe a mismatch in some _shape_ (even afine shape in Rust), a contract
error blames the responsible party at runtime for the wrong _behavior_.

```rust
#![contract(pre  = "n >= 0",
            post = "n >= 2")]
fn add2(n: isize) -> isize {
    n
}

// TODO: This kind of bridge shouldn't be needed.
#![contract("n >= 0")]
fn nat?(n: isize) -> bool { n >= 0 }

// These are better.
let nat_c = contract!("n >= 0")]
define_contract!("nat_c", "n >= 0")]
let nat_c = contract!(n) {
    n >= 0
}

// Then this can be either...
#![contract(pre  = nat_c,
            post = contract!("n >= 2")]
fn add2(n: isize) -> isize {
    n
}
```

```rust
// Test a return value post condition fail.
#![contract(pre  = contract!("n >= 0"),
            post = contract!("ret >= 2")]
fn add2(n: isize) -> isize {
    0
}

add2(1);

// flat add2 failed to return a "n >= 2" contract.
```

TODO: Remove need for `ret`.

```rust
#![contract(pre  = "listof?(nat?, list)",
            post = "listof?(nat?, ret)")]
fn nat_map<F: Fn(&T) -> U, T, U>(list: &[T], f: F) -> Vec<U> {
  if list.len() == 1 {
    vec![]
  } else {
    list.iter().map(f).collect()
  }
}
```

```rust
nat_map(vec![1,2,3], |n| n);
// [1, 2, 3]
nat_map(vec![1,0,-1], |n| n);
// ^ contract error on `list`, blame caller.
nat_map(vec![1], |n| n);
// ^ contract error ("bug" in code), blame `nat_map`.
nat_map(vec![1,2,3], |n| n - 1);
// ^ contract error on return value (due to f being the sub1 function),
// blame caller?
```
