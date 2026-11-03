//! Module: human_quantity
//!
//! Responsibility: format exact cycle and byte counts for human-facing text.
//! Does not own: report fields, JSON serialization, or terminal styling.
//! Boundary: uses decimal cycle units and binary IEC byte units with bounded precision.

use ic_host_text::decimal::compact_decimal_text;
use ic_host_text::escape::terminal_text;
use ic_host_text::quantity::scaled_quantity_text;

const CYCLE_UNITS: &[(u64, &str)] = &[
    (1_000_000_000_000_000_000, "E"),
    (1_000_000_000_000_000, "P"),
    (1_000_000_000_000, "T"),
    (1_000_000_000, "B"),
    (1_000_000, "M"),
    (1_000, "k"),
    (1, ""),
];
const BYTE_UNITS: &[(u64, &str)] = &[
    (1_u64 << 60, "EiB"),
    (1_u64 << 50, "PiB"),
    (1_u64 << 40, "TiB"),
    (1_u64 << 30, "GiB"),
    (1_u64 << 20, "MiB"),
    (1_u64 << 10, "KiB"),
    (1, "B"),
];

pub fn cycle_count_text(value: u128) -> String {
    scaled_quantity_text(value, CYCLE_UNITS).expect("cycle unit divisors form a valid scale")
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
    scaled_quantity_text(value, BYTE_UNITS).expect("byte unit divisors form a valid scale")
}

pub fn decimal_byte_count_text(value: &str) -> String {
    value
        .parse::<u128>()
        .map_or_else(|_| terminal_text(value).into_owned(), byte_count_text)
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
