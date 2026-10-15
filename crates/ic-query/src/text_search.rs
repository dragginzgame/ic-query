//! Module: text_search
//!
//! Responsibility: shared helpers for small human-text filters.
//! Does not own: command parsing, report filtering, or locale-sensitive search.
//! Boundary: provides Unicode-aware case-insensitive matching for view predicates.

/// Match one field against a query lowercased once by its caller.
pub fn optional_text_contains_lowercase_query(value: Option<&str>, lowercase_query: &str) -> bool {
    value.is_some_and(|value| {
        lowercase_query.is_empty() || value.to_lowercase().contains(lowercase_query)
    })
}

#[cfg(test)]
mod tests {
    use super::optional_text_contains_lowercase_query;

    #[test]
    fn optional_text_contains_case_insensitive_matches_unicode_text() {
        assert!(optional_text_contains_lowercase_query(
            Some("Subnet Upgrade"),
            "subnet"
        ));
        assert!(optional_text_contains_lowercase_query(
            Some("Subnet Upgrade"),
            "upgrade"
        ));
        assert!(optional_text_contains_lowercase_query(
            Some("RÉSUMÉ proposal"),
            "résumé"
        ));
        assert!(!optional_text_contains_lowercase_query(
            Some("Node Provider"),
            "subnet"
        ));
        assert!(!optional_text_contains_lowercase_query(None, "subnet"));
        assert!(optional_text_contains_lowercase_query(Some(""), ""));
        assert!(!optional_text_contains_lowercase_query(None, ""));
    }
}
