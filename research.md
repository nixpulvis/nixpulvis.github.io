---
title: Research
layout: page
---
{% import "items.html" as items %}

<ul class="posts">
{%- for research in site.collection("research") -%}
    {%- if !research.front.extra.hidden && research.source.path.contains("/articles/") -%}
        {%- call items::post_item(research) -%}{%- endcall -%}
    {%- endif -%}
{%- endfor -%}
</ul>

### Notes

<ul class="posts">
{%- for research in site.collection("research") -%}
    {%- if !research.front.extra.hidden && research.source.path.contains("/notes/") -%}
        {%- call items::post_item(research) -%}{%- endcall -%}
    {%- endif -%}
{%- endfor -%}
</ul>
