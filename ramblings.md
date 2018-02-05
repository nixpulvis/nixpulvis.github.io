---
title: Ramblings
layout: default
---

# Ramblings

<ul>
{% for rambling in site.categories.ramblings %}
    <li><a href="{{ rambling.url }}">{{ rambling.title }}</a></li>
{% endfor %}
</ul>
