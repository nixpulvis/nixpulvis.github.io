---
title: Ramblings
layout: default
---

# Ramblings

Here in lies a collection of words both technical and opinionated. I make no
promise of complete clearity, nor do I wish to ramble to the point of insanity.
If you find something you like here, I've done my job. Otherwise, be gone with
you.

<ul>
{% assign ramblings = site.ramblings | sort: 'date' | reverse %}
{% for rambling in ramblings %}
    {% if rambling.draft %}
        <li class="draft">
            <a href="{{ rambling.url }}">{{ rambling.title }}</a>
            {{ rambling.content | number_of_words }} words
            drafted on {{ rambling.date | date: "%B %d, %Y" }}.
        </li>
    {% else %}
        <li>
            <a href="{{ rambling.url }}">{{ rambling.title }}</a>
            {{ rambling.content | number_of_words }} words
            published on {{ rambling.date | date: "%B %d, %Y" }}.
        </li>
    {% endif %}
{% endfor %}
</ul>
