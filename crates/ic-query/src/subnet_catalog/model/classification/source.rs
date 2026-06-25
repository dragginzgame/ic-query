//! Module: subnet_catalog::model::classification::source
//!
//! Defines provenance labels for catalog classification values.

use serde::{Deserialize, Serialize};

///
/// ClassificationSource
///
/// Provenance for one classification value in a subnet catalog record.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationSource {
    /// Value came directly from registry data.
    Registry,
    /// Value came from curated project metadata.
    Curated,
    /// Value was computed from other catalog data.
    Computed,
    /// Value source is unknown.
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
