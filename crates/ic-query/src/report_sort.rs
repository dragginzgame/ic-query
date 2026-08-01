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
    compare_ord(
        left.to_ascii_lowercase(),
        right.to_ascii_lowercase(),
        descending,
    )
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
