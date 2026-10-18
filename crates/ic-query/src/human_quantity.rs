//! Module: human_quantity
//!
//! Responsibility: format exact cycle and byte counts for human-facing text.
//! Does not own: report fields, JSON serialization, or terminal styling.
//! Boundary: uses decimal cycle units and binary IEC byte units with bounded precision.

use ic_host_text::{decimal::compact_decimal_text, escape::terminal_text};
use std::fmt::Write;

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
        .map_or_else(|_| terminal_text(value).into_owned(), cycle_count_text)
}

pub fn decimal_cycle_rate_text(value: &str) -> String {
    let whole = value.split_once('.').map_or(value, |(whole, _)| whole);
    let whole = whole.trim_start_matches('0');
    let mut unit_index = CYCLE_UNITS
        .iter()
        .position(|(divisor, _)| whole.len() > divisor.ilog10() as usize)
        .unwrap_or(CYCLE_UNITS.len() - 1);
    let Ok(mut number) = compact_decimal_text(value, CYCLE_UNITS[unit_index].0.ilog10() as usize)
    else {
        return terminal_text(value).into_owned();
    };
    if number == "1000" && unit_index > 0 {
        unit_index -= 1;
        number = compact_decimal_text(value, CYCLE_UNITS[unit_index].0.ilog10() as usize)
            .expect("cycle rate decimal syntax was admitted before unit promotion");
    }
    let unit = CYCLE_UNITS[unit_index].1;
    if !unit.is_empty() {
        number.push(' ');
        number.push_str(unit);
    }
    number
}

pub fn byte_count_text(value: u128) -> String {
    scaled_quantity_text(value, BYTE_UNITS, 1_024)
}

pub fn decimal_byte_count_text(value: &str) -> String {
    value
        .parse::<u128>()
        .map_or_else(|_| terminal_text(value).into_owned(), byte_count_text)
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

    quantity_text(whole.to_string(), hundredths, units[unit_index].1)
}

fn quantity_text(mut number: String, hundredths: u128, unit: &str) -> String {
    if hundredths != 0 {
        write!(number, ".{hundredths:02}").expect("writing to a String cannot fail");
        if hundredths.is_multiple_of(10) {
            number.pop();
        }
    }
    if !unit.is_empty() {
        number.push(' ');
        number.push_str(unit);
    }
    number
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
