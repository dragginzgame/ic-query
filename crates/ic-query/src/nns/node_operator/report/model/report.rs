use serde::{Deserialize, Serialize};

///
/// NnsNodeOperatorListReport
///
/// Complete NNS node operator inventory report with source metadata.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsNodeOperatorListReport {
    pub schema_version: u32,
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    pub node_operators: Vec<NnsNodeOperatorRow>,
}

#[cfg(feature = "nns-host")]
impl_nns_inventory_report!(
    NnsNodeOperatorListReport,
    super::super::NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
    "node_operator",
    super::super::COLLECTION,
    node_operators,
    |_report: &NnsNodeOperatorListReport| None,
);

///
/// NnsNodeOperatorRow
///
/// One node operator row projected from the NNS registry inventory.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeOperatorRow {
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub node_allowance: u64,
    pub data_center_id: String,
    pub node_count: Option<u32>,
}

///
/// NnsNodeOperatorInfoReport
///
/// Detailed report for one resolved NNS node operator.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeOperatorInfoReport {
    pub schema_version: u32,
    pub input: String,
    pub resolved_from: String,
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    /// Resolved inventory row.
    #[serde(flatten)]
    pub node_operator: NnsNodeOperatorRow,
}
