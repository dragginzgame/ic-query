//! Module: sns::report::model::sorts::neurons
//!
//! Responsibility: SNS neuron report sort model.
//! Does not own: CLI parsing, cache loading, or row ordering implementation.
//! Boundary: names supported report-level sorting for SNS neuron listings.

///
/// SnsNeuronsCacheSort
///
/// Report-model sort selector for SNS neuron listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsNeuronsCacheSort {
    /// Ascending canonical neuron identity.
    #[default]
    Id,
    /// Descending cached stake, then ascending identity.
    Stake,
    /// Descending maturity, then ascending identity.
    Maturity,
    /// Descending creation time, then ascending identity.
    Created,
}

impl SnsNeuronsCacheSort {
    /// Return the stable label used in text and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Stake => "stake",
            Self::Maturity => "maturity",
            Self::Created => "created",
        }
    }
}
