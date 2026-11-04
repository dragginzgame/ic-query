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
mod tests;
