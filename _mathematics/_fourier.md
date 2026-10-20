TODO: This information is only a sketch, and as such is somewhat sketchy.

```
Plot[
        sin(theta)/pi,
  (3/2) sin(theta)/pi,
  2     sin(theta)/pi,
  (5/2) sin(theta)/pi
]
```

![Plot of four sine waves of increasing amplitude]({{ '/img/fourier_euler.png' | relative_url }})

Unlike the "Euler series" we above, which is purely constructive, a <b>square
wave</b> can be constructed with the Fourier series:

$$
\sin(2 \pi \theta),
\frac{1}{3} \sin(6 \pi \theta),
\frac{1}{5} \sin(10 \pi \theta),
...
$$

```
Plot[
        sin(2 pi theta),
  (1/3) sin(6 pi theta),
  (1/5) sin(10 pi theta)
]
```

![Plot of sin(x), sin(3x)/3 and sin(5x)/5 for a square wave]({{ '/img/fourier_square.png' | relative_url }})
![Animation of rotating circles drawing a square wave]({{ '/img/fourier_square.gif' | relative_url }})

And a <b>saw wave</b> can be constructed by another:

$$
\sin(\pi \theta),
\sin(\frac{\pi \theta}{2}),
\sin(\frac{\pi \theta}{4}),
\ldots
$$

```
Plot[
  sin(pi theta),
  sin((pi theta) / 2),
  sin((pi theta) / 4)
]
```

![Plot of the first few sine terms for a sawtooth wave]({{ '/img/fourier_saw.png' | relative_url }})
![Animation of rotating circles drawing a sawtooth wave]({{ '/img/fourier_saw.gif' | relative_url }})
