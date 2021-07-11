---
layout: default
title: Mathematics
---

# Mathematics

> The study of the measurement, properties, and relationships of quantities and
> sets, using numbers and symbols.

<ul>
{%
assign mathematics = site.mathematics
                 | sort: 'date'
                 | reverse
%}

{% for math in mathematics %}
    {% if math.draft %}
        <li class="draft">
            <a href="{{ math.url }}">{{ math.title }}</a>
            <small>
                drafted on {{ math.date | date: "%B %d, %Y" }}.
            </small>
        </li>
    {% else %}
        <li>
            <a href="{{ math.url }}">{{ math.title }}</a>
            <small>
                published on {{ math.date | date: "%B %d, %Y" }}.
            </small>
        </li>
    {% endif %}
{% endfor %}
</ul>
