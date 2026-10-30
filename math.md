---
layout: page
title: Mathematics
---

> The study of the measurement, properties, and relationships of quantities and
> sets, using numbers and symbols.

<ul>
{%- for math in site.newest_first("mathematics") -%}
    {%- if math.front.draft -%}
        <li class="draft">
            <a href="{{ math.url }}">{{ math.title() }}</a>
            <small>
                drafted on {{ math.long_date() }}
            </small>
        </li>
    {%- else -%}
        <li>
            <a href="{{ math.url }}">{{ math.title() }}</a>
            <small>
                published on {{ math.long_date() }}
            </small>
        </li>
    {%- endif -%}
{%- endfor -%}
</ul>
