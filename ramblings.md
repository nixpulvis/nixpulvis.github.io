---
title: Ramblings
layout: page
---
{% import "items.html" as items %}

Here in lies a collection of words both technical and opinionated. I make no
promise of complete clearity, nor do I wish to ramble to the point of insanity.
If you find something you like here, I've done my job. Otherwise, be gone with
you.

<ul class="posts">
{%- for rambling in site.ramblings() -%}
    {%- if !rambling.front.extra.hidden && !rambling.front.extra.archived -%}
        {%- call items::post_item(rambling) -%}{%- endcall -%}
    {%- endif -%}
{%- endfor -%}
</ul>

<small><a href="/ramblings/archive">Archive</a></small>
