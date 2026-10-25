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
{% for tag in tags %}
    {% assign count = references | where_exp: "reference", "reference.tags contains tag" | size %}
    <a href="#{{ tag }}" id="{{ tag }}">{{ tag }} <small>{{ count }}</small></a>
{% endfor %}
</nav>

<ul>
{% for reference in references %}
    <li data-tags="{{ reference.tags | join: ' ' }}">
        <a href="{{ reference.url }}">{{ reference.title }}</a>
        <span class="tags">{% for tag in reference.tags %}<a href="#{{ tag }}">{{ tag }}</a>{% endfor %}</span>
    </li>
{% endfor %}
</ul>
</div>
