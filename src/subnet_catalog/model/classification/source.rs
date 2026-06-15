use serde::{Deserialize, Serialize};

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
