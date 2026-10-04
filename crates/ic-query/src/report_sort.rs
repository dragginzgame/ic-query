//! Module: report_sort
//!
//! Responsibility: compare common scalar report-view sort keys.
//! Does not own: report-specific sort fields, filters, or public direction models.
//! Boundary: preserves shared direction and missing-value ordering across report families.

use std::cmp::Ordering;

pub fn compare_optional_ascii_case_insensitive_text(
    left: Option<&str>,
    right: Option<&str>,
    descending: bool,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare_ascii_case_insensitive_text(left, right, descending),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub fn compare_ascii_case_insensitive_text(left: &str, right: &str, descending: bool) -> Ordering {
    let ordering = left
        .bytes()
        .map(|byte| byte.to_ascii_lowercase())
        .cmp(right.bytes().map(|byte| byte.to_ascii_lowercase()));
    if descending {
        ordering.reverse()
    } else {
        ordering
    }
}

pub fn compare_optional_ord<T>(left: Option<T>, right: Option<T>, descending: bool) -> Ordering
where
    T: Ord,
{
    match (left, right) {
        (Some(left), Some(right)) => compare_ord(left, right, descending),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub fn compare_ord<T>(left: T, right: T, descending: bool) -> Ordering
where
    T: Ord,
{
    if descending {
        right.cmp(&left)
    } else {
        left.cmp(&right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_ordering_folds_ascii_and_preserves_other_bytes() {
        for (left, right, expected) in [
            ("Alpha", "aLPHA", Ordering::Equal),
            ("", "a", Ordering::Less),
            ("A", "aa", Ordering::Less),
            ("beta", "Alpha", Ordering::Greater),
            ("_", "A", Ordering::Less),
            ("Ä", "ä", Ordering::Less),
            ("ÉA", "Éa", Ordering::Equal),
            ("z", "é", Ordering::Less),
        ] {
            assert_eq!(
                compare_ascii_case_insensitive_text(left, right, false),
                expected
            );
            assert_eq!(
                compare_ascii_case_insensitive_text(left, right, true),
                expected.reverse()
            );
        }
        for descending in [false, true] {
            assert_eq!(
                compare_optional_ascii_case_insensitive_text(Some("a"), None, descending),
                Ordering::Less
            );
            assert_eq!(
                compare_optional_ascii_case_insensitive_text(None, Some("a"), descending),
                Ordering::Greater
            );
        }
    }
}
