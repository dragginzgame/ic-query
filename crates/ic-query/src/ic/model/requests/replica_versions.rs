//! Module: ic::model::requests::replica_versions
//!
//! Responsibility: bounded official Dashboard replica-version request contracts.
//! Does not own: transport, returned release records, validation, or rendering.
//! Boundary: captures one explicit page or one exact replica-version target.

use super::IcDashboardRequest;

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
    /// Explicit Dashboard endpoint and caller collection clock.
    pub dashboard: IcDashboardRequest,
    /// Explicit page bounds.
    pub query: IcReplicaVersionListQuery,
}

impl IcReplicaVersionListRequest {
    /// Construct one bounded live Dashboard replica-version list request.
    #[must_use]
    pub const fn new(dashboard: IcDashboardRequest, query: IcReplicaVersionListQuery) -> Self {
        Self { dashboard, query }
    }
}

///
/// IcReplicaVersionInfoRequest
///
/// Request accepted by the exact official Dashboard replica-version builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcReplicaVersionInfoRequest {
    /// Explicit Dashboard endpoint and caller collection clock.
    pub dashboard: IcDashboardRequest,
    /// Exact lowercase hexadecimal replica-version identifier.
    pub replica_version_id: String,
}

impl IcReplicaVersionInfoRequest {
    /// Construct one exact live Dashboard replica-version request.
    #[must_use]
    pub fn new(dashboard: IcDashboardRequest, replica_version_id: impl Into<String>) -> Self {
        Self {
            dashboard,
            replica_version_id: replica_version_id.into(),
        }
    }
}
