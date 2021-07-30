---
layout: rambling
---

I believe language is how we come to understand the world we live in.  While
the student adopts the languages of their teachers, we must impose our language
directly upon machines which are constructed from matter found on Earth.  These
programming languages, in theory, provide us the machine interface to command
unquestionably; bound only by feasibility and creativity.  Given this immense
world-building power, it is important to decide what aspects we want in the
language, what cases we wish to avoid, and what properties are unavoidable.
These natural consequences of programming language design are fascinating to
me.  In particular, I have become intrigued by the concept of the reference,
much as how I felt enlightened by the function itself.  Taming references
allows a programming language with mutation to act more like a purely
functional language and gives the programmer more control.  One thread I would
like to follow in my research is how substructural type systems can be weaved
together with other interesting concepts.  I have decided to pursue a research
path so I can explore ideas with others who share similar visions and who will
challenge me along the way.

To help guide my research I have come up with a few very high level questions
which should give rise to more specific hypotheses for study.  I do not expect
to receive a direct answer to these questions, but I hope they help keep me on
the right path.

- Does the world really need more languages or simply the right ones?  How can
- we, the developers and users alike, assert more control over our
  software?
- How can software meet modern hardware in more sustainable and practical ways?

More concrete questions should motivate my research. For example, would the
shell programming environment be substantially improved by the ability to embed
other programming languages inline?  Are there valid metrics for assessing the
gain from this approach over the existing methods for calling external
programs?  And perhaps most interestingly, what design patterns become more
naturally expressed in this context?

One project I wish to continue working on over the years is my shell, oursh,
which aims to address some of these questions, as well as remain POSIX
compatible.  The high level idea is somewhat simple, by adding "shebang
blocks", the programmer can write arbitrary languages inline, just as they
would write any other expression.  I have worked on many other projects,
between school projects, personal projects, professional projects, and open
source projects.  Some are simply playgrounds for exploring, while others are
critical tools.  Alacritty, the open source terminal emulator I am writing this
document with, is an example of something critical I use on a daily basis and
contribute to from time to time.  I have also written a number of parsers and
small languages.  So many problems are more concretely understood in the frame
of parsing and serialization, since exhaustive case analysis can be done and
ambiguity can be detected. I believe this makes for much more reliable systems
in practice.

My background is currently somewhat mixed between research and industrial
working experience.  Between the three, semester-long co-op/internships during
my undergraduate education and the few years of work afterwards, I have around
5 years under my belt.  During my junior year and after graduation, I moved to
the San Francisco bay area to work for Apple.  I was on a small team within the
iPhone hardware team whose job it was to write supporting software tools for
project management, development and evaluation of the hardware.  One of my
tasks was to write what amounted to an ORM for Radar (Apple's bug management
service), which was rewarding since it allowed me to replace a massive amount
of ad-hoc code with a cohesive library.  There were some very exciting parts of
my job, but I was becoming quickly drained by the inability to talk with people
about my work and the general culture of secrecy.  I had always planned on
going back to graduate school, so I decided I needed start looking for my way
into academia. This led me back to Boston and Northeastern University.

For the last year or so I have been working at the intersection of Security and
Programming Languages at Northeastern, developing a novel language for
Multiparty Computation (MPC).  This project started under IARPA's HECTOR
program, but has continued with our own focus on designing a language with
explicit information flow between parties and secure conditional evaluation.
While working on this project, I had to balance both research questions and
mandated deliverables, some of which were not directly relevant to our
research.  Despite the fact that the term "multiparty computation" could mean a
variety of things, it has somewhat specific meaning in the security community
and I had to learn to speak their language.  This was especially critical since
we are designing a programming language to assist with the kind of work they
do.  One of our central contributions is the notion of a virtual party, which
simulates a trusted third party who can perform computations on data
themselves.  Of course, as the name implies, the virtual party doesn't actually
exist, and the compiler generates the corresponding program for the real
parties.  By modeling a language with explicit participants of computation and
allowing them to compose shared virtual participants themselves, we can encode
a wide class of programs such as recursively defined protocols, and hybrid
programs.  The concept of a virtual party, implemented with sound cryptography,
gives an appealing way to express programs without free access to information.
The task of keeping a system operating without complete knowledge becomes
significantly harder.  There will always be a risk of program failure, and
there may not be a single right answer to many questions of design.  Still, we
should strive for making systems both secure and accessible where possible and
on giving tools to programmers for controlling access to their user's data.
What I mean by accessibility is roughly the following:

- Language is fundamentally about encoding information, thus making it
  accessible
- Multi-language programming aims to create a context which is accessible to
  more people
- Privacy requires inaccessibility in some form or another

Perhaps accessibility simply means technology should "just work". In order to
achieve that we must all agree on our definitions, just as a conversation
without the need for a dictionary feels more natural.  Having personally
discussed, debated, argued, and come to terms in both academia and industry
alike, I have seen how much energy this can take.  The ability to check our own
understanding before attempting to interface with each other is often a crucial
step in the process.  I see this as analogous to the static type checking of
many programming languages.  These types give us the ability to restrict our
thoughts to what matters and focus on making things work.

Reflecting back on some of my guiding questions, there seems to still be a
chasm between where we are today and where I would like to be.  I would like to
see a world where building secure applications is easy enough that users
themselves can trust their own devices again.  Not everyone is going to be a
security expert and not everyone is even going to understand the first thing
about our languages.  It is the average developer, however, looking to make
something for their constituents who I would like to see empowered to protect
user's integrity.  To accomplish this, I see no other way than for an effort to
be made in industry based on solid foundations I hope to help develop with my
research one day.
