---
layout: default
title: Music
style: /css/musing.css
scripts:
- /js/opensheetmusicdisplay.min.js
- /js/musing.js
---

We can now display music notation (♬) on this site, more to come.

<ul class="musings">
{% for musing in site.musings %}
  <li class="musing">
    <a class="name" href="{{ musing.url }}">{{ musing.title }}</a>
  </li>
{% endfor %}
</ul>

<ul>
{% for asset in site.static_files %}
  {% if asset.path contains '.xml' or asset.path contains '.musicxml' %}
    {% assign name = asset.path | split: '/' | last | split: '.' | first %}
    <li>
      <div class="score" data-music-xml="{{ asset.path }}"/>
    </li>
  {% endif %}
{% endfor %}
</ul>
