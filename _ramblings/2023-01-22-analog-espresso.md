---
layout: rambling
draft: true
---

This is how I currently make coffee.

Someday I'll pull the perfect shot and be lucky enough to have [recorded the
moment](). Another day, I'll have some [new sensors and tools]() to assist the
process. Until then, this should suffice.

It's worth mentioning that these steps represent the _ideal_ process for my
equipment, however as variables and conditions change, so too must the barista.
Some variables, like temperature, I feel I have something of a handle on, while
others like bean quality and humidity, I'm still less sure of.

I try to be consistent about things, but it's important to remember the
following maxim:

> Variety is the spice of life.

Now admittedly, I have already started the process of digitizing some things.
Mainly, at the moment, I have a digital thermometer connected to the grouphead,
which I can use to gauge things a bit. For the purposes of this writeup,
digital readings with take the following form:

```
YYYY-MM-DD HH-MM-SS <TEMP> <NOTE>
                           ...
```

With that, let's begin.

TODO: Document the make/model/year/condition of the equipment.


### Preparation

1. Turn on the machine

```
2023-01-22 2:24:30 87.8F ELEMENT ON
2023-01-22 2:27:04 94.5F ELEMENT OFF
                         FLUSH
```
After this first flush the grouphead temperature should be raising quickly,
which is nice, because we'll use that temperature as well as how hot the
portafilter itself feels to judge how close to ready we are for grinding and
brewing. By my estimations, the temperature is now raising between 1F/5s
and 1F/10s.

Take a breath. Go have a nice long shower. Make breakfast, or lunch. Play a
round of your favorite game. Whatever. At this point the machine is trailing
off calmly to a steady hot state.

```
2023-01-22 2:30:20 112F ELEMENT ON
2023-01-22 2:32:00 130F ELEMENT OFF
2023-01-22 2:37:00 144F
```

The machine's boiler and group head as well as the portafilter are now at
temperature. We're almost ready to go.


### Grinding and the First Wave

Now it's gametime. First we take the portafilter out of the machine's
grouphead and flush the plumbing to start renormalizing the pipes temperatures.
Without the portafilter you should be able to get a really good sense of the
flow and temperature of the water coming out of the grouphead. If the machine
has been maintained and cleaned well, it should be flushed briefly until a
gentle shower is formed. Depending on timing and external temperatures, this
may soon start another heating cycle, no worries.

```
2023-01-22 2:38:30 147F FLUSH
          +    ~5s      ELEMENT ON
```

Dry off the basket completely and then put that dense hunk of metal onto the
grinder's mount and fill'r up. I typically use a flat edge after grinding to
level off the mound, but as long as things are balanced and not over or
underfilled, it doesn't really matter. You can avoid wasting beans by simply
grinding the correct amount into the basket cleanly, but I'm generally not too
concerned about the wasted dust.

Tamping is the fun part. Do it right, and don't be shy. If you have your
grinder dialed in correctly for the beans, you should be applying more force
than you might originally think. Too much force can still lead to a slow and
bitter brew however, so do be careful.

While you've been grinding and tamping, hopefully the machine has found itself
settled at a nice and comfertable temperature. If you hear or sense the boiler
being upset with you in any way, don't hessitate to let it run through another
cycle while flushing the grouphead a bit. Remember, coffee grounds go stale
quickly, but not **that** quickly. It's not until we've actually started
brewing the espresso that we need to hurry.



### Brewing

Once it's all raring to go, vice in the portafilter to the grouphead tightly.
Feel free to flex, not securing the portafilter properly can result in an
explosive mess!

If the boiler element has turned on, wait a moment until it's turned off, then
switch on the pump. Or don't, this is the fun part I haven't had the patenice
to perform proper science on yet. People refer to the game of figuring out
where in this cylce to start brewing and how to achive the perfect curves as
**temperature surfing**. So, maybe feel the waves, hang ten, then start a bit
early or late. You're the boss.

```
2023-01-22 2:40:00 150F BREW ON 
          +     30 154F BREW OFF 
```

With a double spout like mine, you should start to see
dark and creamy liquid streaming out of both sides within no more than 2 or 3
seconds. Watch closely. The streams should become less and less dark quickly,
with viens of oily flavor running through them at first, then diluting to a
softer blond color. If everything is coming out nicely, it's around the 25-30
second mark that your shot is filled and the color is becoming awkwardly pale.

TODO: Dump/wash/clean


### With Company

Repeat the grinding and brewing process as needed, remember things do tend to
get a bit hotter while in use.


### "How do you take it?"

Now that the coffee is ready, it must be served as fast as possible.

The afternoon is for straight espresso, "No milk, thanks".

```
2023-01-22 2:40:32 155F SERVE
                        OFF
2023-01-22 2:46:00 157F CLEAR
```


##### Froth(ing/ed) Milk

Before you started brewing hopefully you had already had a sense of how many
drinks you were going to be making and how much milk to froth (if any).

With only a single boiler, this is a bit slower if you want to use steam to
froth milk (or god forbid a milk alternative). Since the machine's grouphead
was hanging out around 150 degrees Fahrenheit before we started brewing, the
boiler is already catching up while we turn on the steam thermostat. Now we
wait until ideally just before the light indicating the boiler is at
temperature to turn off. For what it's worth, this generally happens after only
a couple minutes. By the time we're finished and the temperature raise has
reached the grouphead, it will be around 170 degrees Fahrenheit or so.

Starting the frothing process at the **peak temperature is critical** on my
machine, since the thermostat allows for a pretty large temperature swing after
it clicks off. Depending on how frothy you want things perhaps letting as few
as 15 seconds go by could improve the quality of the milk's texture, but I'm
not able to reliably reproduce this.

Only capachinnos get more than a veil of proper froth. The goal is generally to
have the milk _stretched_, turning whole milk into the texture of a creamy
cloud.


"Would you care for some cream?" I'm not sure frothing cream is actually a good
idea.


##### The Pour

Start high, then move in low. 

Mochas are for special days, drizzle the syrup into the hot milk, then pour it
into the espresso.


##### Toppings?
