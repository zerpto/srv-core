//! Cleaning text that comes from a client.

/// Trims an optional text field; blank becomes `None`.
#[must_use]
pub fn non_blank(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

/// A name that is safe to show and to store: no folders, no control
/// characters. A blank name becomes `<fallback>.<extension>`.
#[must_use]
pub fn clean_filename(name: &str, fallback: &str, extension: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or_default();
    let cleaned: String = base.chars().filter(|c| !c.is_control()).collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        format!("{fallback}.{extension}")
    } else {
        cleaned.chars().take(200).collect()
    }
}
