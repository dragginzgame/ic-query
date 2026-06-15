use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
