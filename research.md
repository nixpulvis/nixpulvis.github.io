---
title: Research
layout: default
---

# Research

<ul>
{% for article in site.articles %}
    {% if article.draft %}
        <li class="draft">
            <a href="{{ article.url }}">{{ article.title }}</a>
            {{ article.content | number_of_words }} words
            drafted on {{ article.date | date: "%B %d, %Y" }}.
        </li>
    {% else %}
        <li>
            <a href="{{ article.url }}">{{ article.title }}</a>
            {{ article.content | number_of_words }} words
            published on {{ article.date | date: "%B %d, %Y" }}.
        </li>
    {% endif %}
{% endfor %}
</ul>
