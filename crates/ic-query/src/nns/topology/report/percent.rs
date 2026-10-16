//! Module: nns::topology::report::percent
//!
//! Responsibility: format reusable NNS topology percentage values.
//! Does not own: report construction, text tables, or consistency checks.
//! Boundary: centralizes percentage math for capacity and coverage reports.

use ic_metrics::checked_scaled_ratio_nearest;

pub(super) fn coverage_percent_text(known: usize, unknown: usize) -> String {
    let Some(total) = known.checked_add(unknown) else {
        return "-".to_string();
    };
    let (Ok(known), Ok(total)) = (u64::try_from(known), u64::try_from(total)) else {
        return "-".to_string();
    };
    ratio_percent_text(known, total)
}

pub(super) fn ratio_percent_text(numerator: u64, denominator: u64) -> String {
    let Ok(tenths) = checked_scaled_ratio_nearest(u128::from(numerator), denominator, 1000) else {
        return "-".to_string();
    };
    format!("{}.{:01}%", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentages_keep_nearest_tenths_and_wide_capacity_inputs() {
        for (numerator, denominator, expected) in [
            (0, 0, "-"),
            (0, 1, "0.0%"),
            (1, 16, "6.3%"),
            (1, 3, "33.3%"),
            (2, 3, "66.7%"),
            (3, 2, "150.0%"),
            (u64::MAX, u64::MAX, "100.0%"),
            (u64::MAX, 1, "1844674407370955161500.0%"),
        ] {
            assert_eq!(ratio_percent_text(numerator, denominator), expected);
        }
    }

    #[test]
    fn coverage_requires_an_exact_representable_total() {
        for (known, unknown, expected) in [
            (0, 0, "-"),
            (0, 10, "0.0%"),
            (1, 15, "6.3%"),
            (1, 2, "33.3%"),
            (10, 0, "100.0%"),
            (usize::MAX, 0, "100.0%"),
            (usize::MAX, 1, "-"),
        ] {
            assert_eq!(coverage_percent_text(known, unknown), expected);
        }
    }
}
