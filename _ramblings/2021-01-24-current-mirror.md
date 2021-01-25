---
title: The Current Mirror
layout: rambling
---

When designing many electrical circuits, it is often useful to have a fixed
source (or sink) of current. This requirement lead me to the discovery of the
current mirror, which can be used to implement a fixed current, amongst other
things. As the name suggests, a current mirror _mirrors_ the current on one
path to another. One can then use a specific application of the current mirror
to fix the current on the first path, thus holding the current on the other
path steady over changing conditions.

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

<!-- TODO: Link to a post on the temperature controller project. -->

### Preliminaries

Before we can understand the workings of the current mirror, we must first
understand some basics of diodes and transistors. The arrows/triangles in the
symbols below should help remind you of the relationship between these
components.

<figure class="image">
  <img src="/img/ramblings/current-mirror/diode_transistor.png" />
  <figcaption>A "diode-connected" transistor.</figcaption>
</figure>

#### Diodes

To some of you, the most familiar kind of diode will be the Light-emitting
Diode, however we're not interested in generating light for this application of
a diode. Instead, we are interesting in a property of all diodes.

Anode = P
Cathode = N

<figure class="image">
  <img src="/img/ramblings/current-mirror/led.png" />
  <figcaption>The schematic symbol for an LED.</figcaption>
</figure>

TODO: Explain NP (or PN) junction of a diode leading into transistors.

#### Transistors

For the purposes of explaining our current mirror I'll only dive into Bipolar
Junction Transistors (BJTs) in any length, however many of these concepts are
similar to other kinds of transistors. Specifically, I've seen schematics which
make use of a metal-oxide-semiconductor field-effect transistor (MOS-FET) or
even an Operational Amplifier (Op-Amp) to implement a current mirror.

There are two kinds of BJTs: n-type NPN transistors and p-type PNP transistors.
The "N" and "P" components of the name refer to the polarity of the material
used in the construction of the transistor. A NPN transistor consists of a
positive region sandwiched between two negative regions, and visa-versa for the
PNP transistor.

<figure class="image">
  <img src="/img/ramblings/current-mirror/pnp_npn.png" />
  <figcaption>The schematic symbols for the two kinds of BJT.</figcaption>
</figure>

### Putting it Together

A current mirror is a circuit configuration of two transistors which, by virtue
of their shared base, cause the current flowing through the emitters to be the
same regardless of the load resistance. This is set by the resistance (`Rs`
below) which is constrained by the saturation requirement for the transistors.
For this circuit to work well the transistors (`Q1` & `Q2`) must be as similar
as possible.

TODO: Saturation or active mode?

#### Experimental Configuration 1

First we tested a very basic current mirror configuration with a couple random
BJT transistors which were lying around.

![](/img/ramblings/current-mirror/source.png)


In this initial small set of experiments we fix a reference resistor `Rs`
$$\approx 1$$ k$$\Omega$$, then change the value of `Rn`. Additionally, for the
`R1'` experiment, we move the two transistors physically closer to each other.
The measurement of each experiment is the voltage across the resistors. We only
recorded the voltage across `Rs` once, since it is constant.

<table>
  {% for row in site.data.ramblings.current-mirror.pnp %}
    {% if forloop.first %}
    <tr>
      {% for pair in row %}
        <th>{{ pair[0] }}</th>
      {% endfor %}
      <th>Ω/V = Current (mA)</th>
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

#### Experimental Configuration 2

Unlike the first experiment where we simply rely on the properties of two
2N3906 transistors to be the same, here we use a dedicated current mirror
device, which are manufactured to minimize the difference between the
transistors.

![](/img/ramblings/current-mirror/sink.png)

<table>
  {% for row in site.data.ramblings.current-mirror.npn %}
    {% if forloop.first %}
    <tr>
      {% for pair in row %}
        <th>{{ pair[0] }}</th>
      {% endfor %}
      <th>Current (mA)</th>
    </tr>
    {% endif %}

    <tr>
      {% for pair in row %}
        <td>{{ pair[1] }}</td>
      {% endfor %}
      <td>{{row["Voltage (V)"] | divided_by: row["Resistance (Ω)"] | times: 1000 | round: 2}}</td>
    </tr>
  {% endfor %}
</table>
