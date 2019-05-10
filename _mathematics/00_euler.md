---
title: Euler's Identity
layout: default
---

# Euler's Identity

$$
e^{i\pi} + 1 = 0 \\
$$

This equation is supremely beautiful and bewildering. It uses 5 **very**
fundamental objects: $$0$$, $$1$$, $$\pi = 3.141\ldots$$, $$e = 2.718\ldots$$,
and $$i = \sqrt{-1}$$, along with 4 operations: $$a + b$$ (addition), $$a b$$
(multiplication), $$a^b$$ (exponentiation), and finally $$a = b$$ (equality,
identity).

$$
                       e^{\frac{i\pi}{2}}    = i  \\
e^{i\pi}            = (e^{\frac{i\pi}{2}})^2 = i^2 = -1  \\
e^{\frac{3i\pi}{2}} = (e^{\frac{i\pi}{2}})^3 = i^3 = -i  \\
e^{2i\pi}           = (e^{\frac{i\pi}{2}})^4 = i^4 = 1 \\
e^{\frac{5i\pi}{2}} = (e^{\frac{i\pi}{2}})^5 = i^5 = i
$$

If you can't see the "whole" picture yet (can anyone?), this might help. Our
cyclic circular sidekicks, $$\sin$$ and $$\cos$$ are here to enlighten us.

$$
\begin{align}
              e^{i \theta} &= \cos(\theta) + i \sin(\theta) \\\\
  \frac{\sin(\theta)}{\pi} &= \frac{e^{i \theta} - \cos(\theta)}{i\pi}
\end{align}
$$

Substitute $$\pi$$ for $$\theta$$.

$$
\frac{\sin(\pi)}{\pi} = \frac{e^{i \pi} - \cos(\pi)}{i\pi}
                      = \frac{0}{i\pi}
                      = 0
$$

```
Plot[
        sin(theta)/pi,
  (3/2) sin(theta)/pi,
  2     sin(theta)/pi,
  (5/2) sin(theta)/pi
]
```
![](/img/fourier_euler.png)

<details>
  <summary><strong>Tangent on Fourier Series</strong></summary>
<p>
  Unlike the Euler series we above, which is purely constructive, a <b>square
  wave</b> can be constructed with the Fourier series:

  $$
  \sin(2 \pi \theta),
  \frac{1}{3} \sin(6 \pi \theta),
  \frac{1}{5} \sin(10 \pi \theta),
  ...
  $$

  <pre><code>
  Plot[
          sin(2 pi theta),
    (1/3) sin(6 pi theta),
    (1/5) sin(10 pi theta)
  ]
  </code></pre>

  <img src="/img/fourier_square.png" />
  <img src="/img/fourier_square.gif" />
</p>

<p>
  And a <b>saw wave</b> can be constructed by another:

  $$
  \sin(\pi \theta),
  \sin(\frac{\pi \theta}{2}),
  \sin(\frac{\pi \theta}{4}),
  \ldots
  $$

  <pre><code>
  Plot[
    sin(pi theta),
    sin((pi theta) / 2),
    sin((pi theta) / 4)
  ]
  </code></pre>

  <img src="/img/fourier_saw.png" />
  <img src="/img/fourier_saw.gif" />
</p>
</details>

Euler's identity can be geometrically interpreted as saying that rotating any
point $$\pi$$ radians around an origin of a complex plane has the same effect
as reflecting the point across the origin.

More generally, for the $$n$$th roots of unity, we can write Euler's
Identity as the $$n = 2$$ case of the equation:

$$
\sum_{k=0}^{n-1} e^{2\pi i\frac{k}{n}} = 0 \\
\therefore \because
$$
