---
title: Secure Multi-Party Communication & Rust
layout: research
draft: true
---

> Abstract: We show extensions to Rust (and the formal Oxide) for secure
> multi-party computation, named `obliv-rust`. This language allows for N party,
> and M protocol computations, and further allows hybrid embeddings of secure
> code inside insecure code, **as well as** insecure code inside of secure
> code.

## N Party, M Protocol, Computations

### Rust Example: 2-Program

Two seperate programs can be run. In this case the seperate programs are run as
threads in a test harness, however it's possible to think of these as being
seperate implementations as well.

```rust
// `u32, u32 -> @`  TODO: How to handle party-arity (lol).
use obliv::oldest;

// garbler?
fn A() {
    let mut obliv = obliv! {
        A <= 56734,
        B?
    }.unwrap();

    // Return value (can you rerun a protocol? &mut?).
    let output = obliv.run(oldest, 10);
    assert_eq!(output, B);  // consistent tests!
}

// evaluator?
fn B() {
    let mut obliv = obliv! {
        A => 56734,  // could be nixpulvis.com.
        B?
    }.unwrap();

    let output = obliv.run(oldest, 20);
    assert_eq!(output, B);  // consistent tests!
}

/// Runs a two-party Yao's garbled circuit algorithm using
/// obliv-rust's local mode?
#[test]
fn test_native() {
    // Server in 2 party direct communication is the one accepts.
    // (require block?)
    let server = thread::spawn(A);
    B();
    server.join().unwrap();
}
```

### Rust Example: 2-Process 1-Program

A single program could be run on a single machine (in this case) or multiple
machines, with many processes.

```rust
// same `u32, u32 -> @` function.
use obliv::oldest;

fn main() {
    let party = read_cli("--party={A,B} [--send] [--recv] AGE");

    let mut obliv = if party.send {
      obliv!(party.label => party.address)
    } else {
      obliv!(party.label <= party.address)
    }.unwrap();

    let output = obliv.run(oldest, party.age);
    println!("Oldest is: {}", output);
}
```

### Party Permutations

2 party communication is? fundamentally asymmetric:

    A -> B

... but can introduce symmetric communication:

    A <-> B

3 party communication can *also* be arguably symmetric:

    A -> B <- C

... and can host symmetric 2 party communication:

    A <-> B <-> C

3 party communication can *also* be asymmetric:

    A -> B -> C

4 party communication can be asymmetric in a number of ways...

Permutations of size 4, level 2:

    A -> B
      -> C
      -> D

    A -> B
      -> D
      -> C

    A -> C
      -> B
      -> D

    ...

Some options for size 4, level 3:

    A -> B -> D
      -> C ->

    A -> B -> D
      -> C

    A -> B -> C
           -> D

And obviously the permutations of level 4:

    A -> B -> C -> D
    B -> C -> D -> A
    ...

4 party communication can host a number of symmetric schemes:

    A <-> B
      <-> C
      <-> D

    A <-> B <-> D
      <-> C

### Oblivious Function Definition

We add two new elements to the Rust syntax: `obliv` and `@`. The `obliv`
keyword introduces new semantics to be aware of to the programmer (secure
computations take time, and change things). Meanwhile, the syntax `@` is a type
for _parties_, and `T @ P` is type `T` coined by `P`.

```rust
pub obliv fn oldest(x: u32 @A, y: u32 @B) -> @ {
    obliv if x > y { A } else { B }
}
```

To call one of these `obliv` functions you can do one of two things:

```rust
#[test]
fn two_argument_oldest() {
    assert_eq!(B, oldest(10 @A, 20 @B));
}
```

##### Hybrid Aspect

```rust
pub obliv fn oldest<P:{@A, @B}>(x: u32 @A, y: u32 @B) -> P {
    let result = obliv if x > y { @A } else { @B }

    // Print some local code, why not?

    aware @ A {
        println!("hi I'm A");
    }

    aware @ B {
        println!("hi I'm B");
    }

    // Or again as:
    announce_2();
    // Backwards?
    announce_2::<@B,@A>();

    // Dangerous
    aware @ A {
        println!("A stealing x: {}, y: {}", x, y);
    }
}

pub fn announce_2<@A, @B>() {
    aware @A {
        println!("hi I'm A");
    }

    aware @B {
        println!("hi I'm B");
    }
}
```
TODO

### Protocol Execution

We have two possible choices:

1. Order of secure operations defines wiring
2. Binding names define wiring (invent labels for each send and recv?)

### Crazy IDE Idea

OK, so I was giving more thought to the issue of making protocol code nicer to
read in an editor... But I didn't love the idea of requiring real GUI like
features (arrows, dragging, etc). But I don't mind the idea of color, and I was
reviewing some earlier notes, and I think I like the looks of a paint bucket
$$\bullet$$, and something like $$\lambda_{\bullet}$$ in general:

$$
(\lambda_{\bullet} \ \color{blue}{x} \ \color{red}{y} \ . \color{blue}{\bullet}\ e) \
\color{blue}{\bullet}a \
\color{red}{\bullet}b
\rightarrow \color{blue}{\bullet \
e[\color{black}a/\color{blue}x,\color{black}b/\color{red}y]}
$$

$$
\begin{align*}
e \ \color{blue}x \ \color{red}y &= f(\color{blue}x,\color{red}y) \\
\color{blue}{\bullet} \ e \ \color{blue}x \ \color{red}y &= \color{blue}{f(x,\color{red}y)} \\
\color{red}{\bullet} \ e \ \color{blue}x \ \color{red}y &= \color{red}{f(\color{blue}x,y)}
\end{align*}
$$

This is ideal for the editor, since I fear coloring in general is too hard for
deeply dependent protocols since every mix of "black" functions is really a
seperate coloring. For Rust syntax highlighting, I'll need support for the main
schemes.
