---
title: Projects
layout: page
---
{% import "items.html" as items %}

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
{%- for project in site.collection("projects") -%}
    {%- if !project.front.hidden && !project.front.draft -%}
        {%- call items::project_item(project) -%}{%- endcall -%}
    {%- endif -%}
{%- endfor -%}
</ul>

<ul class="projects drafted">
{%- for project in site.collection("projects") -%}
    {%- if !project.front.hidden && project.front.draft -%}
        {%- call items::project_item(project) -%}{%- endcall -%}
    {%- endif -%}
{%- endfor -%}
</ul>
