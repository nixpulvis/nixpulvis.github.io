---
layout: default
---

<h2 class="title">
    {{ page.title }}
    {% if page.status %}<span class="status {{ page.status | slugify }}">{{ page.status }}</span>{% endif %}
</h2>

<ul class="references">
{% for reference in page.references %}
    <li><a href="{{reference}}">{{ reference }}</a></li>
{% endfor %}
</ul>

{{ content }}
