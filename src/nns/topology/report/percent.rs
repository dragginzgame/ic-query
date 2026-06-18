//! Module: nns::topology::report::percent
//!
//! Responsibility: format reusable NNS topology percentage values.
//! Does not own: report construction, text tables, or health checks.
//! Boundary: centralizes percentage math for capacity and coverage reports.

pub(super) fn coverage_percent_text(known: usize, unknown: usize) -> String {
    let total = known.saturating_add(unknown);
    ratio_percent_text(known as u128, total as u128)
}

pub(super) fn ratio_percent_text(numerator: u128, denominator: u128) -> String {
    if denominator == 0 {
        return "-".to_string();
    }
    let tenths = numerator
        .saturating_mul(1000)
        .saturating_add(denominator / 2)
        / denominator;
    format!("{}.{:01}%", tenths / 10, tenths % 10)
}
