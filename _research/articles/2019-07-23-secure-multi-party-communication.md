---
title: Secure Multi-Party Communication & Rust
layout: article
draft: true
---

> **Abstract:** We show an extension to Rust for secure multi-party computation,
> named `obliv-rust`. This language allows for N party, M protocol
> computations, and further allows hybrid embeddings of secure code inside
> insecure code and vise-versa.

## Yao's Millionaire Problem

One of the classic examples of a MPC problem is the millionaire's problem.
It's typically concerned with two friends at dinner deciding how to pay. They
agree whoever has the most money should pay, but **do not** want to tell each
other (or anyone else for that matter) how much money they have. The following
code implements a slightly more complicated (perhaps Americanized) version of
the millionaires problem, where tipping is considered.

```rust
// Insecure local code cast into oblivion.
let mut a_worth = obliv 1_362;
let mut b_worth = obliv 4_626_291;

// Remove `amount` from the given reference to a `wallet`.
/// This is done without leaking any information about what
/// either value is. For convenience, this function also
/// returns the expected tip based on the amount.
obliv fn pay(amount: obliv u64, wallet: &mut obliv u64)
    -> obliv u64
{
    *wallet -= amount;
    (amount as f64 * 0.2).round() as u64
}

// The amount either A or B will `pay`.
let bill = 245;

// Obliviously compare A's worth with B's. If A is worth more
// than B, A pays for dinner and B pays the tip, otherwise A
// just pays the tip and B is stuck with the bill.
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


### Oblivious Function Definition

We add two new elements to the Rust syntax: `obliv` and `@`. The `obliv`
keyword introduces new semantics to be aware of to the programmer (secure
computations take time, and change things). Meanwhile, the syntax `@` is a type
for _parties_, and `T@P` is type `T` coined by `P`.

It's worth mentioning that `obliv T` syntax is essentially sugar for `T@P`
where `P` is a vacuous party with no inhabitants.

```rust
obliv fn oldest(x: u32@A, y: u32@B) -> @ {
    obliv if x > y { A } else { B }
}

#[test]
fn two_argument_oldest() {
    assert_eq!(B, oldest(10@A, 20@B));
}
```

### Crazy IDE Idea

So I was giving more thought to the issue of making protocol code nicer to
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
separate coloring. For Rust syntax highlighting, I'll need support for the main
schemes.

This is how some research papers handle multiple language embedding, which is
not too different from multi-party/protocol code.
