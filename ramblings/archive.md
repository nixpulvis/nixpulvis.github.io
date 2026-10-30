---
title: Archive
layout: default
permalink: /ramblings/archive
---
{% import "items.html" as items %}

# Archive

Old ramblings I've moved off the [main list](/ramblings).

<ul class="posts">
{%- for rambling in site.ramblings() -%}
    {%- if rambling.front.extra.archived -%}
        {%- call items::post_item(rambling) -%}{%- endcall -%}
    {%- endif -%}
{%- endfor -%}
</ul>
