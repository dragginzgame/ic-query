use super::*;

#[test]
fn percentages_keep_nearest_tenths_and_wide_capacity_inputs() {
    for (numerator, denominator, expected) in [
        (0, 0, "-"),
        (0, 1, "0.0%"),
        (1, 16, "6.3%"),
        (1, 2001, "0.0%"),
        (1, 2000, "0.1%"),
        (1, 1999, "0.1%"),
        (1, 3, "33.3%"),
        (2, 3, "66.7%"),
        (3, 2, "150.0%"),
        (2, 1, "200.0%"),
        (u64::MAX, u64::MAX, "100.0%"),
        (u64::MAX - 1, u64::MAX, "100.0%"),
        (u64::MAX, u64::MAX - 1, "100.0%"),
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
