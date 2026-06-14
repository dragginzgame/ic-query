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

    let digits = digits.trim_start_matches('0');
    Some(if digits.is_empty() {
        "0".to_string()
    } else {
        digits.to_string()
    })
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
    let discarded = left_pad_digits(discarded, discarded_digit_count);
    if discarded
        .as_bytes()
        .first()
        .is_some_and(|digit| *digit >= b'5')
    {
        increment_decimal_string(hundredths)
    } else {
        hundredths.to_string()
    }
}

fn left_pad_digits(digits: &str, width: usize) -> String {
    if digits.len() >= width {
        return digits.to_string();
    }
    let mut padded = "0".repeat(width - digits.len());
    padded.push_str(digits);
    padded
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

#[cfg(test)]
mod tests {
    use super::{base_units_decimal_text, e8s_decimal_text};

    #[test]
    fn base_units_render_as_two_decimal_token_amounts() {
        assert_eq!(base_units_decimal_text("0", 8), "0.00");
        assert_eq!(base_units_decimal_text("000000000", 8), "0.00");
        assert_eq!(base_units_decimal_text("10_000", 8), "0.00");
        assert_eq!(
            base_units_decimal_text("100_923_109_141_460", 8),
            "1009231.09"
        );
        assert_eq!(base_units_decimal_text("500000", 8), "0.01");
        assert_eq!(base_units_decimal_text("123456789", 8), "1.23");
        assert_eq!(base_units_decimal_text("123500000", 8), "1.24");
        assert_eq!(base_units_decimal_text("3000000000000", 8), "30000.00");
        assert_eq!(base_units_decimal_text("123", 0), "123.00");
        assert_eq!(base_units_decimal_text("123", 1), "12.30");
        assert_eq!(base_units_decimal_text("123", 2), "1.23");
        assert_eq!(base_units_decimal_text("999", 3), "1.00");
        assert_eq!(base_units_decimal_text("not-a-number", 8), "not-a-number");
    }

    #[test]
    fn e8s_render_as_two_decimal_token_amounts() {
        assert_eq!(e8s_decimal_text(0), "0.00");
        assert_eq!(e8s_decimal_text(123), "0.00");
        assert_eq!(e8s_decimal_text(499_999), "0.00");
        assert_eq!(e8s_decimal_text(500_000), "0.01");
        assert_eq!(e8s_decimal_text(100_000_000), "1.00");
        assert_eq!(e8s_decimal_text(123_456_789), "1.23");
        assert_eq!(e8s_decimal_text(123_500_000), "1.24");
        assert_eq!(e8s_decimal_text(3_000_000_000_000), "30000.00");
    }
}
