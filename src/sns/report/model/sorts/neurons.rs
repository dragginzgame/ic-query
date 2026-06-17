//! Module: sns::report::model::sorts::neurons
//!
//! Responsibility: SNS neuron report sort model.
//! Does not own: CLI parsing, cache loading, or row ordering implementation.
//! Boundary: names supported report-level sorting for SNS neuron listings.

///
/// SnsNeuronsSort
///
/// Report-model sort selector for SNS neuron listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsNeuronsSort {
    #[default]
    Api,
    Id,
    Stake,
    Maturity,
    Created,
}

impl SnsNeuronsSort {
    /// Return the stable label used in text and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Id => "id",
            Self::Stake => "stake",
            Self::Maturity => "maturity",
            Self::Created => "created",
        }
    }

    /// Return whether this sort requires a complete local snapshot.
    #[must_use]
    pub const fn uses_cache(self) -> bool {
        !matches!(self, Self::Api)
    }
}
