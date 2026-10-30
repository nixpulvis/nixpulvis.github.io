//! Front matter, the YAML between `---` lines at the top of a page.

/// Splits the YAML between the `---` lines at the top of `text` off the
/// rest. Without them, it's all the rest.
pub(crate) fn split(text: &str) -> (&str, &str) {
    let Some(rest) = text.strip_prefix("---\n") else {
        return ("", text);
    };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" || line.trim_end() == "..." {
            return (&rest[..offset], &rest[offset + line.len()..]);
        }
        offset += line.len();
    }
    ("", text)
}
