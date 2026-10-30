---
title: Fortunes
layout: page
permalink: /fortunes/
---

Caught thoughts. Some are wisdom, some are wordplay, and most I don't even
remember thinking. Each year is kept as a [database for
`fortune`](/ramblings/2018-06-26-fortune), so they find me at random, the way
they found me originally.

{% for year in site.fortunes -%}
<h2 id="{{ year.year }}">{{ year.year }}</h2>
<p class="meta">
    <a href="{{ year.path() }}">text</a>,
    <a href="{{ year.dat() }}">dat</a>
</p>
<ul class="fortunes">
{%- for entry in year.entries -%}
    <li>{{ year.html(entry) }}</li>
{%- endfor -%}
</ul>
{%- endfor -%}
