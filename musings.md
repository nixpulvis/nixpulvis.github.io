---
layout: default
title: Music
---

We can now display music notation (♬) on this site, more to come.

<ul class="musings">
{%- for musing in site.collection("musings") -%}
  <li class="musing">
    <a class="name" href="{{ musing.url }}">{{ musing.title() }}</a>
  </li>
{%- endfor -%}
</ul>
