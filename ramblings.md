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
{%
assign ramblings = site.ramblings
                 | concat: site.mathematics
                 | sort: 'date'
                 | reverse
%}

{% for rambling in ramblings %}
    {% if rambling.hidden %}
        {% continue %}
    {% endif %}
    {% include post-item.html post=rambling %}
{% endfor %}
</ul>
