---
title: Projects
layout: default
---

# Projects

I work on projects from time to time, and try to post about them here. If you
learn something, that's great. If you are inspired, that's even better. Half
the motivation to write up information about my projects is to force myself to
think/build clearly.

> The reward of a thing well done is having done it.
>
> -- Ralph Waldo Emerson

Please be aware, unlike my [Ramblings](/ramblings), these projects are living
documents. I may update them without notice to add detail or track changes to
the projects themselves.

{% assign nondrafts = site.projects | where:"draft","false" %}
{% if nondrafts == empty %}
<img src="/img/dust.jpg" />
{% endif %}

<ul>
{% for project in site.projects %}
    {% unless project.draft %}
        <li>
            <a href="{{ project.url }}">{{ project.title }}</a>
            {{ project.excerpt }}
        </li>
    {% endunless %}
{% endfor %}
</ul>
