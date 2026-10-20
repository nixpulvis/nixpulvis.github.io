---
title: Fortunes
layout: default
permalink: /fortunes/
---

# Fortunes

A few thoughts a year, kept as [databases for `fortune`]({% link _ramblings/2018-06-26-fortune.md %}).
To get them in your own terminal, download a year's file and its `.dat` into
the same directory:

```sh
curl -O https://nixpulvis.com/fortunes/2021 \
     -O https://nixpulvis.com/fortunes/2021.dat
fortune 2021
```

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
