---
title: Projects
layout: default
---

# Projects

<ul>
{% for project in site.categories.projects %}
    <li><a href="{{ project.url }}">{{ project.title }}</a></li>
{% endfor %}
</ul>
