---
title: Euler's Identity
layout: default
---

# Euler's Identity

$$
e^{\pi i} + 1 = 0
$$

This equation is supremely beautiful and bewildering. It uses 5 **very**
fundamental objects: $$0$$, $$1$$, $$\pi = 3.141\ldots$$, $$e = 2.718\ldots$$,
and $$i = \sqrt{-1}$$, along with 4 operations: $$a + b$$ (addition), $$a b$$
(multiplication), $$a^b$$ (exponentiation), and finally $$a = b$$ (equality,
identity).

$$
                        e^{\frac{\pi i}{2}}    = i  \\
e^{\pi i}            = (e^{\frac{\pi i}{2}})^2 = i^2 = -1  \\
e^{\frac{3\pi i}{2}} = (e^{\frac{\pi i}{2}})^3 = i^3 = -i  \\
e^{2\pi i}           = (e^{\frac{\pi i}{2}})^4 = i^4 = 1 \\
e^{\frac{5\pi i}{2}} = (e^{\frac{\pi i}{2}})^5 = i^5 = i
$$

If you can't see the "whole" picture yet (can anyone?), this might help. Our
cyclic circular sidekicks, $$\sin$$ and $$\cos$$ are here to enlighten us.

Euler's identity is a special case of _Euler's formula_, which states:

$$
e^{i \theta} = \cos(\theta) + i \sin(\theta)
$$

Substitute $$\pi$$ (or in general, $$n\pi$$) for $$\theta$$ and we get back our
identity.

$$
\begin{align}
  \cos(\pi) + i \sin(\pi) = -1 + 0 =\ & e^{\pi i} = i^2 \\
                                     & e^{\pi i} + 1 = 0
\end{align}
$$

<details>
  <summary>Tangent on Fourier Series</summary>
<p>
  <pre><code>
  Plot[
          sin(theta)/pi,
    (3/2) sin(theta)/pi,
    2     sin(theta)/pi,
    (5/2) sin(theta)/pi
  ]
  </code></pre>

  <img src="/img/fourier_euler.png" />
</p>
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

More generally, Euler's Identity is a second-root of unity, meaning that it is
a solution for $$z$$ in the equation $$z^n = 1$$ where $$n = 2$$.

This is the definition for the $$n$$th root of unity.

$$
\sum_{k=0}^{n-1} e^{2\pi i\frac{k}{n}} = 0
$$

<details>
  <summary>
    Wanna compute <strong><i>e</i></strong> in
    <a href="http://nixpulvis.com/brainfuck">Brainfuck</a>?
  </summary>
$$
\hphantom{nothing} \\
e = 2.718281828459\ldots
$$

<pre><code>
git clone https://github.com/nixpulvis/brainfuck
cd brainfuck
cargo run fixtures/e.bf
</code></pre>

<pre><code>
>>>>++>+>++>+>>++<+[
  [>[>>[>>>>]<<<<[[>>>>+<<<<-]<<<<]>>>>>>]+<]>-
  >>--[+[+++<<<<--]++>>>>--]+[>>>>]<<<<[<<+<+<]<<[
    >>>>>>[[<<<<+>>>>-]>>>>]<<<<<<<<[<<<<]
    >>-[<<+>>-]+<<[->>>>[-[+>>>>-]-<<-[>>>>-]++>>+[-<<<<+]+>>>>]<<<<[<<<<]]
    >[-[<+>-]]+<[->>>>[-[+>>>>-]-<<<-[>>>>-]++>>>+[-<<<<+]+>>>>]<<<<[<<<<]]<<
  ]>>>+[>>>>]-[+<<<<--]++[<<<<]>>>+[
    >-[
      >>[--[++>>+>>--]-<[-[-[+++<<<<-]+>>>>-]]++>+[-<<<<+]++>>+>>]
      <<[>[<-<<<]+<]>->>>
    ]+>[>>>>]-[+<<<<--]++<[
      [>>>>]<<<<[
        -[->--[<->+]++<[[>-<+]++[<<<<]+>>+>>-]++<<<<-]
        >-[+[<+[<<<<]>]<+>]+<[->->>>[-]]+<<<<
      ]
    ]>[<<<<]>[
      -[
        -[
          +++++[>++++++++<-]>-.>>>-[<<<----.<]<[<<]>>[-]>->>+[
            [>>>>]+[-[->>>>+>>>>>>>>-[-[+++<<<<[-]]+>>>>-]++[<<<<]]+<<<<]>>>
          ]+<+<<
        ]>[
          -[
            ->[--[++>>>>--]->[-[-[+++<<<<-]+>>>>-]]++<+[-<<<<+]++>>>>]
            <<<<[>[<<<<]+<]>->>
          ]<
        ]>>>>[--[++>>>>--]-<--[+++>>>>--]+>+[-<<<<+]++>>>>]<<<<<[<<<<]<
      ]>[>+<<++<]<
    ]>[+>[--[++>>>>--]->--[+++>>>>--]+<+[-<<<<+]++>>>>]<<<[<<<<]]>>
  ]>
]

This program computes the transcendental number e, in decimal. Because this is
infinitely long, this program doesn't terminate on its own; you will have to
kill it. The fact that it doesn't output any linefeeds may also give certain
implementations trouble, including some of mine.

(c) 2016 Daniel B. Cristofani
http://brainfuck.org/
</code></pre>
</details>
