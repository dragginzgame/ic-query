//! Module: human_quantity
//!
//! Responsibility: format exact cycle and byte counts for human-facing text.
//! Does not own: report fields, JSON serialization, or terminal styling.
//! Boundary: uses decimal cycle units and binary IEC byte units with bounded precision.

use crate::text_value::sanitize_text;

const CYCLE_UNITS: &[(u128, &str)] = &[
    (1_000_000_000_000_000_000, "E"),
    (1_000_000_000_000_000, "P"),
    (1_000_000_000_000, "T"),
    (1_000_000_000, "B"),
    (1_000_000, "M"),
    (1_000, "k"),
    (1, ""),
];
const DECIMAL_CYCLE_UNITS: &[(usize, &str)] = &[
    (18, "E"),
    (15, "P"),
    (12, "T"),
    (9, "B"),
    (6, "M"),
    (3, "k"),
    (0, ""),
];
const BYTE_UNITS: &[(u128, &str)] = &[
    (1_u128 << 60, "EiB"),
    (1_u128 << 50, "PiB"),
    (1_u128 << 40, "TiB"),
    (1_u128 << 30, "GiB"),
    (1_u128 << 20, "MiB"),
    (1_u128 << 10, "KiB"),
    (1, "B"),
];

pub fn cycle_count_text(value: u128) -> String {
    scaled_quantity_text(value, CYCLE_UNITS, 1_000)
}

pub fn decimal_cycle_count_text(value: &str) -> String {
    value
        .parse::<u128>()
        .map_or_else(|_| sanitize_text(value), cycle_count_text)
}

pub fn decimal_cycle_rate_text(value: &str) -> String {
    let Some((whole, fraction)) = decimal_parts(value) else {
        return sanitize_text(value);
    };
    let mut unit_index = DECIMAL_CYCLE_UNITS
        .iter()
        .position(|(exponent, _)| whole.len() > *exponent)
        .unwrap_or(DECIMAL_CYCLE_UNITS.len() - 1);
    let (mut rounded_whole, mut hundredths) =
        rounded_decimal_parts(whole, fraction, DECIMAL_CYCLE_UNITS[unit_index].0);
    if rounded_whole == "1000" && unit_index > 0 {
        unit_index -= 1;
        (rounded_whole, hundredths) =
            rounded_decimal_parts(whole, fraction, DECIMAL_CYCLE_UNITS[unit_index].0);
    }

    let number = match hundredths {
        0 => rounded_whole,
        value if value.is_multiple_of(10) => format!("{rounded_whole}.{}", value / 10),
        value => format!("{rounded_whole}.{value:02}"),
    };
    let unit = DECIMAL_CYCLE_UNITS[unit_index].1;
    if unit.is_empty() {
        number
    } else {
        format!("{number} {unit}")
    }
}

pub fn byte_count_text(value: u128) -> String {
    scaled_quantity_text(value, BYTE_UNITS, 1_024)
}

pub fn decimal_byte_count_text(value: &str) -> String {
    value
        .parse::<u128>()
        .map_or_else(|_| sanitize_text(value), byte_count_text)
}

fn scaled_quantity_text(value: u128, units: &[(u128, &str)], radix: u128) -> String {
    let mut unit_index = units
        .iter()
        .position(|(divisor, _)| value >= *divisor)
        .unwrap_or(units.len() - 1);
    let (mut whole, mut hundredths) = rounded_parts(value, units[unit_index].0);
    if whole >= radix && unit_index > 0 {
        unit_index -= 1;
        (whole, hundredths) = rounded_parts(value, units[unit_index].0);
    }

    let number = match hundredths {
        0 => whole.to_string(),
        value if value.is_multiple_of(10) => format!("{whole}.{}", value / 10),
        value => format!("{whole}.{value:02}"),
    };
    let unit = units[unit_index].1;
    if unit.is_empty() {
        number
    } else {
        format!("{number} {unit}")
    }
}

const fn rounded_parts(value: u128, divisor: u128) -> (u128, u128) {
    let mut whole = value / divisor;
    let remainder = value % divisor;
    let mut hundredths = (remainder * 100 + divisor / 2) / divisor;
    if hundredths == 100 {
        whole += 1;
        hundredths = 0;
    }
    (whole, hundredths)
}

fn decimal_parts(value: &str) -> Option<(&str, &str)> {
    let (whole, fraction) = value.split_once('.').map_or((value, ""), |parts| parts);
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.contains('.')
    {
        return None;
    }
    let whole = whole.trim_start_matches('0');
    Some((if whole.is_empty() { "0" } else { whole }, fraction))
}

fn rounded_decimal_parts(whole: &str, fraction: &str, exponent: usize) -> (String, u8) {
    let (rounded_whole, mut scaled_fraction) = if whole.len() > exponent {
        let split = whole.len() - exponent;
        (whole[..split].to_string(), whole[split..].to_string())
    } else {
        let mut scaled_fraction = "0".repeat(exponent - whole.len());
        scaled_fraction.push_str(whole);
        ("0".to_string(), scaled_fraction)
    };
    scaled_fraction.push_str(fraction);
    while scaled_fraction.len() < 3 {
        scaled_fraction.push('0');
    }

    let bytes = scaled_fraction.as_bytes();
    let mut hundredths = (bytes[0] - b'0') * 10 + (bytes[1] - b'0');
    if bytes[2] >= b'5' {
        hundredths += 1;
    }
    if hundredths == 100 {
        (increment_decimal(&rounded_whole), 0)
    } else {
        (rounded_whole, hundredths)
    }
}

fn increment_decimal(value: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_counts_use_trimmed_decimal_units() {
        assert_eq!(cycle_count_text(0), "0");
        assert_eq!(cycle_count_text(999), "999");
        assert_eq!(cycle_count_text(1_000), "1 k");
        assert_eq!(cycle_count_text(10_100_000_000_000), "10.1 T");
        assert_eq!(cycle_count_text(251_819_971_939_853), "251.82 T");
        assert_eq!(cycle_count_text(999_999_999_999), "1 T");
    }

    #[test]
    fn decimal_cycle_rates_use_exact_decimal_rounding() {
        assert_eq!(decimal_cycle_rate_text("40067084771.847176"), "40.07 B");
        assert_eq!(decimal_cycle_rate_text("999999999999.9"), "1 T");
        assert_eq!(decimal_cycle_rate_text("0.125"), "0.13");
        assert_eq!(decimal_cycle_rate_text("not-a-number"), "not-a-number");
    }

    #[test]
    fn byte_counts_use_trimmed_binary_iec_units() {
        assert_eq!(byte_count_text(0), "0 B");
        assert_eq!(byte_count_text(1_023), "1023 B");
        assert_eq!(byte_count_text(1_024), "1 KiB");
        assert_eq!(byte_count_text(1_372), "1.34 KiB");
        assert_eq!(byte_count_text(108_782_218), "103.74 MiB");
        assert_eq!(byte_count_text(1_048_575), "1 MiB");
    }

    #[test]
    fn decimal_text_falls_back_without_losing_unrepresentable_evidence() {
        let oversized = "340282366920938463463374607431768211456";
        assert_eq!(decimal_cycle_count_text(oversized), oversized);
        assert_eq!(decimal_byte_count_text(oversized), oversized);
    }
}
