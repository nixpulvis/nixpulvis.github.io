TODO: This information is only a sketch, and as such is somewhat sketchy.

```
Plot[
        sin(theta)/pi,
  (3/2) sin(theta)/pi,
  2     sin(theta)/pi,
  (5/2) sin(theta)/pi
]
```

![](/img/fourier_euler.png)

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

![](/img/fourier_square.png)
![](/img/fourier_square.gif)

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

![](/img/fourier_saw.png)
![](/img/fourier_saw.gif)
