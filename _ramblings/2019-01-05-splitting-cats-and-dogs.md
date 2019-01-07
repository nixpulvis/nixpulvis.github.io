---
title: Splitting Cats and Dogs
layout: rambling
---

Often when you're programming you find the need to split things up a bit. This
is such a fundamental part of programming that almost every language gives you
many tools to aid with this. Maybe it's _modules_, _classes_, _impls_, or
even _packages_. The list goes on... In fact, one of the best forms of
splitting can be a simple _function_. Even _comments_ can play a role in this
if you'd like.

There's a number of reasons you'd want to split up some part of a program. You
might be looking at a class with the need for countless functions. Instead of
defining them all one after another, it makes sense to _group_ the functions
and move them somewhere so it's clear how they are related. Or, you might be
looking to _share_ some logic across a few classes. This is key to writing
good software, since maintaining two versions of the same thing is cumbersome.
Remember, stay DRY (Don't Repeat Yourself).

### Grouping

Grouping shouldn't require too much thought, aside from a **name** for the way
in which the functions are grouped. It can often be all to easy to end up with
needless wrapper classes, or other added machinery. Another good acronym to
keep around is, KISS (Keep It Simple, Stupid). A bit aggressive, but that's OK,
this is important shit.

Consider an example:


```ruby
class Dog
  def bark
    # ...
  end

  def growl
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

  def pee
    # ...
  end
end
```

I think it's pretty clear here what functions should be grouped with each other
(this isn't always so clear), and all we need to do is split them up in some
way.  Methods like `bark` and `growl` are clearly concerned with dog
communications, while methods like `eat` and `poop` are concerned with the
sustaining life at the most basic level.

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

  def pee
    # ...
  end
end

# Or, even better in my book.

class Dog
  module Communication
    def bark
      # ...
    end

    def growl
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

    def pee
      # ...
    end
  end

  include Communication
  include Sustenance
end
```

People often view a `module` as a way to share functionality, but here we're
just using it to group similar functions. The main thing we've done is make it
clear which functions are related, and therefor **a good idea to read
together**.  If you want to share some part of your program's design, then
you'll need to think more, though a `module` may still be the solution.

### Sharing

Sharing is about *abstraction*. It's about defining an interface, and letting
users rely on it. Sharing code lets you keep things DRY and maintainable, and
also helps keep testing focused.

> It's a fun fact that in theoretical computer science, the term abstraction
> and function can be used interchangably. The dual, also called an elimination
> rule, is _application_, or calling the function. With only these two things
> you've got yourself a turing complete language. Cool!

Let's consider what happens when we want to add a new `Cat` class to our
program. The first thing we should notice is that we can share all of the
`Sustenance` functionality, since just like dogs, cats also eat, drink, and the
rest.

Here's a complete implementation of the `Sustenance` module we'll share:

```ruby
module Sustenance
  def initialize
    @stomach = []
    super
  end

  def eat(food)
    @stomach << { food: food }
  end

  def drink(drink)
    @stomach << { drink: drink }
  end

  def poop
    digest_to_completion(:food, "can't poop")
  end

  def pee
    digest_to_completion(:drink, "can't pee")
  end

  private

  def digest_to_completion(kind, error_message)
    victuals = @stomach.select { |v| v[kind] }

    if victuals.empty?
      raise error_message
    end

    @stomach = @stomach - victuals
  end
end
```

Notice how we've made use of a Ruby instance variable here. It's a really good
idea to keep the use of these as close to each other as possible. It's very
hard to reason about a module that's touching another module's state. Keep
spooky action at a distance out of our programs!

Now everything is wrapped up nicely inside this module, and all we have to do
is include it, like we did when we used a `module` for grouping before. The
only difference is that the `Sustenance` module now must be visible to both the
`Cat` and `Dog` classes.

```ruby
class Dog
  include Sustenance

  # ...
end

class Cat
  include Sustenance
end
```

### Testing

With all this in mind, let's see how we test the `Dog` class and the
`Sustenance` module. Each should only test the parts they are concerned with
directly.

First, since all of this code should run, let's `require` Ruby's testing
framework.

```ruby
require 'minitest/autorun'
```

Next, we can define the `Dog` spesific tests. These are free to do anything
they'd like with the dog, but shouldn't test `Sustenance` since that's defined
elsewhere.

```ruby
class TestDog < Minitest::Test
  def setup
    @dog = Dog.new
  end

  # Implemention needed, since we left it as "# ...".
  def test_bark
    assert_equal @dog.bark, 'Woof'
  end

  # Tests ommited...
end
```

Finally, we can test the `Sustenance` module. We want to test this functionality
without relying on our `Dog` or `Cat` since they aren't the concern of this
module. A dummy, class can be created just for our test, making it very clear
what's going on.

```ruby
class TestSustenance < Minitest::Test
  class TestAnimal
    include Sustenance
    attr_reader :stomach
  end

  def setup
    @animal = TestAnimal.new
  end

  def test_eat
    assert_empty @animal.stomach
    @animal.eat("pepper")
    refute_empty @animal.stomach
  end

  def test_drink
    assert_empty @animal.stomach
    @animal.drink("slurppy")
    refute_empty @animal.stomach
  end

  def test_poop
    @animal.eat("sand")
    refute_empty @animal.stomach
    @animal.poop
    assert_empty @animal.stomach
  end

  def test_pee
    @animal.drink("soda")
    refute_empty @animal.stomach
    @animal.pee
    assert_empty @animal.stomach
  end
end
```

These tests don't cover enough cases, but you get the idea. We seem to have
a functioning set of animals on our hands!

### Conventions

TODO: Conventions vs the World, i.e. my convention says to put things here, but
      I want to put things there.

### Footnote for Rails Programmers

If you're a Rails programmer you've surely run into a `Concern` or two. In fact
concerns are really just a nice set of helpers (a DSL even) for a plain ol'
Ruby `module`. Let's see how this looks.

TODO: Concerns
