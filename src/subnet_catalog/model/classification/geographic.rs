//! Module: subnet_catalog::model::classification::geographic
//!
//! Responsibility: define stable geographic-scope values for catalog data.
//!
//! Does not own: scope derivation, registry fetching, or output rendering.
//!
//! Boundary: keeps serialized scope values aligned with CLI filter syntax.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Geographic scope classification for a subnet.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeographicScope {
    Global,
    Europe,
    Unknown,
}

impl GeographicScope {
    /// Returns the stable snake_case value used in CLI filters and text output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Europe => "europe",
            Self::Unknown => "unknown",
        }
    }
}

impl FromStr for GeographicScope {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "global" => Ok(Self::Global),
            "europe" => Ok(Self::Europe),
            "unknown" => Ok(Self::Unknown),
            other => Err(format!(
                "invalid value {other}; use global, europe, or unknown"
            )),
        }
    }
}
