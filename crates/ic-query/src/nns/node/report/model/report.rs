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
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    pub nodes: Vec<NnsNodeRow>,
}

///
/// NnsNodeRow
///
/// One node row projected from the NNS registry inventory.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    /// Resolved inventory row.
    #[serde(flatten)]
    pub node: NnsNodeRow,
}
