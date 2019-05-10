---
title: Fibonacci's Sequence
layout: default
---

# Fibonacci Sequence

The sequence 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987,
1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368, 75025, 121393, 196418,
317811, 514229, 832040, 1346269, 2178309, 3524578, 5702887, 9227465, 14930352,
24157817, 39088169, 63245986, 102334155 is given by:

$$
\begin{align}
  F_0 &= 0 \\
  F_1 &= 1 \\
  F_n &= F_{n-1} + F_{n-2}
\end{align}
$$

A closed form solution can be found and is rather interesting, since
$$\varphi$$ is the golden ratio (a solution to the equation $$x^2 - x - 1 =
0$$).

$$
F_n = \frac{\varphi^n - \psi^n}{\varphi - \psi}
    = \frac{\varphi^n - \psi^n}{\sqrt 5}
$$

$$
\psi = \frac{1 - \sqrt 5}{2} = -\frac{1}{\varphi}
$$

$$
\varphi = \frac{1 + \sqrt 5}{2} \approx 1.6180
$$

Naturally, the sum of the first $$n$$ numbers of the sequence can be found
rather easily if you know $$F_{n+2}$$.

$$
\sum_{i=1}^n F_i = F_{n+2} - 1
$$

In Rust:

```rust
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 1,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

Tail recursive, in Elixir:

```elixir
defmodule Math do
  def fib(n) do fib_acc(1, 0, n) end
  def fib_acc(a, b, 0) do a + b end
  def fib_acc(a, b, n) do fib_acc(b, a+b, n-1) end
end
```
