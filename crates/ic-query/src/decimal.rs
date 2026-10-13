//! Module: decimal
//!
//! Responsibility: validate, increment and compare unsigned decimal text.
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

pub fn increment_decimal(value: &str) -> String {
    let mut digits = value.as_bytes().to_vec();
    for digit in digits.iter_mut().rev() {
        if *digit < b'9' {
            *digit += 1;
            return String::from_utf8(digits).expect("ASCII decimal digits remain UTF-8");
        }
        *digit = b'0';
    }
    digits.insert(0, b'1');
    String::from_utf8(digits).expect("ASCII decimal digits remain UTF-8")
}
