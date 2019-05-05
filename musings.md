---
layout: default
title: Music
script: /js/opensheetmusicdisplay.min.js
---

We can now display music notation (♬) on this site, more to come.

<ul class="musings">
{% for musing in site.musings %}
  <li class="musing">
    <a class="name" href="{{ musing.url }}">{{ musing.title }}</a>
  </li>
{% endfor %}
</ul>
