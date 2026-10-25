---
title: References
layout: default
---

# References

{% assign references = site.data.references | sort_natural: "title" %}
{% assign tags = "" | split: "" %}
{% for reference in references %}
{% assign tags = tags | concat: reference.tags %}
{% endfor %}
{% assign tags = tags | uniq | sort %}

{% comment %} Tags in scale order, then the groups, then the rest. {% endcomment %}
{% assign ordered = "" | split: "" %}
{% assign steps = site.data.reference_tags.scale %}
{% for group in site.data.reference_tags.groups %}
{% assign steps = steps | push: group[1] %}
{% endfor %}
{% for step in steps %}
{% for tag in step %}
{% if tags contains tag %}{% assign ordered = ordered | push: tag %}{% endif %}
{% endfor %}
{% endfor %}
{% for tag in tags %}
{% unless ordered contains tag %}{% assign ordered = ordered | push: tag %}{% endunless %}
{% endfor %}

<!-- Picking a tag targets its link, and these rules hide every reference
     without that tag. With no tag picked, everything shows. -->
<style>
{% for tag in tags %}
.tagged:has(#{{ tag }}:target) li:not([data-tags~="{{ tag }}"]) { display: none; }
{% endfor %}
</style>

<div class="tagged">
<nav class="tags">
    <a href="#all" id="all">all</a>
{% for tag in ordered %}
    {% assign count = references | where_exp: "reference", "reference.tags contains tag" | size %}
    {% include reference-tag.html tag=tag count=count %}
{% endfor %}
</nav>

<ul>
{% for reference in references %}
    <li data-tags="{{ reference.tags | join: ' ' }}">
        <a href="{{ reference.url }}">{{ reference.title }}</a>
        <span class="tags">{% for tag in reference.tags %}{% include reference-tag.html tag=tag %}{% endfor %}</span>
    </li>
{% endfor %}
</ul>
</div>
