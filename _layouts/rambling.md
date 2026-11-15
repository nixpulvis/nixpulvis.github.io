---
layout: default
---

<h2 class="title">{{ page.title }}</h2>
<p class="meta">
    {{ page.content | strip_html | number_of_words }} words
    {% if page.draft %}drafted{% else %}published{% endif %}
    on {{ page.date | date: "%B %d, %Y" }}
    <span class="comments-controls" hidden>
        <button type="button" class="comments-toggle"></button>
        <button type="button" class="comments-open" hidden>Read comments</button>
    </span>
</p>

{{ content }}
