---
title: "\"Borrowing\" a Better WiFi Signal"
layout: post
draft: true
---

Let me start by saying that "stealing" WiFi (or anything for that matter) is
wrong, and bad. Please don't do it. But if you happen to find yourself
"borrowing" someone elses WiFi with permission, i.e. they gave you the password,
then the next step is clearly improving the link quality.

### Attenuate like Hell

I'm currently living in the mountains of Colorado, in a place that doesn't have
it's own ISP hookup. One of our neighbors is nice enough to let us borrow thier
WiFi though, so all is not lost. If I had to guess I'd say that I'm about 100ft
from the access point (AP), and about 6 or so walls stand in the way. It's not
an ideal RF setup by any means, but it works.

The AP is broadcasting at 2412MHz (channel 1 of the 2.4GHz band), the bandwidth
is 20MHz.
TODO: Other link stats.

However, it's nowhere near good enough. Enter the world of fancy antennas. In
the RF antenna world there exists a device named "Yagi", after one of it's
creators. A Yagi is a directional antenna which operates with a single driven
element, a reflecting element, and many directing elements. These are the same
design as many household TV antennas from the old days.

![]()

I'm not going to get into the design of these things, because frankly that's
above my pay-grade at the moment, but there are a few key things to know about
these antennas.

- They are directional,
- The antenna should be arranged to match the polarization of the AP.
- The frequency

So I bought [one](amazon), and a [USB WiFi adapter](amazon) to go along with it. I
chose the adapter to meet the following requirments:

- 802.11n, 802.11ac isn't needed because we're only dealing with the 2.4GHz
  band here.
- Drivers included with Linux by default
- External antenna connection (typically SMA)

I After ordering the wrong kind of SMA adapter I finnaly got the [right](amazon)
one, and we're in business. Simply moving from my laptop's built-in WiFi to this
new setup improved signal level by TODO, and link quality by TODO.

### Share the World

It's one thing to have access to a nice, fast network on a single computer, but
I want my roomates to be able to benifit from this high quality link as well.
Enter the NAT.

Building a NAT device isn't too bad. I've got an extra Rasperry Pi (v1) lying
around running ArchARM.

TODO: more words, and links to GitHub sources.

First let's connect to the WiFi network, and make sure it's working.

```sh
netctl start <profile>
…
ping 8.8.8.8
```

We need to tell the kernal to route packets accross network interfaces, for
more information start with [this SO post](https://unix.stackexchange.com/questions/14056/what-is-kernel-ip-forwarding).

```sh
sysctl net.ipv4.ip_forward=1
```

Once we've enabled packet forwarding, we need to setup the [NAT](wiki) between
the wireless interface, and the Pi's ethernet interface.

```sh
iptables -t nat -A POSTROUTING -o $from -j MASQUERADE
iptables -A FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
iptables -A FORWARD -i $to -o $from -j ACCEPT
```

Finally, we enable [DHCP](wiki) to allow clients to negociate for an IP address
on the new local network.

```sh
systemctl start dhcpd4@$to.service
```

With that all said and done we should be able to connect a computer via ethernet
and access the internet. Woot! All that's left is to connect that ethernet cable
to a functional wireless router, and we've got our own WiFi network with good
connectivity.

### Going Futher (literally…)

One thing that I'm now dying to test is how far a 2.4GHz signal can go, with
modern, FCC complient hardware like I have. If I were to buy another one of
these antennas and point them at each other, how many meters, or kilometers
could it go.

Another factor in all this is the TX (sending) power. I don't know,
at the moment, how much power this WiFi adapter can send with, nor do I know
what the legal limits are. But the more power the further the signal will go, so
it's worth looking into.

With an optimal setup, I've heard of people beaming WiFi over a kilometer, so
this could be fun indeed! Tune in next time...
