use serde::{Deserialize, Serialize};

///
/// NnsTopologyAssessmentStatus
///
/// Overall result of a binary NNS topology report or consistency check.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsTopologyAssessmentStatus {
    /// Every assessed invariant passed.
    Ok,
    /// At least one assessed invariant needs attention.
    Attention,
}

impl NnsTopologyAssessmentStatus {
    #[cfg(feature = "nns-host")]
    pub(crate) const fn from_ok(is_ok: bool) -> Self {
        if is_ok { Self::Ok } else { Self::Attention }
    }

    /// Return the stable serialized and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Attention => "attention",
        }
    }
}

///
/// NnsTopologyCapacityStatus
///
/// Assignment state for one NNS node operator's Registry allowance.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsTopologyCapacityStatus {
    /// Assigned node count exceeds the operator allowance.
    Over,
    /// Assigned node count is unavailable.
    Unknown,
    /// Assigned node count exactly consumes the operator allowance.
    Full,
    /// At least one node slot remains available.
    Available,
}

impl NnsTopologyCapacityStatus {
    /// Return the stable serialized and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Over => "over",
            Self::Unknown => "unknown",
            Self::Full => "full",
            Self::Available => "available",
        }
    }

    #[cfg(feature = "nns-host")]
    pub(crate) const fn sort_rank(self) -> u8 {
        match self {
            Self::Over => 0,
            Self::Unknown => 1,
            Self::Full => 2,
            Self::Available => 3,
        }
    }
}

///
/// NnsTopologyProviderStatus
///
/// Registry registration and capacity state for one NNS node provider.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsTopologyProviderStatus {
    /// The provider is referenced by topology records but absent from Governance.
    UnknownProvider,
    /// At least one provider node operator exceeds its Registry allowance.
    Over,
    /// The registered provider has no nodes or node operators.
    Unused,
    /// The registered provider has topology activity without over-assignment.
    Ok,
}

impl NnsTopologyProviderStatus {
    /// Return the stable serialized and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownProvider => "unknown_provider",
            Self::Over => "over",
            Self::Unused => "unused",
            Self::Ok => "ok",
        }
    }

    #[cfg(feature = "nns-host")]
    pub(crate) const fn sort_rank(self) -> u8 {
        match self {
            Self::UnknownProvider => 0,
            Self::Over => 1,
            Self::Unused => 2,
            Self::Ok => 3,
        }
    }
}

#[cfg(test)]
mod tests;
