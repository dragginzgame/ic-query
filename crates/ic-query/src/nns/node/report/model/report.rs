use crate::subnet_catalog::SubnetKind;
use serde::{Deserialize, Serialize};

///
/// NnsNodeListReport
///
/// Filtered NNS node inventory report with source metadata.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsNodeListReport {
    pub schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_count: usize,
    pub nodes: Vec<NnsNodeRow>,
}

#[cfg(feature = "nns-host")]
impl_nns_inventory_report!(
    NnsNodeListReport,
    super::super::NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
    "node",
    node_count,
    nodes,
    |_report: &NnsNodeListReport| None,
);

///
/// NnsNodeRow
///
/// One node row projected from the NNS registry inventory.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeRow {
    pub node_principal: String,
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub subnet_principal: String,
    pub subnet_kind: SubnetKind,
    pub data_center_id: String,
}

///
/// NnsNodeInfoReport
///
/// Detailed report for one resolved NNS node.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeInfoReport {
    pub schema_version: u32,
    pub input: String,
    pub resolved_from: String,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_principal: String,
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub subnet_principal: String,
    pub subnet_kind: SubnetKind,
    pub data_center_id: String,
}
