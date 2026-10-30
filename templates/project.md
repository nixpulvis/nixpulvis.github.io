<h2 class="title">
    {% if let Some(status) = page.front.status %}<span class="status {{ page.status_class() }}">{{ status }}</span>{% endif %}
    {{ page.title() }}
</h2>

<ul class="references">
{%- for reference in page.front.references %}
    <li><a href="{{ reference }}">{{ reference }}</a></li>
{%- endfor %}
</ul>

{{ content }}
