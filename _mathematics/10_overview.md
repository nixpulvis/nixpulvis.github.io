---
title: Overview of Mathematics
layout: default
---

## Overview of Mathematics

Mathematics makes use of symbols and names somewhat liberally, allowing for a
certain amount of conciseness and expressiveness, but not always
disambiguation. Notion can be context sensitive, be careful.

Also, this is **not** a guide, or a lesson. I'm mostly trying to catalog useful
notation and expressions.

**Reader Be Warned**

1. [Logic](#logic)
    1. Symbols
    2. Rules
    3. Proofs
2. [Mathematics](#mathematics)
    1. Notation
        1. Objects
        2. Operations & Expressions
        3. Conventions
    2. Subjects
        1. Algebra
        2. Calculus
        3. Linear Algebra
        4. Statistics
        5. Combinatorics
        6. Set/Group/Category Theory
        7. Real Analysis*
3. Programming
4. Graphics and Plotting
5. Abstraction & Application

---

# Logic

Logic is the foundation of all reasoning and thought. Though, not all logics
are created equal.

Formal logic denotes a collection of symbols and rules for these symbols. Used
carefully, one can start here and build whole models of mathematics and thus
anything.

### Symbols

##### Negation: $$\neg A$$, $$!A$$
"Not" is written as $$\neg$$ (or $$!$$ by programmers). For example, all roads
do **not** lead to Rome. Let's call this proposition $$\neg R$$.

##### Equivalence: $$A \equiv B$$, $$A \leftrightarrow B$$
We could say that all roads _do_ in fact lead to Rome, and that would be $$\neg
\neg R \equiv R$$. This could also be written as $$\neg \neg R \leftrightarrow
R$$ (or even $$\neg \neg R \sim R$$ in Kleene's "Mathematical Logic").

##### Junction: $$A \land B$$, $$A \lor B$$
"And" and "Or" are written as $$\land$$, and $$\lor$$, respectively. We
obviously can't claim $$R \land \neg R$$ is true.

##### Implication: $$A \Rightarrow B$$, $$A \rightarrow B$$
_(also $$A \supset B$$ in Kleene's "Mathematical Logic")_

##### Existentialization: $$\exists A$$
##### Universilization: $$\forall A$$

##### Proof: $$\vdash A$$
##### Entailment: $$\vDash A$$
We can now formally state in our observer language, $$\nvDash R \land \neg R$$,
for example.

##### Approximation: $$\sim A$$, $$\approx A$$, $$O$$
Somewhere along the way to Rome I might ask, "How long has it been since we
left camp?", which, unless someone or something was perfectly counting, could
only be met with $$\sim 12$$ hours. It might also be useful to write $$t
\approx 12$$. Looking over at the travelers along side us, we recognize a few
from hours ago when we left. We could say our distance ($$d_1$$) and their
distance ($$d_2$$) are related as $$d_1(t) \sim d_2(x)$$, or even $$O(d_1(t)) =
O(d_2(t))$$.


# Mathematics

### Notation

##### Objects

Numbers are the primary objects of math. You can use numbers to count, for
example, $$1, 2, 3$$, Go! You can use numbers for measures of units, for
example, the current temperature $$65°F$$.

Fractions are for frogs, $$\frac{3}{7}$$, ribbit.

##### Operations & Expressions

Math is full of various operations you can write into valid (or invalid)
mathematical notation. Values can be computed, like $$\pi^3$$, or $$\sqrt[3]
2$$.

##### Equations

All hail the mighty equal sign, $$=$$. Simply put, the equal sign says two
things are identical, or equal. We call this an equation.

A really obvious equation might be $$2 + 2 = 4$$, while we can represent
something as complex as the concept of a limit as simply as $$1 =
0.9999999\dots$$ or $$1 = 0.\overline{999}$$.

##### Variables

Many equations have letters in them, which can allow a fixed value to change
with the variable, for example $$x^2$$ grows quickly, but $$2^x$$ even faster.
Some variables are used to represent physical, or conceptual properties, for
example $$a^2 + b^2 = c^2$$ describes the edge lengths of a triangle, where
$$a$$ and $$b$$ and the sides, and $$c$$ is the hypotenuse.

$$
k_{n+1} = n^2 + k_n^2 - k_{n-1}
$$

## Summation and Production

$$
\sum_{i=0}^n f(i) \hspace{5em} \prod_{i=1}^n g(i)
$$

## Vectors and Matrices

Vectors come in a many forms. For example as a $$\vec{v}$$ or as three (in this
case) coefficients along with component unit vectors starting with $$\hat{i},
\hat{j}, \hat{k}, \ldots$$.

$$
\vec{v} = [10, 25, -37.2]
        = 10\hat{i} + 25\hat{j} - 37.2\hat{k}
$$

Vectors may also be written in matrix form as $$A$$ or $$B$$.

$$
\begin{align}
A_n &\in [a_1, a_2, \ldots, a_n] \\
B_n &\in [b_1, b_2, \ldots, b_n] \\\\
A + B      &= (a_1 + b_1, a_2 + b_2, \ldots, a_n + b_n) \\
A \times B &= \hat{n} \left\|A\right\| \left\|B\right\| \sin(\theta)
  \hspace{3em}\rlap{\in \mathbb{R}^3} \\
A \cdot B  &= \sum_{i=1}^n a_i b_i \\
\end{align}
$$

Matrices are just vectors of vectors.

$$
A =
  \begin{pmatrix}
    a_{11}  & a_{12} & \cdots & a_{1n} \\
    a_{21}  & a_{22} & \cdots & a_{2n} \\
    \vdots  & \vdots & \ddots & \vdots \\
    a_{m1}  & a_{m2} & \cdots & a_{mn}
 \end{pmatrix} \\
$$

Transpose

$$
(A^T)_{ij} = A_{ji}
$$

$$
(A^T)^T = A
$$

Addition

$$
A + B =
 \begin{pmatrix}
  a_{11} + b_{11} & a_{12} + b_{12} & \cdots & a_{1n} + b_{1n} \\
  a_{21} + b_{21} & a_{22} + b_{22} & \cdots & a_{2n} + b_{2n} \\
  \vdots          & \vdots          & \ddots & \vdots          \\
  a_{m1} + b_{m1} & a_{m2} + b_{m2} & \cdots & a_{mn} + b_{mn}
 \end{pmatrix} \\
$$

$$
\begin{align}
A + 0     &= A \\
A + B     &= B + A \\
A + B + C &= (A + B) + C = A + (B + C) \\
(A + B)^T &= A^T + B^T
\end{align}
$$

Scalar Multiplication

$$
\lambda A =
 \begin{pmatrix}
  \lambda a_{11} & \lambda a_{12} & \cdots & \lambda a_{1n} \\
  \lambda a_{21} & \lambda a_{22} & \cdots & \lambda a_{2n} \\
  \vdots         & \vdots         & \ddots & \vdots         \\
  \lambda a_{m1} & \lambda a_{m2} & \cdots & \lambda a_{mn}
 \end{pmatrix} \\\\
$$

$$
\begin{align}
        y &= Ax \hspace{5em}x\in\mathbb{R}^n, \hspace{.5em}y\in\mathbb{R}^m \\
\lambda A &\neq A \lambda
\end{align}
$$

Matrix Multiplication

$$
AB = \sum_{k=1}^p a_{ik} b_{kj} \hspace{4em}i = 1, \ldots, m,
                                \hspace{.5em}j = 1, \ldots, n
$$

$$
\begin{align}
0A       &= 0, A0 = 0 \\
IA       &= A, AI = A \\
ABC      &= (AB)C = A(BC) \\
A(B + C) &= AB + AC \\
(AB)^T   &= B^T A^T \\
\end{align}
$$

Exponentiation

$$
\begin{align}
A^0     &= I \\
A^k     &= \prod_{i=1}^k A \\\\
\end{align}
$$

$$
A^k A^l = A^{k+l} \\
$$

Inverse

$$
\begin{align}
A^{-1} A         &= I \\
(A^{-1})^{-1}    &= A
\end{align}
$$

$$
\begin{align}
(AB)^{-1}        &= B^{-1} A^{-1} \\
(A^T)^{-1}       &= (A^{-1})^T \\
I^{-1}           &= I \\
(\lambda A)^{-1} &= (1/\lambda) A^{-1}
\end{align}
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
