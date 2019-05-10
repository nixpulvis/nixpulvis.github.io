---
layout: default
title: Mathematics
---

# Mathematics

> The study of the measurement, properties, and relationships of quantities and
> sets, using numbers and symbols.

<ul>
{% assign mathematics = site.mathematics | sort: 'date' | reverse %}
{% for math in mathematics %}
  {% if math.draft %}
    <li class="draft">
  {% else %}
    <li>
  {% endif %}
      <a href="{{ math.url }}">{{ math.title }}</a>
    </li>
{% endfor %}
</ul>
