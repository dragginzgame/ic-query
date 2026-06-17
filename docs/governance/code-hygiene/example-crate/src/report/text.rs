//! Module: report::text
//!
//! Responsibility: report row labels and row-kind classification.
//! Does not own: query validation or cache side effects.
//! Boundary: validates row labels before text rendering receives them.

use crate::diagnostic::StyleDiagnostic;

///
/// ReportRowKind
///
/// Coarse report row family selected by the owner module.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportRowKind {
    CacheRead,

    CacheRefresh,
}

impl ReportRowKind {
    /// Return whether this report row describes cache replacement.
    #[must_use]
    pub const fn is_refresh(self) -> bool {
        matches!(self, Self::CacheRefresh)
    }
}

///
/// ReportRow
///
/// Validated report row selected by a query owner before rendering.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportRow {
    kind: ReportRowKind,
    label: String,
}

impl ReportRow {
    /// Build one validated report row.
    pub fn new(kind: ReportRowKind, label: impl Into<String>) -> Result<Self, StyleDiagnostic> {
        let label = label.into();
        let label = label.trim();

        if label.is_empty() {
            return Err(StyleDiagnostic::empty_report_label());
        }

        Ok(Self {
            kind,
            label: label.to_owned(),
        })
    }

    /// Return the report-row family.
    #[must_use]
    pub const fn kind(&self) -> ReportRowKind {
        self.kind
    }

    /// Return the normalized report label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

///
/// TESTS
///

#[cfg(test)]
mod tests {
    use crate::{
        diagnostic::StyleDiagnosticCode,
        report::{ReportRow, ReportRowKind},
    };

    #[test]
    fn report_labels_are_normalized() {
        let row = ReportRow::new(ReportRowKind::CacheRead, " sns-neurons ")
            .expect("trimmed report labels should be valid");

        assert_eq!(row.label(), "sns-neurons");
        assert!(!row.kind().is_refresh());
    }

    #[test]
    fn empty_report_labels_return_typed_diagnostic() {
        let err =
            ReportRow::new(ReportRowKind::CacheRead, " ").expect_err("empty labels should fail");

        assert_eq!(err.code(), StyleDiagnosticCode::EmptyReportLabel);
    }
}
