---
title: References
layout: page
---
{% import "items.html" as items %}

<!-- Picking a tag targets its link, and these rules hide every reference
     without that tag. With no tag picked, everything shows. -->
<style>
{%- for tag in site.data.used_tags() -%}
.tagged:has(#{{ tag }}:target) li:not([data-tags~="{{ tag }}"]) { display: none; }
{%- endfor -%}
</style>

<div class="tagged">
<nav class="tags">
    <a href="#all" id="all">all</a>
{% for tag in site.data.ordered_tags() -%}
    {%- call items::reference_tag(site, tag, site.data.tag_count(tag)) -%}{%- endcall -%}
{%- endfor -%}
</nav>

<ul>
{%- for reference in site.data.references_by_title() -%}
    <li data-tags="{{ reference.tags.join(" ") }}">
        <a href="{{ reference.url }}">{{ reference.title }}</a>
        <span class="tags">{% for tag in reference.tags %}{% call items::reference_tag(site, tag, 0) %}{% endcall %}{% endfor %}</span>
    </li>
{%- endfor -%}
</ul>
</div>
