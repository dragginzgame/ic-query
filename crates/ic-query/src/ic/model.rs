//! Module: ic::model
//!
//! Responsibility: public IC Dashboard canister requests, source data, reports, and errors.
//! Does not own: HTTP transport, source validation, report assembly, or rendering.
//! Boundary: preserves raw Dashboard values and explicit off-chain provenance.

#[cfg(feature = "host")]
use crate::runtime::RuntimeError;
use serde::Serialize;
#[cfg(feature = "host")]
use thiserror::Error as ThisError;

///
/// IcCanisterRequest
///
/// Request accepted by the official Dashboard canister report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterRequest {
    /// Dashboard API base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Canister principal to inspect.
    pub canister_id: String,
}

impl IcCanisterRequest {
    /// Construct a live Dashboard canister request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        canister_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            canister_id: canister_id.into(),
        }
    }
}

///
/// IcCanisterUpgrade
///
/// One proposal-linked canister upgrade recorded by the Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCanisterUpgrade {
    /// Proposal execution time as raw Unix seconds.
    pub executed_timestamp_seconds: u64,
    /// Wasm module hash as raw lowercase hexadecimal text.
    pub module_hash: String,
    /// NNS proposal that installed this module.
    pub proposal_id: u64,
}

///
/// IcCanisterReport
///
/// One live canister metadata report from the official Dashboard API.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCanisterReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Network represented by the official Dashboard API.
    pub network: String,
    /// Authority that supplied the report fields.
    pub authority: String,
    /// Dashboard API base endpoint queried by the source.
    pub source_endpoint: String,
    /// Time this report was collected.
    pub fetched_at: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Whether the API response is cryptographically certified IC state.
    pub certified: bool,
    /// Whether every returned value is guaranteed to describe one point in time.
    pub point_in_time_guaranteed: bool,
    /// Canonical canister principal.
    pub canister_id: String,
    /// Dashboard database row identifier.
    pub dashboard_id: u64,
    /// Raw optional Dashboard canister classification.
    pub canister_type: Option<String>,
    /// Raw Dashboard canister name; an empty string means no name was recorded.
    pub name: String,
    /// Canonical Subnet principal recorded by the Dashboard.
    pub subnet_id: String,
    /// Canonically ordered controller principals recorded by the Dashboard.
    pub controllers: Vec<String>,
    /// Raw Dashboard language label; an empty string means no language was recorded.
    pub language: String,
    /// Raw current module hash; an empty string means no hash was recorded.
    pub module_hash: String,
    /// Raw Dashboard row update timestamp.
    pub dashboard_updated_at: String,
    /// Number of proposal-linked upgrades when history is available.
    pub upgrade_count: Option<usize>,
    /// Proposal-linked upgrade history, or `None` when the Dashboard returned `null`.
    pub upgrades: Option<Vec<IcCanisterUpgrade>>,
}

///
/// IcSourceRequest
///
/// Shared endpoint and collection provenance for IC Dashboard source calls.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcSourceRequest {
    /// Dashboard API base endpoint.
    pub endpoint: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// Collector identity recorded in report provenance.
    pub fetched_by: String,
}

#[cfg(feature = "host")]
impl IcSourceRequest {
    /// Construct source-call provenance.
    #[must_use]
    pub fn new(
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

///
/// IcCanisterSourceData
///
/// Raw canister metadata and provenance returned by an IC Dashboard source.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterSourceData {
    /// Dashboard API base endpoint used by the source.
    pub source_endpoint: String,
    /// Collection timestamp supplied in the source request.
    pub fetched_at: String,
    /// Collector identity supplied in the source request.
    pub fetched_by: String,
    /// Canister principal returned by the Dashboard.
    pub canister_id: String,
    /// Dashboard database row identifier.
    pub dashboard_id: u64,
    /// Raw optional Dashboard canister classification.
    pub canister_type: Option<String>,
    /// Raw Dashboard canister name.
    pub name: String,
    /// Subnet principal returned by the Dashboard.
    pub subnet_id: String,
    /// Controller principals returned by the Dashboard.
    pub controllers: Vec<String>,
    /// Raw Dashboard language label.
    pub language: String,
    /// Raw current module hash.
    pub module_hash: String,
    /// Raw Dashboard row update timestamp.
    pub dashboard_updated_at: String,
    /// Proposal-linked upgrades, or `None` when the Dashboard returned `null`.
    pub upgrades: Option<Vec<IcCanisterUpgrade>>,
}

///
/// IcHostError
///
/// Typed error returned by IC Dashboard report builders and live sources.
///

#[cfg(feature = "host")]
#[derive(Debug, ThisError)]
pub enum IcHostError {
    /// The synchronous adapter could not create its local async runtime.
    #[error("failed to run IC Dashboard query: {0}")]
    Runtime(#[from] RuntimeError),

    /// A request supplied an invalid canister principal.
    #[error("invalid {field}: {reason}")]
    InvalidPrincipal {
        /// Principal field being validated.
        field: &'static str,
        /// Principal parser diagnostic.
        reason: String,
    },

    /// The Dashboard API base endpoint is malformed or unsupported.
    #[error("invalid IC Dashboard endpoint {endpoint}: {reason}")]
    InvalidEndpoint {
        /// Rejected endpoint.
        endpoint: String,
        /// URL validation diagnostic.
        reason: String,
    },

    /// The HTTP client could not be constructed.
    #[error("failed to build IC Dashboard HTTP client: {reason}")]
    HttpClientBuild {
        /// HTTP client construction diagnostic.
        reason: String,
    },

    /// The live Dashboard request failed before a response was received.
    #[error("IC Dashboard request to {url} failed: {reason}")]
    HttpRequest {
        /// Fully resolved request URL.
        url: String,
        /// HTTP transport error.
        reason: String,
    },

    /// The Dashboard returned a non-success HTTP status.
    #[error("IC Dashboard request to {url} returned HTTP status {status}")]
    HttpStatus {
        /// Fully resolved request URL.
        url: String,
        /// Numeric HTTP status.
        status: u16,
    },

    /// The Dashboard response did not match the expected JSON shape.
    #[error("failed to decode IC Dashboard response from {url}: {reason}")]
    JsonDecode {
        /// Fully resolved request URL.
        url: String,
        /// JSON response decoding error.
        reason: String,
    },

    /// A source capability returned data that violates its public result contract.
    #[error("invalid IC Dashboard canister source data: {reason}")]
    InvalidSourceData {
        /// Deterministic invariant failure.
        reason: String,
    },
}
