---
title: Research
layout: default
---

# Research

<ul>
{% for research in site.research %}
    {% if research.hidden %}
        {% continue %}
    {% endif %}
    {% if research.draft %}
        <li class="draft">
            <a href="{{ research.url }}">{{ research.title }}</a>
            {{ research.content | number_of_words }} words
            drafted on {{ research.date | date: "%B %d, %Y" }}.
        </li>
    {% else %}
        <li>
            <a href="{{ research.url }}">{{ research.title }}</a>
            {{ research.content | number_of_words }} words
            published on {{ research.date | date: "%B %d, %Y" }}.
        </li>
    {% endif %}
{% endfor %}
</ul>
