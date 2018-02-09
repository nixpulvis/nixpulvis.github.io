---
layout: project
title: Epic UI
---

I used to be an avid World of Warcraft player, raiding bosses many times a
week with my guild. In order to get the most out of my own performance I forked
an advanced UI replacement addon named [Tukui][tukui] into my own project, I
named Epic UI (after my gamertag, Epicgrim).

### Boxy Beginnings

Before my days with Tukui and EpicUI I did what many WoW players did, and
customized my UI with addons that let me drag around boxes, and put things
inside them. All from within the game, and without a single line of code.
**At this point in my life I had never written any code.**

![](/img/epic-ui/pretukui.jpg)

As you can see the result was a very "boxy" UI, and not one that I was very
happy with. I was developing an opinion that the raider (PvE WoW player) is
only as good as his UI lets him be. It was crucial that I find a way to show
the needed information, without so much clutter.

### Let There be Code!

After I discovered Tukui, it only took my a little while before I downloaded
Notepad++ (the OG editor) and started making my own modifications. I didn't
know LUA, but for the basic things like moving a UI element somewhere else the
code was simple.

```lua
-- Position `element` just below `another_element`.
element:SetPoint("TOP", another_element, "BOTTOM", 0, -10)
```

Since so much of UI programming is just a matter of getting things into the
correct place, this got me off the ground running. As I needed to make more and
more changes I learned more and more about LUA, and the [WoW APIs][wow-api]
(some of the best APIs I've worked with to date).

An early version of Epic UI can be seen below. The unitframes (boxes with a
claw next to them in the screenshot) have player info in the middle, a design
I'm glad to have dropped.

![](/img/epic-ui/early.jpg)

Many other features were added, most notably a "buff" tracker (under the
minimap), to ensure I had every advantage possible before each boss fight.
Many of these features were collaberations with people on the Tukui forums,
which I had become a regular member of.

### A Style Emerges

I ditched the middle info panel, and started to develop a real style. Some
friends of mine started to use my UI, and I created a bit of a brand for the
whole thing.

![](/img/epic-ui/epicui_banner_v1.jpg)

As the style grew, I updated the banner logo, and even started to version
things (sadly 2.1.1 was the last version).

![](/img/epic-ui/epicui_banner_v2.png)

In EpicUI 2.0 I decided to move the power bar (mana/etc) to the border of the
unitframes. This was in many ways what destinguished my UI from the rest. Even
though much work had been done under the hood as well. The visual changes are
the most obvious to people. World of Warcraft has so many mechanics, that
building a UI that handles them all consistently is no small task.

I would occasionally play a healer (resto shaman). I quickly discovered that my
UI wasn't optimal for this role, so I started further customizing it for each
class/spec.

![](/img/epic-ui/healing_uf.png)

I only ever made the base (caster DPS), and shamman healer layouts however.

### Bugs and Hacks

Of course, programming is not without bugs, and when you are modifying your UI
in the same timeframe as you are raiding at a relavitly high level the bugs
can have very dramatic effect. Below I managed to capture a bug that blocked a
good part of my screen while fighting a new progression boss in Firelands,
Majordomo Staghelm.

![](/img/epic-ui/bug.jpg)

Other bugs have caused me to be unable to perform actions, or see needed boss
abilities. It was a delicate act balancing developing my UI, and maintaining
a reputation of a reliable DPS player.

#### Legendary Calculations

An interesting story from my days in Firelands is the tale of Dragonwrath,
Tarecgosa's Rest, a legendary weapon that required a full raiding guild (25+
members) to create over many weeks for a *single* player to obtain. I was lucky
enough to be the second member of my guild to recive it. Once I did however it
became clear to me that the inner workings of this item were not yet known to
the public. Without going into too much detail, the staff had a chance to do
a lot of damage, but it wasn't exactly clear what caused it, and what the
probabilities actaully were. I know how to write code now I thought, this
should be easy to figure out. Sure enough after only a few hours of programming
I had a simple addon that would calculate the "proc-rate", even if the addon
looked like garbage.

![](/img/epic-ui/dragonwrath.png)

#### A Hack to Progress

TODO: maloriak.

### What a Long, Strange Trip It's Been

I still have my account, and log on from time to time, but the long nights and
endless thought has mostly died. I'm a casual player, and I don't run my own UI
anymore. ElvUI, another project from the same community is my go to, and I'm
pretty happy with it overall.

WoW to me will always hold a special place in my heart. Not only for the people
I met playing the game, but also as the gateway drug to harder programming. I
owe it all to a little LUA and a lot of help from the members of Tukui.

![](/img/epic-ui/invincible.jpg)

[tukui]: https://www.tukui.org/
[wow-api]: http://wowprogramming.com/docs
