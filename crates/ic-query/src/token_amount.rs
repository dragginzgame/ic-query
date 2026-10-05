//! Module: token_amount
//!
//! Responsibility: render token base-unit amounts as two-decimal human text.
//!
//! Does not own: token metadata lookup, report row selection, or JSON amount fields.
//!
//! Boundary: keeps display rounding for token amounts centralized while preserving
//! raw amount values in typed reports and JSON output.

/// Renders a base-unit token amount with two decimal places.
///
/// Numeric input may contain surrounding whitespace and `_` separators. Other
/// input is returned unchanged so callers can preserve upstream sentinel values.
#[must_use]
pub fn base_units_decimal_text(value: &str, decimals: u8) -> String {
    let Some(digits) = normalized_base_unit_digits(value) else {
        return value.to_string();
    };
    let digits = digits.as_str();
    let decimals = usize::from(decimals);
    let hundredths = if decimals <= 2 {
        scaled_hundredths(digits, 2 - decimals)
    } else {
        rounded_hundredths(digits, decimals - 2)
    };
    format_hundredths(&hundredths)
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

fn scaled_hundredths(digits: &str, trailing_zero_count: usize) -> String {
    let mut hundredths = digits.to_string();
    hundredths.push_str(&"0".repeat(trailing_zero_count));
    hundredths
}

fn rounded_hundredths(digits: &str, discarded_digit_count: usize) -> String {
    let (hundredths, discarded) = if digits.len() > discarded_digit_count {
        digits.split_at(digits.len() - discarded_digit_count)
    } else {
        ("0", digits)
    };
    // A shorter suffix has an implicit leading zero at the rounding position.
    if discarded.len() == discarded_digit_count
        && discarded
            .as_bytes()
            .first()
            .is_some_and(|digit| *digit >= b'5')
    {
        increment_decimal_string(hundredths)
    } else {
        hundredths.to_string()
    }
}

fn increment_decimal_string(digits: &str) -> String {
    let mut reversed = String::with_capacity(digits.len() + 1);
    let mut carry = true;
    for digit in digits.bytes().rev() {
        if carry {
            if digit == b'9' {
                reversed.push('0');
            } else {
                reversed.push(char::from(digit + 1));
                carry = false;
            }
        } else {
            reversed.push(char::from(digit));
        }
    }
    if carry {
        reversed.push('1');
    }
    reversed.chars().rev().collect()
}

fn format_hundredths(hundredths: &str) -> String {
    let hundredths = hundredths.trim_start_matches('0');
    let hundredths = if hundredths.is_empty() {
        "0"
    } else {
        hundredths
    };
    if hundredths.len() <= 2 {
        return format!("0.{hundredths:0>2}");
    }

    let (whole, fractional) = hundredths.split_at(hundredths.len() - 2);
    format!("{whole}.{fractional}")
}
