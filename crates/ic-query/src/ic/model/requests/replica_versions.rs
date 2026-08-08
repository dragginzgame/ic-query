//! Module: ic::model::requests::replica_versions
//!
//! Responsibility: bounded official Dashboard replica-version request contracts.
//! Does not own: transport, returned release records, validation, or rendering.
//! Boundary: captures one explicit page or one exact replica-version target.

use serde::Serialize;

///
/// IcReplicaVersionListQuery
///
/// One explicitly bounded official Dashboard replica-version page query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcReplicaVersionListQuery {
    /// Maximum release rows requested from the API.
    pub limit: u16,
    /// Zero-based release-row offset.
    pub offset: u64,
    /// Optional API proposal-index ceiling returned by an earlier page.
    pub max_proposal_index: Option<u64>,
}

impl IcReplicaVersionListQuery {
    /// Construct one bounded replica-version page query.
    #[must_use]
    pub const fn new(limit: u16, offset: u64, max_proposal_index: Option<u64>) -> Self {
        Self {
            limit,
            offset,
            max_proposal_index,
        }
    }
}

///
/// IcReplicaVersionListRequest
///
/// Request accepted by the bounded official Dashboard replica-version list builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcReplicaVersionListRequest {
    /// Dashboard API v3 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Explicit page bounds.
    pub query: IcReplicaVersionListQuery,
}

impl IcReplicaVersionListRequest {
    /// Construct one bounded live Dashboard replica-version list request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        query: IcReplicaVersionListQuery,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            query,
        }
    }
}

///
/// IcReplicaVersionInfoRequest
///
/// Request accepted by the exact official Dashboard replica-version builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcReplicaVersionInfoRequest {
    /// Dashboard API v3 base endpoint.
    pub source_endpoint: String,
    /// Collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Exact lowercase hexadecimal replica-version identifier.
    pub replica_version_id: String,
}

impl IcReplicaVersionInfoRequest {
    /// Construct one exact live Dashboard replica-version request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        replica_version_id: impl Into<String>,
    ) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            replica_version_id: replica_version_id.into(),
        }
    }
}
