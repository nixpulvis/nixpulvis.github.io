---
layout: default
title: References
---

# References

<ul>
{% for ref in site.references %}
    <li><a href="{{ ref.url | relative_url }}">{{ ref.title }}</a></li>
{% endfor %}
</ul>
