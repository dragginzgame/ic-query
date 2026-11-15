//! Query-owned daily price reports and typed partial failures.

use crate::ReadError;
use ic_platform::{
    candid_codec::{self, decoder},
    cloud_engine_types::NodeType,
};
use ic_platform_types::Principal;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::time::Duration;
use thiserror::Error;

/// Bounded daily prices with separately observed network markup.
#[derive(Debug, Serialize)]
pub struct CloudEnginePricesReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// Local collection start, in Unix seconds.
    pub started_at_unix_seconds: u64,
    /// Monotonic duration of collection.
    pub elapsed: Duration,
    /// Both reads succeeded and passed field admission; not marketplace freshness.
    pub complete: bool,
    /// Always false: prices and fee are independent queries.
    pub atomic_snapshot: bool,
    /// Admitted declarations and source evidence.
    pub prices: PriceObservation,
    /// Independent markup observation or explicit failure.
    pub network_fee: FeeOutcome,
}

/// Admitted prices in lexicographic source-key order.
#[derive(Debug, Serialize)]
pub struct PriceObservation {
    /// Every admitted row; an empty list is not a zero-price claim.
    pub entries: Vec<MarketplacePrice>,
    /// Query source and assurance evidence.
    pub evidence: PriceEvidence,
}

/// One exact marketplace declaration, without currency or monthly conversion.
#[derive(Debug, Serialize)]
pub struct MarketplacePrice {
    /// Raw source identity, validated against class/provider/data center.
    pub key: String,
    /// Raw external category; not a hardware-performance or capacity measurement.
    pub node_type: NodeType,
    /// Optional provider scope; absence denotes a broader declaration.
    pub provider: Option<Principal>,
    /// Optional raw data-center scope, preserving present-empty distinctly.
    pub data_center: Option<String>,
    /// Exact canonical decimal whole cycles per node per day, at most 256 digits.
    pub net_cycles_per_node_per_day: String,
    /// Source-supplied gross amount; do not add the separately observed fee again.
    pub gross_cycles_per_node_per_day: String,
    /// Raw signed source update timestamp in Unix nanoseconds.
    pub updated_at_unix_nanoseconds: i64,
}

/// Evidence for one successful anonymous, signature-verified query.
#[derive(Debug, Serialize)]
pub struct PriceEvidence {
    /// Admitted mainnet endpoint.
    pub endpoint: &'static str,
    /// Exact engine canister principal.
    pub source_canister: &'static str,
    /// Exact selected method.
    pub source_method: &'static str,
    /// Local response-completion time in Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Agent verified the query signature.
    pub query_signature_verified: bool,
    /// False: query signatures do not certify result consensus.
    pub result_certified: bool,
}

/// Independent network-fee acquisition; failure never discards admitted prices.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FeeOutcome {
    /// Finite nonnegative markup on net; 0.25 means +25%, not 25% of gross.
    Read {
        /// Raw markup fraction; supplied gross prices already include their fee.
        markup: f64,
        /// Independent source and timing evidence.
        evidence: PriceEvidence,
    },
    /// The fee read or field admission failed; original cause remains in Rust.
    Failed {
        /// Typed failure; JSON projects category and admission facts.
        error: PriceError,
    },
    /// The shared 30-second deadline stopped or prevented the fee read.
    OverallDeadline,
}

/// Query-owned admission failures without interpreting raw source text.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum AdmissionError {
    /// A report collection or field exceeded its specified maximum.
    #[error("marketplace {field} exceeded limit {maximum}: {actual}")]
    Limit {
        /// Bound being checked.
        field: &'static str,
        /// Observed count or UTF-8 byte length.
        actual: usize,
        /// Admitted maximum.
        maximum: usize,
    },
    /// Source key differs from its class/provider/data-center identity.
    #[error("marketplace key disagrees with its price identity")]
    KeyMismatch,
    /// Multiple declarations identify the same source key.
    #[error("marketplace contains duplicate keys")]
    DuplicateKey,
    /// A cycle amount needs more than 256 decimal digits.
    #[error("marketplace cycle amount exceeds 256 decimal digits")]
    CycleDigits,
    /// Source gross amount is smaller than its net amount.
    #[error("marketplace gross price is below net")]
    GrossBelowNet,
    /// Markup must be finite and nonnegative before JSON projection.
    #[error("network markup is nonfinite or negative")]
    InvalidFee,
}

/// Source, codec and report failures with original typed causes retained.
#[derive(Debug, Error)]
pub enum PriceError {
    /// Agent query, leaf deadline or local clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// Bounded empty-request encoding failed.
    #[error("cloud-engine request encoding failed: {0:?}")]
    Encode(candid_codec::Error),
    /// Bounded external response decoding failed.
    #[error("cloud-engine response decoding failed: {0:?}")]
    Decode(decoder::Error),
    /// Decoded values failed Query-owned report admission.
    #[error(transparent)]
    Admission(#[from] AdmissionError),
    /// Shared collection budget expired before usable prices were acquired.
    #[error("cloud-engine collection exceeded its 30-second deadline")]
    OverallDeadline,
}

impl From<candid_codec::Error> for PriceError {
    fn from(error: candid_codec::Error) -> Self {
        Self::Encode(error)
    }
}

impl From<decoder::Error> for PriceError {
    fn from(error: decoder::Error) -> Self {
        Self::Decode(error)
    }
}

impl Serialize for PriceError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let category = match self {
            Self::Read(ReadError::Transport(_)) => "transport",
            Self::Read(ReadError::Deadline) => "read_deadline",
            Self::Read(ReadError::Clock(_)) => "clock",
            Self::Encode(_) => "encode",
            Self::Decode(_) => "decode",
            Self::Admission(_) => "admission",
            Self::OverallDeadline => "overall_deadline",
        };
        let mut state = serializer.serialize_struct("PriceError", 2)?;
        state.serialize_field("category", category)?;
        state.serialize_field(
            "admission",
            &match self {
                Self::Admission(error) => Some(error),
                _ => None,
            },
        )?;
        state.end()
    }
}
