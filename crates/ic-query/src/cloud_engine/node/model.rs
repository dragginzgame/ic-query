//! Module: cloud_engine::node::model
//!
//! Responsibility: CloudEngine Type4 node request, report, and source-data contracts.
//! Does not own: source validation, HTTP transport, rendering, or command parsing.
//! Boundary: reuses the canonical raw Dashboard node row without changing its authority.

use crate::ic::{IcDashboardReportProvenance, IcNodeStatusCounts};
use serde::Serialize;

#[cfg(feature = "dashboard-host")]
use crate::ic::IcSourceRequest;

///
/// CloudEngineNodeRow
///
/// Raw official Dashboard node observation selected by the Type4 reward-type scope.
///

pub type CloudEngineNodeRow = crate::ic::IcNodeStatusRow;

///
/// CloudEngineNodeListRequest
///
/// Request for one complete official Dashboard Type4 node resource.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineNodeListRequest {
    /// Requested network identity; the built-in source accepts only `ic`.
    pub network: String,
    /// Official Dashboard v3 base endpoint.
    pub source_endpoint: String,
    /// Caller collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Optional exact node-provider principal applied by the remote resource.
    pub node_provider_id: Option<String>,
}

impl CloudEngineNodeListRequest {
    /// Construct one complete live Type4 node-list request.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            node_provider_id: None,
        }
    }

    /// Restrict collection to one exact node-provider principal.
    #[must_use]
    pub fn with_node_provider_id(mut self, node_provider_id: impl Into<String>) -> Self {
        self.node_provider_id = Some(node_provider_id.into());
        self
    }
}

///
/// CloudEngineNodeInfoRequest
///
/// Request for one exact official Dashboard node required to be Type4.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineNodeInfoRequest {
    /// Requested network identity; the built-in source accepts only `ic`.
    pub network: String,
    /// Official Dashboard v3 base endpoint.
    pub source_endpoint: String,
    /// Caller collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Exact node principal.
    pub node_id: String,
}

impl CloudEngineNodeInfoRequest {
    /// Construct one exact live Type4 node request.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        node_id: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            node_id: node_id.into(),
        }
    }
}

///
/// CloudEngineNodeListReport
///
/// Complete bounded Dashboard Type4 node scope, optionally restricted to one provider.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CloudEngineNodeListReport {
    /// Shared official Dashboard provenance, flattened in report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Exact raw reward type selected by the request.
    pub node_reward_type: String,
    /// Complete current Dashboard status values explicitly selected by the request.
    pub included_statuses: Vec<String>,
    /// Canonical provider filter, or `None` for the complete Type4 network scope.
    pub requested_node_provider_id: Option<String>,
    /// Number of returned Type4 node rows.
    pub node_count: usize,
    /// Raw operational-status totals across returned rows.
    pub status_counts: IcNodeStatusCounts,
    /// Distinct provider principals represented by returned rows.
    pub node_provider_count: usize,
    /// Distinct non-null CloudEngine Subnet principals represented by returned rows.
    pub cloud_engine_subnet_count: usize,
    /// Rows without a current `cloud_engine_subnet_id` observation.
    pub unassigned_cloud_engine_node_count: usize,
    /// Canonically ordered raw Type4 node observations.
    pub nodes: Vec<CloudEngineNodeRow>,
}

///
/// CloudEngineNodeInfoReport
///
/// One exact official Dashboard node observation validated as Type4.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CloudEngineNodeInfoReport {
    /// Shared official Dashboard provenance, flattened in report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Exact raw Type4 node observation.
    pub node: CloudEngineNodeRow,
}

///
/// CloudEngineNodeListSourceData
///
/// Untrusted complete Type4 node resource and query echo returned by a source.
///

#[cfg(feature = "dashboard-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineNodeListSourceData {
    /// Source request and collection provenance echoed by the source.
    pub source: IcSourceRequest,
    /// Canonical provider filter echoed by the source.
    pub requested_node_provider_id: Option<String>,
    /// Raw reward-type filter echoed by the source.
    pub node_reward_type: String,
    /// Explicit status filters echoed in request order.
    pub included_statuses: Vec<String>,
    /// Raw node rows returned by the source.
    pub nodes: Vec<CloudEngineNodeRow>,
}

///
/// CloudEngineNodeInfoSourceData
///
/// Untrusted exact Dashboard node observation returned by a source.
///

#[cfg(feature = "dashboard-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineNodeInfoSourceData {
    /// Source request and collection provenance echoed by the source.
    pub source: IcSourceRequest,
    /// Exact canonical node target echoed by the source.
    pub node_id: String,
    /// Raw exact node observation.
    pub node: CloudEngineNodeRow,
}
