//! Module: ic::api_boundary_node::model
//!
//! Responsibility: certified API boundary-node requests, evidence, reports, and host errors.
//! Does not own: state-tree collection, validation, command parsing, or text rendering.
//! Boundary: preserves authenticated node identities, domains, addresses, and certificate time.

#[cfg(feature = "ic-state-host")]
use crate::runtime::RuntimeError;
use serde::Serialize;
#[cfg(feature = "ic-state-host")]
use thiserror::Error as ThisError;

///
/// IcApiBoundaryNodeRequest
///
/// Caller request for the complete certified API boundary-node state-tree collection.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcApiBoundaryNodeRequest {
    /// Mainnet IC API endpoint used for the certified `read_state` request.
    pub source_endpoint: String,
    /// Caller observation time as Unix seconds.
    pub now_unix_secs: u64,
}

impl IcApiBoundaryNodeRequest {
    /// Construct one live certified API boundary-node request.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
        }
    }
}

///
/// IcCertifiedStateProvenance
///
/// Certificate and retrieval provenance for one authenticated IC state-tree report.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcCertifiedStateProvenance {
    /// Emitted report schema version.
    pub schema_version: u32,
    /// Network identity authenticated by the built-in mainnet root key.
    pub network: String,
    /// Stable authority label for certified IC state-tree evidence.
    pub authority: String,
    /// IC API endpoint that returned the certificate.
    pub source_endpoint: String,
    /// Canister principal used only to route the `read_state` request.
    pub effective_canister_id: String,
    /// Caller observation time as Unix seconds.
    pub fetched_at_unix_seconds: u64,
    /// Caller observation time formatted in UTC.
    pub fetched_at: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Raw certified state-tree time as Unix nanoseconds.
    pub certificate_time_unix_nanos: u64,
    /// Certified state-tree time rounded down to Unix seconds.
    pub certificate_time_unix_seconds: u64,
    /// Certified state-tree time formatted in UTC at second precision.
    pub certificate_time: String,
    /// Whether the report rows are authenticated by an IC certificate.
    pub certified: bool,
    /// Whether the returned rows belong to one certified state-tree time.
    pub point_in_time_guaranteed: bool,
}

///
/// IcApiBoundaryNodeRow
///
/// One API boundary node authenticated by the certified IC state tree.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcApiBoundaryNodeRow {
    /// Canonical API boundary-node principal.
    pub node_id: String,
    /// Certified DNS domain.
    pub domain: String,
    /// Optional certified IPv4 address.
    pub ipv4_address: Option<String>,
    /// Certified IPv6 address.
    pub ipv6_address: String,
}

///
/// IcApiBoundaryNodeReport
///
/// Complete API boundary-node collection from one authenticated IC state tree.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcApiBoundaryNodeReport {
    /// Certificate and retrieval provenance, flattened in serialized report JSON.
    #[serde(flatten)]
    pub provenance: IcCertifiedStateProvenance,
    /// Number of authenticated API boundary-node rows.
    pub node_count: usize,
    /// Rows in canonical node-principal order.
    pub rows: Vec<IcApiBoundaryNodeRow>,
}

///
/// IcApiBoundaryNodeSourceRequest
///
/// Exact state-tree request and collection provenance supplied to a host source.
///

#[cfg(feature = "ic-state-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcApiBoundaryNodeSourceRequest {
    /// Fixed mainnet network identity.
    pub network: String,
    /// IC API endpoint used for `read_state`.
    pub endpoint: String,
    /// Canister principal used only to route the request.
    pub effective_canister_id: String,
    /// Caller observation time as Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Caller observation time formatted in UTC.
    pub fetched_at: String,
    /// Collector identity.
    pub fetched_by: String,
}

///
/// IcApiBoundaryNodeSourceData
///
/// Authenticated state-tree evidence returned by an API boundary-node source.
///

#[cfg(feature = "ic-state-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcApiBoundaryNodeSourceData {
    /// Exact source request echoed by the source.
    pub source: IcApiBoundaryNodeSourceRequest,
    /// Raw certified state-tree time as Unix nanoseconds.
    pub certificate_time_unix_nanos: u64,
    /// Authenticated API boundary-node rows.
    pub rows: Vec<IcApiBoundaryNodeRow>,
}

///
/// IcApiBoundaryNodeHostError
///
/// Failure while collecting or projecting certified API boundary-node evidence.
///

#[cfg(feature = "ic-state-host")]
#[derive(Debug, ThisError)]
pub enum IcApiBoundaryNodeHostError {
    /// The synchronous adapter could not create its local async runtime.
    #[error("failed to run certified IC state query: {0}")]
    Runtime(#[from] RuntimeError),

    /// The response-bounded IC agent could not be constructed.
    #[error("failed to build IC state agent for {endpoint}: {reason}")]
    AgentBuild {
        /// Rejected endpoint.
        endpoint: String,
        /// Agent or endpoint diagnostic.
        reason: String,
    },

    /// The certified `read_state` request or certificate authentication failed.
    #[error("certified IC state read through {endpoint} failed: {reason}")]
    CertifiedReadState {
        /// Endpoint used for the request.
        endpoint: String,
        /// Transport, decoding, delegation, signature, or age diagnostic.
        reason: String,
    },

    /// The authenticated state tree did not match the specified boundary-node contract.
    #[error("invalid certified IC state-tree data: {reason}")]
    InvalidCertifiedState {
        /// Deterministic state-tree validation diagnostic.
        reason: String,
    },

    /// A source capability returned inconsistent request or report data.
    #[error("invalid API boundary-node source data: {reason}")]
    InvalidSourceData {
        /// Deterministic source-contract diagnostic.
        reason: String,
    },
}
