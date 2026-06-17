//! Module: subnet_catalog::model::classification::source
//!
//! Responsibility: identify where a catalog classification value came from.
//!
//! Does not own: registry fetches, curated metadata, or classification algorithms.
//!
//! Boundary: records provenance for already-computed classification fields.

use serde::{Deserialize, Serialize};

/// Provenance for one classification value in a subnet catalog record.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationSource {
    Registry,
    Curated,
    Computed,
    Unknown,
}

impl ClassificationSource {
    /// Returns the stable snake_case value used in report text.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Registry => "registry",
            Self::Curated => "curated",
            Self::Computed => "computed",
            Self::Unknown => "unknown",
        }
    }
}
