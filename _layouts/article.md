---
layout: default
---

<h2>
    {{ page.title }}
    <small>
        {{ page.content | strip_html | number_of_words }} words
        written by Nathan Lilienthal
    </small>
</h2>

{{ content }}
