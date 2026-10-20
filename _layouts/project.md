---
layout: default
---

<h2 class="title">{{ page.title }}</h2>

<ul class="references">
{% for reference in page.references %}
    <li><a href="{{reference}}">{{ reference }}</a></li>
{% endfor %}
</ul>

{{ content }}
