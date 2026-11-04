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
