---
layout: default
---

# References

<ul>
{% for ref in site.references %}
    <li><a href="{{ ref.url }}">{{ ref.title }}</a></li>
{% endfor %}
</ul>
