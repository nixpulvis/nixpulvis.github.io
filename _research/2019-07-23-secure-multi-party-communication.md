---
title: Secure Multi-Party Communication & Rust
layout: article
draft: true
published: false
---

> Abstract: We show extensions to Rust (and the formal Oxide) for secure
> multi-party computation, named `obliv-rust`. This language allows for N party,
> and M protocol computations, and further allows hybrid embeddings of secure
> code inside insecure code, **as well as** insecure code inside of secure
> code.

## Yao's Millionaire Problem

One of the classic examples of an MPC problem is the millionaire's problem.
It's typically concerned with two friends at dinner deciding how to pay. They
agree whoever has the most money should pay, but **do not** want to tell each
other (or anyone else for that matter) how much money they have.

```rust
// TODO: Protocol initialization.

// Insecure local code cast into oblivion.
let mut a_worth = obliv 1362;
let mut b_worth = obliv 4626291;

// Remove `amount` from the given reference to a `wallet`. This is done without
// leaking any information about what either value is. For convenience, this
// function also returns the expected tip based on the amount.
obliv fn pay(amount: obliv u64, wallet: &mut obliv u64) -> obliv u64 {
    // TODO: How to avoid leaking knowledge of the size of the type. If we're
    // computing on a u8, it's clear nobody is very rich, or the units aren't
    // $.
    *wallet -= amount;
    (amount as f64 * 0.2).round() as u64
}

// The amount either A or B will `pay`.
let bill = 245;

// Obliviously compare A's worth with B's, and call `pay` with the corresponding
// reference. If A has more worth than B, A pays for dinner and B pays the tip,
// and visa-versa if B has more worth than A.
obliv if a_worth > b_worth {
    let tip = pay(bill, &mut a_worth);
    pay(tip, &mut b_worth);
} else {
    let tip = pay(bill, &mut b_worth);
    pay(tip, &mut a_worth);
}

// Refactor

let (payer, tipper) = obliv if a_worth > b_worth {
    (&mut a_worth, &mut b_worth)
} else {
    (&mut b_worth, &mut a_worth)
}
let tip = pay(bill, payer);
pay(tip, tipper);
```

## N Party, M Protocol, Computations

Next we begin to deal with how computations can compute on the parties involved
in the protocol more directly. Since it's safe to say every party must have
a unique identifier within the context of the protocol.

We start with the simple case of a single, two party MPC, then show how this
works with more complex interactions of parties and protocols.

### Rust Example: 2-Program

Two separate programs can be run. In this case the separate programs are run as
threads in a test harness, however it's possible to think of these as being
separate implementations as well.

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

<details>
  <summary>Plenty of Party Permutations</summary>
  <div markdown="1">
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
  </div>
</details>

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
