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
mod tests;
