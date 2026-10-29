---
title: Archive
layout: default
permalink: /ramblings/archive
---

# Archive

Old ramblings I've moved off the [main list]({{ '/ramblings' | relative_url }}).

<ul class="posts">
{%
assign ramblings = site.ramblings
                 | concat: site.mathematics
                 | sort: 'date'
                 | reverse
%}

{% for rambling in ramblings %}
    {% unless rambling.archived %}
        {% continue %}
    {% endunless %}
    {% include post-item.html post=rambling %}
{% endfor %}
</ul>
