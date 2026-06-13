use serde::{Deserialize, Serialize};

///
/// NnsTopologySummaryReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologySummaryReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub subnet_count: usize,
    pub application_subnet_count: usize,
    pub cloud_engine_subnet_count: usize,
    pub system_subnet_count: usize,
    pub unknown_subnet_count: usize,
    pub routing_range_count: usize,
    pub node_count: usize,
    pub application_node_count: usize,
    pub cloud_engine_node_count: usize,
    pub system_node_count: usize,
    pub unknown_node_count: usize,
    pub node_provider_count: usize,
    pub node_operator_count: usize,
    pub data_center_count: usize,
    pub nodes_with_known_node_provider_count: usize,
    pub nodes_with_unknown_node_provider_count: usize,
    pub nodes_with_known_node_operator_count: usize,
    pub nodes_with_unknown_node_operator_count: usize,
    pub nodes_with_known_data_center_count: usize,
    pub nodes_with_unknown_data_center_count: usize,
    pub node_operators_with_known_node_provider_count: usize,
    pub node_operators_with_unknown_node_provider_count: usize,
    pub node_operators_with_known_data_center_count: usize,
    pub node_operators_with_unknown_data_center_count: usize,
    pub subnet_catalog_stale: bool,
    pub subnet_catalog_stale_reason: String,
    pub registry_versions: Vec<NnsTopologyRegistryVersionRow>,
}

///
/// NnsTopologyCoverageReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCoverageReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub node_count: usize,
    pub node_provider_count: usize,
    pub node_operator_count: usize,
    pub data_center_count: usize,
    pub nodes_with_known_node_provider_count: usize,
    pub nodes_with_unknown_node_provider_count: usize,
    pub nodes_with_known_node_operator_count: usize,
    pub nodes_with_unknown_node_operator_count: usize,
    pub nodes_with_known_data_center_count: usize,
    pub nodes_with_unknown_data_center_count: usize,
    pub node_operators_with_known_node_provider_count: usize,
    pub node_operators_with_unknown_node_provider_count: usize,
    pub node_operators_with_known_data_center_count: usize,
    pub node_operators_with_unknown_data_center_count: usize,
}

///
/// NnsTopologyRegistryVersionRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRegistryVersionRow {
    pub source: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub stale: Option<bool>,
}

///
/// NnsTopologyVersionsReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyVersionsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub source_count: usize,
    pub registry_versions: Vec<NnsTopologyRegistryVersionRow>,
}

///
/// NnsTopologyHealthReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyHealthReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: String,
    pub registry_source_count: usize,
    pub registry_version_min: Option<u64>,
    pub registry_version_max: Option<u64>,
    pub registry_versions_aligned: bool,
    pub stale_source_count: usize,
    pub subnet_catalog_stale: bool,
    pub subnet_catalog_stale_reason: String,
    pub known_join_count: usize,
    pub unknown_join_count: usize,
    pub join_coverage: String,
    pub checks: Vec<NnsTopologyHealthCheckRow>,
}

///
/// NnsTopologyHealthCheckRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyHealthCheckRow {
    pub check: String,
    pub status: String,
    pub detail: String,
}

///
/// NnsTopologyGapsReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyGapsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: String,
    pub gap_count: usize,
    pub gaps: Vec<NnsTopologyGapRow>,
}

///
/// NnsTopologyGapRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyGapRow {
    pub subject_kind: String,
    pub subject: String,
    pub missing_relation: String,
    pub referenced_id: String,
}

///
/// NnsTopologyCapacityReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCapacityReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: String,
    pub node_operator_count: usize,
    pub total_node_allowance: u64,
    pub assigned_node_count: u64,
    pub unknown_node_count_operator_count: usize,
    pub available_node_slots: u64,
    pub over_assigned_operator_count: usize,
    pub over_assigned_node_count: u64,
    pub capacity: Vec<NnsTopologyCapacityRow>,
}

///
/// NnsTopologyCapacityRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyCapacityRow {
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub data_center_id: String,
    pub node_allowance: u64,
    pub assigned_node_count: Option<u64>,
    pub available_node_slots: Option<u64>,
    pub over_assigned_node_count: Option<u64>,
    pub utilization: String,
    pub status: String,
}

///
/// NnsTopologyRegionsReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRegionsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub region_count: usize,
    pub data_center_count: usize,
    pub node_operator_count: u64,
    pub node_provider_count: u64,
    pub node_count: u64,
    pub regions: Vec<NnsTopologyRegionRow>,
}

///
/// NnsTopologyRegionRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRegionRow {
    pub region: String,
    pub data_center_count: usize,
    pub node_operator_count: u64,
    pub node_provider_count: u64,
    pub node_count: u64,
}

///
/// NnsTopologyProvidersReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyProvidersReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub registered_node_provider_count: usize,
    pub referenced_node_provider_count: usize,
    pub provider_with_nodes_count: usize,
    pub provider_with_node_operators_count: usize,
    pub total_node_count: u64,
    pub total_node_operator_count: u64,
    pub total_node_allowance: u64,
    pub over_assigned_provider_count: usize,
    pub unknown_provider_count: usize,
    pub providers: Vec<NnsTopologyProviderRow>,
}

///
/// NnsTopologyProviderRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyProviderRow {
    pub node_provider_principal: String,
    pub registered: bool,
    pub name: Option<String>,
    pub governance_node_count: Option<u64>,
    pub topology_node_count: u64,
    pub node_operator_count: u64,
    pub data_center_count: usize,
    pub region_count: usize,
    pub total_node_allowance: u64,
    pub assigned_node_count: u64,
    pub available_node_slots: u64,
    pub over_assigned_node_count: u64,
    pub status: String,
}

///
/// NnsTopologyRefreshReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub dry_run: bool,
    pub component_count: usize,
    pub wrote_cache_count: usize,
    pub replaced_existing_cache_count: usize,
    pub components: Vec<NnsTopologyRefreshRow>,
}

///
/// NnsTopologyRefreshRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyRefreshRow {
    pub source: String,
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub dry_run: bool,
    pub wrote_cache: bool,
    pub replaced_existing_cache: bool,
    pub item_count: usize,
}
