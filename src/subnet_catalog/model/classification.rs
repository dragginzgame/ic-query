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

///
/// SubnetSpecialization
///
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubnetSpecialization {
    None,
    Fiduciary,
    European,
    Unknown,
}

impl SubnetSpecialization {
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

///
/// GeographicScope
///
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeographicScope {
    Global,
    Europe,
    Unknown,
}

impl GeographicScope {
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

///
/// ClassificationSource
///
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationSource {
    Registry,
    Curated,
    Computed,
    Unknown,
}

impl ClassificationSource {
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
