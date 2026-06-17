//! Module: subnet_catalog::model::classification::specialization
//!
//! Responsibility: define stable subnet specialization values for catalog data.
//!
//! Does not own: specialization derivation, registry fetching, or report filtering.
//!
//! Boundary: keeps serialized specialization values aligned with CLI filter syntax.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Subnet specialization classification used by catalog records and reports.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubnetSpecialization {
    None,
    Fiduciary,
    European,
    Unknown,
}

impl SubnetSpecialization {
    /// Returns the stable snake_case value used in CLI filters and text output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Fiduciary => "fiduciary",
            Self::European => "european",
            Self::Unknown => "unknown",
        }
    }
}

impl FromStr for SubnetSpecialization {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "none" => Ok(Self::None),
            "fiduciary" => Ok(Self::Fiduciary),
            "european" => Ok(Self::European),
            "unknown" => Ok(Self::Unknown),
            other => Err(format!(
                "invalid value {other}; use none, fiduciary, european, or unknown"
            )),
        }
    }
}
