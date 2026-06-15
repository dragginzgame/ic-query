use super::super::percent::ratio_percent_text;

pub(in crate::nns::topology::report) fn coverage_percent_text(
    known: usize,
    unknown: usize,
) -> String {
    let total = known.saturating_add(unknown);
    ratio_percent_text(known as u128, total as u128)
}
