---
layout: default
---

<h2 class="title">{{ page.title }}</h2>
<p class="meta">
    {{ page.content | strip_html | number_of_words }} words
    written by Nathan Lilienthal
</p>

{{ content }}
