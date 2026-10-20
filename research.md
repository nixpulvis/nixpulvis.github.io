---
title: Research
layout: default
---

# Research

<ul class="posts">
{% for research in site.research %}
    {% if research.hidden %}
        {% continue %}
    {% endif %}
    {% unless research.path contains '/articles/' %}
        {% continue %}
    {% endunless %}
    {% include post-item.html post=research %}
{% endfor %}
</ul>

### Notes

<ul class="posts">
{% for research in site.research %}
    {% if research.hidden %}
        {% continue %}
    {% endif %}
    {% unless research.path contains '/notes/' %}
        {% continue %}
    {% endunless %}
    {% include post-item.html post=research %}
{% endfor %}
</ul>
