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
    let mut unit_index = CYCLE_UNITS
        .iter()
        .position(|(divisor, _)| whole.len() > divisor.ilog10() as usize)
        .unwrap_or(CYCLE_UNITS.len() - 1);
    let (mut rounded_whole, mut hundredths) =
        rounded_decimal_parts(whole, fraction, CYCLE_UNITS[unit_index].0.ilog10() as usize);
    if rounded_whole == "1000" && unit_index > 0 {
        unit_index -= 1;
        (rounded_whole, hundredths) =
            rounded_decimal_parts(whole, fraction, CYCLE_UNITS[unit_index].0.ilog10() as usize);
    }

    let number = match hundredths {
        0 => rounded_whole,
        value if value.is_multiple_of(10) => format!("{rounded_whole}.{}", value / 10),
        value => format!("{rounded_whole}.{value:02}"),
    };
    let unit = CYCLE_UNITS[unit_index].1;
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
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let whole = whole.trim_start_matches('0');
    Some((if whole.is_empty() { "0" } else { whole }, fraction))
}

fn rounded_decimal_parts(whole: &str, fraction: &str, exponent: usize) -> (String, u8) {
    let (rounded_whole, integer_fraction) = if whole.len() > exponent {
        let split = whole.len() - exponent;
        (&whole[..split], &whole[split..])
    } else {
        ("0", whole)
    };
    let mut digits = std::iter::repeat_n(b'0', exponent.saturating_sub(whole.len()))
        .chain(integer_fraction.bytes())
        .chain(fraction.bytes());
    let tenths = digits.next().unwrap_or(b'0') - b'0';
    let hundredth = digits.next().unwrap_or(b'0') - b'0';
    let mut hundredths = tenths * 10 + hundredth;
    if digits.next().unwrap_or(b'0') >= b'5' {
        hundredths += 1;
    }
    if hundredths == 100 {
        (increment_decimal(rounded_whole), 0)
    } else {
        (rounded_whole.to_string(), hundredths)
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
        for (value, expected) in [
            ("0", "0"),
            ("0.004", "0"),
            ("0.005", "0.01"),
            ("0.1", "0.1"),
            ("12.3", "12.3"),
            ("999.994", "999.99"),
            ("999.995", "1 k"),
            ("999999.995", "1 M"),
            ("000001000.0", "1 k"),
            ("1000000000", "1 B"),
            ("1000000000000", "1 T"),
            ("1000000000000000", "1 P"),
            ("1000000000000000000", "1 E"),
            (
                "340282366920938463463374607431768211456.789",
                "340282366920938463463.37 E",
            ),
        ] {
            assert_eq!(decimal_cycle_rate_text(value), expected);
        }
        assert_eq!(
            decimal_cycle_rate_text(&format!("1.234{}", "9".repeat(4096))),
            "1.23"
        );
        for invalid in ["1.2.3".to_string(), format!("1.234{}x", "9".repeat(4096))] {
            assert_eq!(decimal_cycle_rate_text(&invalid), invalid);
        }
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
