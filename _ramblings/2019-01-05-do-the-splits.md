---
title: Do the Splits
layout: rambling
---

Often when you're programming you find the need to split things up a bit. This
is such a fundamental part of programming that almost every language gives you
many tools to aid with this. Maybe it's _modules_, _classes_, _impls_, or
even _packages_. The list goes on... In fact, one of the best forms of
splitting can be a simple _function_. Even _comments_ can play a role in this
if you'd like.

There's a number of reasons you'd want to split up some part of a program. You
might be looking at a class with countless functions, all defined one after
another and decide it makes sense to _group_ the functions and move them
somewhere so it's clear they are related. Or, you might be looking to _share_
some logic across a few classes.

### Grouping

Grouping shouldn't require too much thought, aside from a **name** for the way
in which the functions are grouped. It can often be all to easy to end up with
needless wrapper classes, or other added machinery in this case.

Consider an example:


```ruby
class Dog
    def bark
        # ...
    end

    def growl
        # ...
    end

    def wimper
        # ...
    end

    def eat
        # ...
    end

    def drink
        # ...
    end

    def poop
        # ...
    end
end
```

I think it's pretty clear here what functions are grouped with each other (this
isn't always so clear), and all we need to do is split them up in some way.

Here's a couple options:

```ruby
class Dog
    # Communication Methods #
    #########################

    def bark
        # ...
    end

    def growl
        # ...
    end

    def wimper
        # ...
    end

    # Sustenance Methods #
    ######################

    def eat
        # ...
    end

    def drink
        # ...
    end

    def poop
        # ...
    end
end

# Or

class Dog
    module Communication
        def bark
            # ...
        end

        def growl
            # ...
        end

        def wimper
            # ...
        end
    end

    module Sustenance
        def eat
            # ...
        end

        def drink
            # ...
        end

        def poop
            # ...
        end
    end

    include Communication
    include Sustenance
end
```

People often view a `module` as a way to share functionality, but here we're
just using it to group similar functions. If you want to share some part of
your program's design, then you'll need to think more, though a `module` may
still be the solution.

### Sharing

Sharing is about *abstraction*. It's about defining an interface, and letting
users rely it. It's a fun fact that in theoretical computer science, the term
abstraction and function can be used interchangably. The dual, also called an
elimination rule, is _application_, or calling the function. With only these two
things you've got yourself a turing complete language. Cool!

TODO: examples

### Testing

Since it's been a topic of some debate, let's talk about testing in this
context.


### Conventions

TODO: Conventions vs the World, i.e. my convention says to put things here, but
      I want to put things there.

### Footnote for Rails Programmers

If you're a Rails programmer you've surely run into a `Concern` or two. In fact
concerns are really just a nice set of helpers (a DSL even) for a plain ol'
Ruby `module`. Let's see how this looks.

TODO: Concerns
