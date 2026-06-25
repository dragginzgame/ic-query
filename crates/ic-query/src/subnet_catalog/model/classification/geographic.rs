//! Module: subnet_catalog::model::classification::geographic
//!
//! Defines stable geographic-scope labels used by catalog data.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

///
/// GeographicScope
///
/// Geographic scope classification for a subnet.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeographicScope {
    /// Global subnet scope.
    Global,
    /// European subnet scope.
    Europe,
    /// Unknown or unclassified geographic scope.
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
