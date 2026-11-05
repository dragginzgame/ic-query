//! Module: report_sort
//!
//! Responsibility: sort report rows and compare common scalar view keys.
//! Does not own: report-specific sort fields, filters, or public direction models.
//! Boundary: preserves shared direction and missing-value ordering across report families.

use std::cmp::Ordering;

///
/// ReportSortKey
///
/// A selected scalar key whose value and missing-value ordering use shared report rules.
///

pub enum ReportSortKey<Row> {
    OptionalU64(fn(&Row) -> Option<u64>),
    Text(fn(&Row) -> &str),
    OptionalText(fn(&Row) -> Option<&str>),
}

impl<Row> ReportSortKey<Row> {
    pub fn sort(
        self,
        rows: &mut [Row],
        descending: bool,
        tiebreak: impl Fn(&Row, &Row) -> Ordering,
    ) {
        match self {
            Self::OptionalU64(key) => sort_by_optional_u64(rows, descending, key, tiebreak),
            Self::Text(key) => sort_by_text(rows, descending, key, tiebreak),
            Self::OptionalText(key) => sort_by_optional_text(rows, descending, key, tiebreak),
        }
    }
}

pub fn sort_by_optional_u64<Row>(
    rows: &mut [Row],
    descending: bool,
    key: impl Fn(&Row) -> Option<u64>,
    tiebreak: impl Fn(&Row, &Row) -> Ordering,
) {
    rows.sort_by(|left, right| {
        compare_optional_ord(key(left), key(right), descending).then_with(|| tiebreak(left, right))
    });
}

pub fn sort_by_text<Row>(
    rows: &mut [Row],
    descending: bool,
    key: impl Fn(&Row) -> &str,
    tiebreak: impl Fn(&Row, &Row) -> Ordering,
) {
    rows.sort_by(|left, right| {
        compare_ascii_case_insensitive_text(key(left), key(right), descending)
            .then_with(|| tiebreak(left, right))
    });
}

pub fn sort_by_optional_text<Row>(
    rows: &mut [Row],
    descending: bool,
    key: impl Fn(&Row) -> Option<&str>,
    tiebreak: impl Fn(&Row, &Row) -> Ordering,
) {
    rows.sort_by(|left, right| {
        compare_optional_ascii_case_insensitive_text(key(left), key(right), descending)
            .then_with(|| tiebreak(left, right))
    });
}

pub fn compare_optional_ascii_case_insensitive_text(
    left: Option<&str>,
    right: Option<&str>,
    descending: bool,
) -> Ordering {
    compare_optional_by(left, right, |left, right| {
        compare_ascii_case_insensitive_text(left, right, descending)
    })
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
    T: Ord + Copy,
{
    compare_optional_by(left, right, |left, right| {
        compare_ord(left, right, descending)
    })
}

fn compare_optional_by<T>(
    left: Option<T>,
    right: Option<T>,
    compare: impl FnOnce(T, T) -> Ordering,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare(left, right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub fn compare_ord<T>(left: T, right: T, descending: bool) -> Ordering
where
    T: Ord + Copy,
{
    if descending {
        right.cmp(&left)
    } else {
        left.cmp(&right)
    }
}

#[cfg(test)]
mod tests;
