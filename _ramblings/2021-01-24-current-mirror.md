---
title: The Current Mirror
draft: true
layout: rambling
---

When designing many electrical circuits, it is often useful to have a fixed
source (or sink) of current. This requirement lead me to the discovery of the
current mirror, which can be used to implement a fixed current, amongst other
things. As the name suggests, a current mirror _mirrors_ the current on one
path to another. By fixing the current on the left path we hold the current on
the right path steady over changing conditions.

<!-- TODO: Explain why sourcing current, and sourcing a "known" amount of current is -->
<!-- different. -->
<!-- TODO: Why must we fix the left path? -->

<figure class="image">
  <img src="/img/ramblings/current-mirror/mirror.png" />
  <figcaption>The schematic symbol(s) of a BJT current sink.</figcaption>
</figure>

This can be useful, for example, in driving a dynamic number of LEDs in series.
Simply set the fixed current to an LEDs individual current requirement. Since
the current through a circuit in series is constant, each LED will get it's
needed juice. The trick is that when an LED is added, the current mirror adapts
by increasing the voltage to account for the extra drop caused by the new LED.
The project I'm currently working on requires precise measurement of a
Resistance Temperature Detector (RTD), which depends on a fixed current in
order to measure the voltage with an Analog to Digital Converter (ADC). But,
more on that project another time...

<!-- TODO: This paragraph needs help. -->
<!-- TODO: Link to a post on the temperature controller project. -->

### Preliminaries

Before we can understand the workings of the current mirror, we must first
understand some basics of diodes and transistors. The arrows/triangles in the
symbols below should help remind you of the relationship between these
components. It may help to first realize that the left transistor from the
current mirror schematic above is actually acting like a diode.

<figure class="image">
  <img src="/img/ramblings/current-mirror/diode_transistor.png" />
  <figcaption>A "diode-connected" transistor.</figcaption>
</figure>

#### Diodes

Diodes are like one-way valves. Current flow in the _forward_ direction is
relatively uninhibited, while passing current in the _reverse_ direction is
impeded until it reaches what's known as it's _breakdown voltage_.

Diodes, much like the BJTs we'll look at next, are made of two types of
material regions. Current flows from a _p-type_ material, through a depleted
region, into an _n-type_ material. This is called a "PN" junction. Another name
for the p-type and n-type terminals are _anode_ and _cathode_, respectively.

For some readers, the most familiar kind of diode will be the Light-emitting
Diode, however we're not interested in generating light for this application of
a diode.

<figure class="image">
  <img src="/img/ramblings/current-mirror/led.png" />
  <figcaption>The schematic symbol for an LED.</figcaption>
</figure>

What we are interested in is the fact that a diode, when fully forward-biased
has a static voltage drop (0.7 V is a common value). This is because once the
voltage is high enough, the diode's depletion region has been effectively
shrunk to nonexistence.

#### Transistors

For the purposes of explaining our current mirror I'll only dive into Bipolar
Junction Transistors (BJTs) in any length, however many of these concepts are
similar to other kinds of transistors. Specifically, I've seen schematics which
make use of a metal-oxide-semiconductor field-effect transistor (MOS-FET) or
even an Operational Amplifier (Op-Amp) to implement a current mirror.

There are two kinds of BJTs: n-type NPN transistors and p-type PNP transistors.
Like the diode, the "N" and "P" components of the name refer to the polarity of
the material used in the construction of the transistor. A NPN transistor
consists of a positive region sandwiched between two negative regions, and
visa-versa for the PNP transistor.

<figure class="image">
  <img src="/img/ramblings/current-mirror/pnp_npn.png" />
  <figcaption>The schematic symbols for the two kinds of BJT.</figcaption>
</figure>

There are three currents for each transistor we label:
- $$I_E$$, the current through the emitter
- $$I_B$$, the current through the base
- $$I_C$$, the current through the collector 

The ratios $$\alpha = I_C / I_E$$ and $$\beta = I_C / I_B$$ are properties of
the specific transistors and can be found in the datasheet of the appropriate
part. The datasheet may provide the parameter "DC Current Gain" with the symbol
$$h_FE$$ (denoting a hybrid parameter), which is what we use for $$\beta$$.
The reason for the discrepancy in notation is that, while we treat $$\beta$$ as
a constant for our purposes, in reality the value is effected by many things
like temperature, signal frequency, and current itself.

One important rule for transistors is that $$I_E = I_C + I_B$$. Therefor,
$$\alpha= \beta / (\beta + 1)$$. This should also start to build your intuition
for how the current mirror works, since both transistors bases are connected.
More on that in a moment though...

To understand the role of the base in a transistor, let's consider an example:

$$
\beta = 100 \\
I_B = 5mA \\
I_C = \beta I_B = 0.5A
$$

So, the base current ($$I_B$$) has effectivly set the collector current
($$I_C$$) with a gain of $$100 \times$$.

##### Saturation

Sometimes, the goal isn't to provide presice, fixed amounts of current, but to
simply use a transistor as a switch. To accomplish this, the transistor is put
into what's called _saturation_, by making $$V_CE < V_BE$$. Now our equation
$$I_C = \beta I_B$$ from above no longer holds and is replaced by $$I_C < \beta
I_B$$.

In the case of the diode-connected transistor, since the collector and base are
connected, we have 

### Putting it Together

A current mirror is a circuit configuration of two transistors which, by virtue
of their shared base, cause the current flowing through the emitters to be the
same regardless of the load resistance. This is set by the resistance (`Rs`
below) which is constrained by the saturation requirement for the transistors.
For this circuit to work well the transistors (`Q1` & `Q2`) must be as similar
as possible. This is why we do not simply use a diode instead of a
diode-connected transistor, since the base characteristics must match.

TODO: Saturation or active mode?

#### Experimental Configuration 1

First we tested a very basic current mirror configuration with a couple random
BJT transistors which were lying around. These transistors are not designed for
this purpose and surely have some variance in their $$\beta$$ values. However,
we should still expect the current across `Rn` to stay relatively near that of
`Rs`.

![](/img/ramblings/current-mirror/source.png)


In this initial small set of experiments we fix a reference resistor `Rs`
$$\approx 1$$ k$$\Omega$$, then change the value of `Rn`. Additionally, for the
`R1'` experiment, we move the two transistors physically closer to each other.
The measurement of each experiment is the voltage across the resistors. We only
recorded the voltage across `Rs` once, since it is constant.

<table>
  {% for row in site.data.ramblings.current-mirror.source %}
    {% if forloop.first %}
    <tr>
      <th>Label</th>
      <th><code>Rn</code> Value (Ω)</th>
      <th>Measurement (V)</th>
      <th>Current (mA)</th>
    </tr>
    {% endif %}

    <tr>
      <td><code>{{row["Label"]}}</code></td>
      <td>{{row["Resistance (Ω)"]}}</td>
      <td>{{row["Voltage (V)"]}}</td>
      <td>{{row["Voltage (V)"] | divided_by: row["Resistance (Ω)"] | times: 1000 | round: 2}}</td>
    </tr>
  {% endfor %}
</table>

Indeed, the current through `Rn` increased by a factor of $$2\times$$ while the
resistance decreased by a factor of $$10\times$$. Without the current mirror,
it would follow Ohm's Law ($$I = V / R$$) and the current would increase
by $$10\times$$ as well. So, while our homemade current mirror is not perfect,
it is semi-functional and might earn a place in a funhouse's room of mirrors.

#### Experimental Configuration 2

Unlike the first experiment where we simply rely on the properties of two
2N3906 transistors to be the same, here we use a dedicated current mirror
device, which are manufactured to minimize the difference between the
transistors.

![](/img/ramblings/current-mirror/sink.png)

<table>
  {% for row in site.data.ramblings.current-mirror.sink %}
    {% if forloop.first %}
    <tr>
      <th>Label</th>
      <th><code>Rn</code> Value (Ω)</th>
      <th>Measurement (V)</th>
      <th>Current (mA)</th>
    </tr>
    {% endif %}

    <tr>
      {% for pair in row %}
        <td>{{ pair[1] }}</td>
      {% endfor %}
      {% if forloop.last == false %}
      <td>{{row["Voltage (V)"] | divided_by: row["Resistance (Ω)"] | times: 1000 | round: 3}}</td>
      {% else %}
      <td>N/A</td>
      {% endif %}
    </tr>
  {% endfor %}
</table>
