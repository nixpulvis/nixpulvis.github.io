---
layout: default
---

<h2>{{ page.title }}</h2>

<ul class="repos">
{% for repo in page.repos %}
    <li><a href="{{repo}}">{{ repo }}</a></li>
{% endfor %}
</ul>

{{ content }}
