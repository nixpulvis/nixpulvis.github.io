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
    {% unless research.path contains '/articles/' %}
        {% continue %}
    {% endunless %}
    {% if research.draft %}
        <li class="draft">
            <a href="{{ research.url }}">{{ research.title }}</a>
            <small>
                {{ research.content | number_of_words }} words
                drafted on {{ research.date | date: "%B %d, %Y" }}.
            </small>
        </li>
    {% else %}
        <li>
            <a href="{{ research.url }}">{{ research.title }}</a>
            <small>
                {{ research.content | number_of_words }} words
                published on {{ research.date | date: "%B %d, %Y" }}.
            </small>
        </li>
    {% endif %}
{% endfor %}
</ul>

### Notes

<ul>
{% for research in site.research %}
    {% if research.hidden %}
        {% continue %}
    {% endif %}
    {% unless research.path contains '/notes/' %}
        {% continue %}
    {% endunless %}
    {% if research.draft %}
        <li class="draft">
            <a href="{{ research.url }}">{{ research.title }}</a>
            <small>
                {{ research.content | number_of_words }} words
                drafted on {{ research.date | date: "%B %d, %Y" }}.
            </small>
        </li>
    {% else %}
        <li>
            <a href="{{ research.url }}">{{ research.title }}</a>
            <small>
                {{ research.content | number_of_words }} words
                published on {{ research.date | date: "%B %d, %Y" }}.
            </small>
        </li>
    {% endif %}
{% endfor %}
</ul>
