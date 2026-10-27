//! Module: decimal
//!
//! Responsibility: validate and compare unsigned decimal text.
//! Does not own: field errors, amount bounds, cursor normalization, or numeric conversion.
//! Boundary: preserves arbitrary precision; leading zeroes are valid only in noncanonical text.

#[cfg(any(feature = "cloud-engine-host", feature = "icrc-host"))]
use std::cmp::Ordering;

pub fn is_unsigned_decimal(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(any(
    feature = "cloud-engine-host",
    feature = "dashboard-host",
    feature = "icrc-host",
    feature = "sns-host"
))]
pub fn is_canonical_unsigned_decimal(value: &str) -> bool {
    is_unsigned_decimal(value) && (value == "0" || !value.starts_with('0'))
}

/// Compare already-validated canonical unsigned decimal strings without parsing them.
#[cfg(any(feature = "cloud-engine-host", feature = "icrc-host"))]
pub fn compare_canonical_unsigned_decimal(left: &str, right: &str) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}
