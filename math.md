---
layout: default
title: Mathematics
---

# Mathematics

> The study of the measurement, properties, and relationships of quantities and
> sets, using numbers and symbols.

<ul>
{% for math in site.mathematics %}
    {% if math.draft %}
        <li class="draft">
            <a href="{{ math.url }}">{{ math.title }}</a>
            drafted on {{ math.date | date: "%B %d, %Y" }}.
        </li>
    {% else %}
        <li>
            <a href="{{ math.url }}">{{ math.title }}</a>
            published on {{ math.date | date: "%B %d, %Y" }}.
        </li>
    {% endif %}
{% endfor %}
</ul>
