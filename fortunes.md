---
title: Fortunes
layout: default
permalink: /fortunes/
---

# Fortunes

Little thoughts, caught before they got away. Some are wisdom, some are
wordplay, and some I don't remember thinking. Each year is kept as a
[database for `fortune`]({% link _ramblings/2018-06-26-fortune.md %}), so they
can find you at random, the way they found me.

{% for year in site.data.fortunes %}
{% if year.entries.size == 0 %}{% continue %}{% endif %}
<h2 id="{{ year.year }}">
    {{ year.year }}
    <small>
        <a href="{{ year.path | relative_url }}">text</a>{% if year.dat %},
        <a href="{{ year.dat | relative_url }}">dat</a>{% endif %}
    </small>
</h2>
<ul class="fortunes">
{% for entry in year.entries %}
    <li>{{ entry | escape }}</li>
{% endfor %}
</ul>
{% endfor %}
