---
title: Secure Multi-party Communication
layout: research
draft: true
---

> Abstract: Towards a minimal language for multi-party communication and
> computations. I (we?) define **two** new languages: $$\lambda_\div$$, an untyped
> and typed language for probabilistic computation, and $$\lambda_o$$ for
> multi-party probabilistically obscure communication.
>
> $$\lambda_o$$ is an extension of $$\lambda_\div$$, however it doesn't need to
> export the idea of a $$\div$$ flipping primitive.


### $$\lambda_\div$$ the Uncertainty Calculus

$$
\lambda a \ b . \div \ a \ b \\
50\% \rightarrow a \ b \\
50\% \rightarrow b \ a
$$

Below the order of operations $$f \ g$$ is certain, while the order of $$a \
b$$ is uncertain.

$$
\lambda f \ g \ a \ b . f \ g \div a \ b \\
$$

Full expressions can be wrapped and made uncertain.

$$
\lambda \_ . \div \ (\lambda a \ b . a \ b) \ (\lambda a \ b . b \ a)
$$

### Secure Two Party Communication (S2PC)

Here we consider the two party case, between Alice ($$\color{blue}{A}$$) and
Bob ($$\color{red}{B}$$). Generally, both parties will have an input to the
function $$\lambda_o$$, here we'll call $$\color{blue} A$$'s input $$\color{blue}
x$$ and $$\color{red} B$$'s input $$\color{red} y$$.


$$
(\lambda_o \ \color{blue}{x} \ \color{red}{y} \ . e) \
\color{blue}{i_1} \
\color{red}{i_2}
\rightarrow e[\color{blue}{i_1}/\color{blue} x,\color{red}{i_2}/\color{red} y]
$$

We have up to two possible implementations for $$e$$: $$\color{blue}{e_1},
\color{red}{e_2}$$. This allows for each party to perform arbitrary
computations on the oblivious data.

Traditional function notation can be colored to indicate the same notion.

$$
\color{blue}{e_1} = \color{blue}{f(x,\color{red} y)} \\
\color{red}{e_2}  = \color{red}{g(\color{blue}x,y)}
$$

We see that each party, $$\color{blue} A$$ and $$\color{red} B$$, have a unique
function, $$\color{blue} f$$ and $$\color{red} g$$ respectively. However it is
possible to define $$e = \color{blue}{f}$$ = $$\color{red}{g}$$ These functions
are oblivious to the other party's data in these computations. We denote this
is [obliv-rust]() as follows:

```rust
f(obliv x, obliv y);
g(obliv x, obliv y);
```

One example pair of $$\color{blue} f$$ and $$\color{red} g$$ functions is a
classic problem of Oblivious Transfer (OT). In this case, $$\color{blue} A$$
wants to access a specific element from $$\color{red} B$$'s database, without
him knowing what element was retrieved. For this example, we'll use a database
projection function $$\color{red}{p}$$ (oblivious to anything else about the
database), which takes an oblivious index $$\color{blue} i$$.

$$
\lambda_o \ \color{blue}{i} \ \color{red}{p} \ . \color{red}{p} \ \color{blue}{i}
$$

Or as a solution to Yao's Millionare Problem:

$$
\lambda_o \ \color{blue}{a} \ \color{red}{b} \ . \color{blue}{a} > \color{red}{b}
$$

It's worth noting that the $$>$$ operator in the figure above is written in
black because it's "baked in" to the protocol.

### S3PC

This notion expands naturally for a third member of the computation, Carrol
($$\color{green} C$$). Similar to the S2PC case above, we can define a
$$\lambda_o$$, which now takes three arguments and tells each member what
position in line they are in a group.

$$
\lambda_o \ \color{blue}{x} \ \color{red}{y} \ \color{green}{z} \ . \\
\color{blue}{place(x, \left\{x,\color{red}y,\color{green}z\}\right)} \\
\color{red}{place(y, \left\{\color{blue}x,y,\color{green}z\}\right)} \\
\color{green}{place(z, \left\{\color{blue}x,\color{red}y,z\}\right)}
$$
