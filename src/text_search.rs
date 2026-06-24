//! Module: text_search
//!
//! Responsibility: shared helpers for small human-text filters.
//! Does not own: command parsing, report filtering, or locale-sensitive search.
//! Boundary: provides allocation-free ASCII case-insensitive matching for view predicates.

pub fn optional_text_contains_ascii_case_insensitive(value: Option<&str>, query: &str) -> bool {
    value.is_some_and(|value| text_contains_ascii_case_insensitive(value, query))
}

fn text_contains_ascii_case_insensitive(value: &str, query: &str) -> bool {
    let query = query.as_bytes();
    query.is_empty()
        || value
            .as_bytes()
            .windows(query.len())
            .any(|window| window.eq_ignore_ascii_case(query))
}

#[cfg(test)]
mod tests {
    use super::optional_text_contains_ascii_case_insensitive;

    #[test]
    fn optional_text_contains_ascii_case_insensitive_matches_ascii_text() {
        assert!(optional_text_contains_ascii_case_insensitive(
            Some("Subnet Upgrade"),
            "subnet"
        ));
        assert!(optional_text_contains_ascii_case_insensitive(
            Some("Subnet Upgrade"),
            "UPGRADE"
        ));
        assert!(!optional_text_contains_ascii_case_insensitive(
            Some("Node Provider"),
            "subnet"
        ));
        assert!(!optional_text_contains_ascii_case_insensitive(
            None, "subnet"
        ));
    }
}
