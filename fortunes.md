---
title: Fortunes
layout: page
permalink: /fortunes/
---

Caught thoughts. Some are wisdom, some are wordplay, and most I don't even
remember thinking. Each year is kept as a [database for `fortune`]({% link
_ramblings/2018-06-26-fortune.md %}), so they find me at random, the way they
found me originally.

{% for database in site.data.fortunes -%}
{%- if database.entries.size == 0 %}{% continue %}{% endif -%}
<h2 id="{{ database.year }}">{{ database.year }}</h2>
<p class="meta">
    <a href="{{ database.path | relative_url }}">text</a>{% if database.dat %},
    <a href="{{ database.dat | relative_url }}">dat</a>{% endif %}
</p>
<ul class="fortunes">
{%- for entry in database.entries -%}
    <li>{{ entry | escape | newline_to_br | strip_newlines }}</li>
{%- endfor -%}
</ul>
{%- endfor -%}
