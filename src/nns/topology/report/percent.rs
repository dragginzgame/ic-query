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
