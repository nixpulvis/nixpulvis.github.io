---
title: Overview
layout: default
draft: true
---

# Logic

Basically start with `true` and `false`, then lots more.

$$
\forall x \in X, \quad \exists i \leq \epsilon
$$

# Mathematics

## Numbers

Numbers are the primary objects of math. You can use numbers to count, for
example, $$1, 2, 3$$, Go! You can use numbers for measures of units, for
example, the current temperature $$65°F$$.

Fractions are for frogs, $$\frac{3}{7}$$, ribbit.

## Expressions

Math is full of various operations you can write into valid (or invalid)
mathematical notation. Values can be computed, like $$\pi^3$$, or $$\sqrt[3]
2$$.

## Equations

All hail the mighty equal sign, $$=$$. Simply put, the equal sign relates two
different forms to the same underlying truth. We call this an equation.

A really obvious equation might be $$2 + 2 = 4$$, while we can represent
something as complex as the concept of a limit as simply as $$1 =
0.9999999\dots$$ or $$1 = 0.\overline{999}$$.

### Variables

Many equations have letters in them, which can allow a fixed value to change
with the variable, for example $$x^2$$ grows quickly, but $$2^x$$ even faster.
Some variables are used to represent physical, or conceptual properties, for
example $$a^2 + b^2 = c^2$$ describes the edge lengths of a triangle, where
$$a$$ and $$b$$ and the sides, and $$c$$ is the hypotenuse.

$$
k_{n+1} = n^2 + k_n^2 - k_{n-1}
$$

## Summation and Product Notation

$$
\sum_{k=1}^n k^2 = \frac{1}{2} n (n+1)
$$

$$
\prod_{i=a}^{b} f(i)
$$

## Vectors and Matrices

$$
\vec{o} = [1, 2, -1]
$$

$$
A_{m,n} =
 \begin{pmatrix}
  a_{1,1} & a_{1,2} & \cdots & a_{1,n} \\
  a_{2,1} & a_{2,2} & \cdots & a_{2,n} \\
  \vdots  & \vdots  & \ddots & \vdots  \\
  a_{m,1} & a_{m,2} & \cdots & a_{m,n}
 \end{pmatrix}
$$

## Combinatorics

$$
\binom{n}{k} = \frac{n!}{k!(n-k)!}
$$

## Limits

A $$\lim_{x\to\infty}$$ is a way to talk about the convergence of something.

$$
\lim_{x\to\infty} f(x)
$$

## Derivatives

$$
\frac{\partial u}{\partial t}
   = h^2 \left( \frac{\partial^2 u}{\partial x^2}
      + \frac{\partial^2 u}{\partial y^2}
      + \frac{\partial^2 u}{\partial z^2} \right)
$$

## Integration

Integral $$\int_{a}^{b} x^2 dx$$ inside text looks nice too.

$$
\frac{d}{dx}\left( \int_{0}^{x} f(u)\,du\right)=f(x).
$$

## Cases

This is where things can get "fun".

$$
f(n) =
  \begin{cases}
    n/2       & \quad \text{if } n \text{ is even}\\
    -(n+1)/2  & \quad \text{if } n \text{ is odd}
  \end{cases}
$$
