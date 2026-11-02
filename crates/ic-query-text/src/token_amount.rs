//! Module: token_amount
//!
//! Responsibility: render token base-unit amounts as two-decimal human text.
//!
//! Does not own: token metadata lookup, report row selection, or JSON amount fields.
//!
//! Boundary: keeps display rounding for token amounts centralized while preserving
//! raw amount values in typed reports and JSON output.

use ic_host_text::decimal::fixed_decimal_text;

/// Renders a base-unit token amount with two decimal places.
///
/// Numeric input may contain surrounding whitespace and `_` separators. Other
/// input is returned unchanged so callers can preserve upstream sentinel values.
#[must_use]
pub fn base_units_decimal_text(value: &str, decimals: u8) -> String {
    let Some(digits) = normalized_base_unit_digits(value) else {
        return value.to_string();
    };
    fixed_decimal_text(&digits, usize::from(decimals))
        .expect("normalized base units contain only decimal digits")
}

/// Renders an ICP-style e8s amount with two decimal places.
#[must_use]
pub fn e8s_decimal_text(value: u64) -> String {
    base_units_decimal_text(&value.to_string(), 8)
}

fn normalized_base_unit_digits(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let mut digits = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_digit() {
            digits.push(char::from(byte));
        } else if byte != b'_' {
            return None;
        }
    }

    let leading_zero_count = digits.len() - digits.trim_start_matches('0').len();
    digits.replace_range(..leading_zero_count, "");
    if digits.is_empty() {
        digits.push('0');
    }
    Some(digits)
}
