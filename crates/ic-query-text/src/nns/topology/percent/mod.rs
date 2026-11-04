//! Module: nns::topology::percent
//!
//! Responsibility: format reusable NNS topology percentage values.
//! Does not own: report construction, text tables, or consistency checks.
//! Boundary: centralizes percentage math for capacity and coverage reports.

/// Format known and unknown relationship counts as a nearest-tenth percentage.
#[must_use]
pub(super) fn coverage_percent_text(known: usize, unknown: usize) -> String {
    let Some(total) = known.checked_add(unknown) else {
        return "-".to_string();
    };
    let (Ok(known), Ok(total)) = (u64::try_from(known), u64::try_from(total)) else {
        return "-".to_string();
    };
    ratio_percent_text(known, total)
}

/// Format a ratio as a nearest-tenth percentage, preserving unavailable totals.
#[must_use]
pub(super) fn ratio_percent_text(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        return "-".to_string();
    }
    // u64 inputs leave room in u128 for scaling and rounding exact halves upward.
    let denominator = u128::from(denominator);
    let tenths = (u128::from(numerator) * 1000 + denominator / 2) / denominator;
    format!("{}.{:01}%", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests;
