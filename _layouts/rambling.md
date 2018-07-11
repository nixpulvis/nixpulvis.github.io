---
layout: default
---

<h2>
    {{ page.title }}
    <small>
        {{ page.content | number_of_words }} words
        published on {{ page.date | date: "%B %d, %Y" }}.
    </small>
</h2>

{{ content }}
