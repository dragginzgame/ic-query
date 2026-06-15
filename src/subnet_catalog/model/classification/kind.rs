use serde::{Deserialize, Serialize};
use std::str::FromStr;

///
/// SubnetKind
///
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubnetKind {
    Application,
    CloudEngine,
    System,
    Unknown,
}

impl SubnetKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::CloudEngine => "cloud_engine",
            Self::System => "system",
            Self::Unknown => "unknown",
        }
    }

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
