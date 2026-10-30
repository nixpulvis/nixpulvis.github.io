---
title: Galos
layout: project
status: in progress
draft: true
references:
- https://github.com/nixpulvis/galos
---

Galos keeps its own copy of the galaxy in Elite: Dangerous, built from what
players report as they fly. It subscribes to the game's shared event feed,
stores what it hears in a database, and derives a spatial index from that. On
top sits a 3D map, which is the main way to look at it all, and beside it a
comparison against the sky as it's actually measured from Earth. It started as
a way to keep track of factions and systems, and grew from there.

TODO: One or two sentences on why you care about it. The longer personal story
lives in the [introduction rambling]({{ '/ramblings/2026-10-30-galos' | relative_url }}).

![The galaxy from above, the core at the top]({{ '/img/galos/galaxy.jpg' | relative_url }})

### The Feed, the Database and the Index

Elite's galaxy arrives as a stream of events. Every commander running a tool
like EDMC forwards what their game sees to [EDDN][eddn]. `galos` subscribes to
that feed and keeps it in Postgres (with PostGIS for the positions), and can
answer searches and routes from the command line. Dumps from
[Spansh][spansh] and [EDSM][edsm] fill in the history that came before. Your
own game writes a directory of journal files, and Galos reads those too, so the
world and your own commander end up in one place.

From that data a builder derives a single spatial index, an octree written to
disk in its own format. The map reads only that index, with no database or server
behind it, just a directory of files.

```sh
cargo run --release --bin galos -- ingest --from eddn --index ~/.galos_index
cd galos_map && cargo run --release -- --index ~/.galos_index
```

TODO: Check these commands against the current CLI before publishing.

### The Map

The map is written in Rust with [Bevy][bevy]. From far away it draws every
known system as one glowing field, and as you fly in the systems separate into
points, then named stars, then a system's own planets at their real geometry.
Planets are drawn where the game has carried them by now, run forward from
whenever someone last scanned them, rather than where they happened to be at
the time.

![The stars around Sol, named]({{ '/img/galos/local.jpg' | relative_url }})

It can also route. The router builds a jump graph over the index, knows about
neutron star "supercharge" highways, and can order a set of stops into a
sensible trip.

### The Real Sky

Elite's galaxy is a procedural model of the Milky Way, seeded with real
catalog stars near Sol. So the question is, how close is it? Galos reads
published star catalogs and compares them against the game. The photometry
crate works out magnitudes, temperatures and color, and a small CPU renderer
draws a patch of sky, which is how the Big Dipper ends up as a golden test
image.

TODO: A side by side of a real constellation and the game's version would be
great here.

### Status

TODO: What works today, what you want next, and whether anyone else should
try running it yet.

[eddn]: https://github.com/EDCD/EDDN
[spansh]: https://spansh.co.uk
[edsm]: https://www.edsm.net
[bevy]: https://bevyengine.org
