---
layout: rambling
---

Before I begin, allow me to just say that I believe communication is
fundamentally lossy. $$P(recv(send(m)) = m) \neq 1$$. As such, I should really
consider my computational models to be probabilistic, but have not been doing
so thus far for the comfort of concrete answers. Perhaps one day I will achieve
new levels of comfort in spaces which are currently scary and unknown.

Now, I'll start with a few very open ended questions which I believe are
frankly, too vague and off target to be directly answered.

- Does the world really need more languages, or simply better education?
- How can we, the developers **and** users, assert more control of our
  software?
- How can software meet modern hardware in more sustainable and practical ways?
- What am I missing?

I hope to draw nearer to an answer to some of these questions while studying
computer science closely over the course of a PhD education. These are,
however, very high level questions. Much of my work, I expect, will be aimed at
more specific and theoretical results. I will use these questions to frame a
lot of my decisions though, and guide me towards answers to questions which are
not based in purely logical results.

Working backwards...

### Hardware

- Power & complexity
- Turing machines
- LLVM
- 8 bit
- synths?

I believe that software cannot be truly understood without some appreciation
for the hardware on which it runs. This could be hardware as simple as the
Turing machine, or physical constructions of silicon vastly more complex. At
every level there are mechanics which require understanding to truly grasp the
power of the computational model. When hardware is designed well, it exposes
it's own operations clearly enough to allow those who use it to fully harness
it's capabilities. I am always fascinated to learn of these new designs, since
they often come bundled with a new way of reasoning about programs or the
mechanics of our world. In some cases, I feel like even better illustrations of
a problem warrant a new design. While they may be no more efficient, or
powerful, they may be more adoptable, or resilient.

### Ownership and Rust

- Substructural types
- permissions
- information flow
- distributed computation
- syntax coloring

One of the most powerful concepts in software design, I believe, is the notion
of _ownership_. This is partially why I'm a strong proponent of the Rust
language, but is also partially how I view all problems which involve
computation. Not only does it help reason about references and concurrency, it
also help when modeling domain specific systems, since consistent measurements
are often desired. I'm curious just how far this modeling tool can take us? To
jump to the other end of the spectrum... I don't **own** the music I pay for
anymore (generally), instead I pay for a subscription. On any given day, my
songs may disappear, having been removed from the library of that which I
subscribe.  In some ways this is no different than the reality of my own music,
which may be lost to fire, burglary, or even cosmic rays and other forms of bad
luck. There is a difference however, I just don't know exactly how to express
it. Somewhere there exists a party to blame, I'm just not always sure who it
is.

Having worked for a company who's famous for walled gardens, and who I believe
is now quite hypocritically subjugating it's users to remain inside, I would
like to help work towards a brighter future. A future where users and
developers both have access to the devices they own. 

### Language and Education

- lambda calculus
- first order -> higher order logic
- translation
- documentation
- teaching

As an example, I find myself very interested in the process of standardization
or consensus in general. While I'm aware of some results, I am not convinced
that standards are always created with the right specificity, or for the right
problems. Programming languages are often used in a way constrain general
computation to a more specific domain. Even if we all spoke English, I'm not
convinced we'd all understand each other's context well enough to communicate
effectively. I like to think of all communication as fundamentally lossy.
Perhaps this will force me to think about probabilistic computation one day.
But for now, I quite like the concrete answers that lie in the land of the
discrete.

If you load the right language into your head, when you see

$$
\lambda x.(x\ x)\ \lambda x.(x\ x)
$$

you don't just see a sequence of glyphs, but also a loop which carries on
forever. This short expression demands close attention when building a type
system, and is somewhat inspiring to me.
