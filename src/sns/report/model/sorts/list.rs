//! Module: sns::report::model::sorts::list
//!
//! Responsibility: SNS list report sort model.
//! Does not own: CLI parsing or deployed-SNS fetch behavior.
//! Boundary: names supported report-level sorting for deployed SNS listings.

///
/// SnsListSort
///
/// Report-model sort selector for deployed SNS listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsListSort {
    #[default]
    Id,
    Name,
}

impl SnsListSort {
    /// Return the stable label used in text and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
    }
}
