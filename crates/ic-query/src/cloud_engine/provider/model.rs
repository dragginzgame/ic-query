//! Module: cloud_engine::provider::model
//!
//! Responsibility: CloudEngine provider requests, location rows, reports, and source data.
//! Does not own: source validation, HTTP decoding, rendering, or command parsing.
//! Boundary: preserves raw Dashboard provider and location fields without Registry promotion.

use crate::ic::IcDashboardReportProvenance;
use serde::{Deserialize, Serialize};

#[cfg(feature = "dashboard-host")]
use crate::ic::IcSourceRequest;

///
/// CloudEngineProviderListRequest
///
/// Request for one complete official Dashboard provider resource filtered to CloudEngine rows.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineProviderListRequest {
    /// Requested network identity; the built-in source accepts only `ic`.
    pub network: String,
    /// Official Dashboard v3 base endpoint.
    pub source_endpoint: String,
    /// Caller collection time as Unix seconds.
    pub now_unix_secs: u64,
}

impl CloudEngineProviderListRequest {
    /// Construct one live CloudEngine provider-list request.
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
        }
    }
}

///
/// CloudEngineProviderInfoRequest
///
/// Request for one exact official Dashboard node-provider record and its CloudEngine fields.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineProviderInfoRequest {
    /// Requested network identity; the built-in source accepts only `ic`.
    pub network: String,
    /// Official Dashboard v3 base endpoint.
    pub source_endpoint: String,
    /// Caller collection time as Unix seconds.
    pub now_unix_secs: u64,
    /// Exact node-provider principal.
    pub node_provider_id: String,
}

impl CloudEngineProviderInfoRequest {
    /// Construct one exact live provider request.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        node_provider_id: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            node_provider_id: node_provider_id.into(),
        }
    }
}

///
/// CloudEngineProviderLocation
///
/// Raw official Dashboard location attached to one node provider.
///

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CloudEngineProviderLocation {
    /// Dashboard data-center key.
    pub dc_key: String,
    /// Human-facing Dashboard location label.
    pub display_name: String,
    /// Raw decimal latitude decoded as a finite number.
    pub latitude: f64,
    /// Raw decimal longitude decoded as a finite number.
    pub longitude: f64,
    /// Dashboard data-center owner label.
    pub owner: String,
    /// Raw Dashboard region label.
    pub region: String,
}

///
/// CloudEngineProviderRow
///
/// One raw official Dashboard node-provider record with CloudEngine-specific evidence.
///

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CloudEngineProviderRow {
    /// Canonical node-provider principal.
    pub principal_id: String,
    /// Human-facing provider name.
    pub display_name: String,
    /// Raw optional website text.
    pub website: Option<String>,
    /// Raw optional Dashboard logo URL.
    pub logo_url: Option<String>,
    /// Number of all Dashboard locations represented by `locations`.
    pub location_count: usize,
    /// Dashboard locations associated with the provider's ordinary IC nodes.
    pub locations: Vec<CloudEngineProviderLocation>,
    /// Number of CloudEngine locations represented by `cloud_engine_locations`.
    pub cloud_engine_location_count: usize,
    /// Independent Dashboard locations carrying CloudEngine evidence.
    pub cloud_engine_locations: Vec<CloudEngineProviderLocation>,
    /// Raw Dashboard total for CloudEngine nodes.
    pub total_cloud_engine_nodes: u64,
    /// Raw Dashboard total for unassigned CloudEngine nodes.
    pub total_cloud_engine_unassigned_nodes: u64,
    /// Raw Dashboard total for CloudEngine instances.
    pub total_cloud_engines: u64,
    /// Raw Dashboard total node allowance.
    pub total_node_allowance: u64,
    /// Raw Dashboard total ordinary nodes.
    pub total_nodes: u64,
    /// Raw Dashboard total rewardable nodes.
    pub total_rewardable_nodes: u64,
    /// Raw Dashboard total Subnets.
    pub total_subnets: u64,
    /// Raw Dashboard total unassigned ordinary nodes.
    pub total_unassigned_nodes: u64,
}

impl CloudEngineProviderRow {
    /// Whether any returned CloudEngine-specific field carries nonempty evidence.
    #[must_use]
    pub const fn has_cloud_engine_evidence(&self) -> bool {
        self.cloud_engine_location_count > 0
            || !self.cloud_engine_locations.is_empty()
            || self.total_cloud_engine_nodes > 0
            || self.total_cloud_engine_unassigned_nodes > 0
            || self.total_cloud_engines > 0
    }
}

///
/// CloudEngineProviderListReport
///
/// Complete Dashboard provider resource filtered to rows with explicit CloudEngine evidence.
///

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CloudEngineProviderListReport {
    /// Shared official Dashboard provenance, flattened in report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Number of provider rows in the complete source resource before filtering.
    pub source_node_provider_count: usize,
    /// Number of rows with explicit CloudEngine evidence.
    pub cloud_engine_provider_count: usize,
    /// CloudEngine-bearing providers in canonical principal order.
    pub providers: Vec<CloudEngineProviderRow>,
}

///
/// CloudEngineProviderInfoReport
///
/// One exact Dashboard provider record with an explicit CloudEngine-scope classification.
///

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CloudEngineProviderInfoReport {
    /// Shared official Dashboard provenance, flattened in report JSON.
    #[serde(flatten)]
    pub provenance: IcDashboardReportProvenance,
    /// Whether the exact row contains any explicit CloudEngine evidence.
    pub cloud_engine_evidence_present: bool,
    /// Exact raw provider record.
    pub provider: CloudEngineProviderRow,
}

///
/// CloudEngineProviderListSourceData
///
/// Complete decoded official Dashboard provider resource returned by a source.
///

#[cfg(feature = "dashboard-host")]
#[derive(Clone, Debug, PartialEq)]
pub struct CloudEngineProviderListSourceData {
    /// Source request and collection provenance echoed by the source.
    pub source: IcSourceRequest,
    /// Every provider row returned by the complete resource before filtering.
    pub providers: Vec<CloudEngineProviderRow>,
}

///
/// CloudEngineProviderInfoSourceData
///
/// Exact decoded official Dashboard provider record returned by a source.
///

#[cfg(feature = "dashboard-host")]
#[derive(Clone, Debug, PartialEq)]
pub struct CloudEngineProviderInfoSourceData {
    /// Source request and collection provenance echoed by the source.
    pub source: IcSourceRequest,
    /// Exact provider row returned by the source.
    pub provider: CloudEngineProviderRow,
}
