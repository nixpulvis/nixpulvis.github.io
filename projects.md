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
> -- <cite>Ralph Waldo Emerson</cite>

Please be aware, unlike my [Ramblings](/ramblings), these projects are living
documents. I may update them without notice to add detail or track changes to
the projects themselves.

<ul class="projects finished">
{% for project in site.projects %}
    {% unless project.draft %}
        <li class="project">
            <a class="name" href="{{ project.url }}">{{ project.title }}</a>
            {{ project.excerpt }}
            {% if project.references[0] %}
                <ul class="references">
                {% for reference in project.references %}
                    <li><a href="{{reference}}">{{ reference }}</a></li>
                {% endfor %}
                </ul>
            {% endif %}
        </li>
    {% endunless %}
{% endfor %}
</ul>

<ul class="projects drafted">
{% for project in site.projects %}
    {% if project.draft %}
        <li class="project draft">
            <a class="name" href="{{ project.url }}">{{ project.title }}</a>
            {{ project.excerpt }}
            {% if project.references[0] %}
                <ul class="references">
                {% for reference in project.references %}
                    <li><a href="{{reference}}">{{ reference }}</a></li>
                {% endfor %}
                </ul>
            {% endif %}
        </li>
    {% endif %}
{% endfor %}
</ul>
