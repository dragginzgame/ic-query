//! Module: sns::report::live::convert::common
//!
//! Responsibility: shared live conversion cleanup helpers.
//! Does not own: wire type definitions, report assembly, or rendering.
//! Boundary: normalizes optional text values before source/report models receive them.

/// Trim optional text and drop empty values.
pub(super) fn clean_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
