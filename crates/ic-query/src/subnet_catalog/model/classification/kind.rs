//! Module: subnet_catalog::model::classification::kind
//!
//! Defines stable subnet-kind labels and their default charging meaning.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

///
/// SubnetKind
///
/// Subnet execution kind from registry or derived catalog classification.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubnetKind {
    /// Application subnet.
    Application,
    /// Cloud Engine subnet.
    CloudEngine,
    /// System subnet.
    System,
    /// Unknown or unclassified subnet kind.
    Unknown,
}

impl SubnetKind {
    /// Returns the stable snake_case value used in CLI filters and text output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::CloudEngine => "cloud_engine",
            Self::System => "system",
            Self::Unknown => "unknown",
        }
    }

    /// Returns whether a subject on this subnet kind normally incurs application charges.
    #[must_use]
    pub const fn charges_apply_by_default(self) -> bool {
        matches!(self, Self::Application | Self::CloudEngine)
    }
}

impl FromStr for SubnetKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "application" => Ok(Self::Application),
            "cloud_engine" => Ok(Self::CloudEngine),
            "system" => Ok(Self::System),
            "unknown" => Ok(Self::Unknown),
            other => Err(format!(
                "invalid value {other}; use application, cloud_engine, system, or unknown"
            )),
        }
    }
}
