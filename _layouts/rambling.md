---
layout: default
---

<h2>
    {{ page.title }}
    <small>
        {{ page.content | number_of_words }} words
        {% if page.draft %}
            drafted
        {% else %}
            published
        {% endif %}
        on {{ page.date | date: "%B %d, %Y" }}.
    </small>
</h2>

{{ content }}
