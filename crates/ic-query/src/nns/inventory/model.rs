//! Complete pinned Registry inventory shared by NNS views.

use crate::nns::{
    data_center::NnsDataCenterRow, node::NnsNodeRow, node_operator::NnsNodeOperatorRow,
    registry::NnsRegistryInventoryProvenance,
};
use serde::{Deserialize, Serialize};

///
/// NnsRegistryInventorySnapshot
///
/// Complete node, operator and referenced data-centre inventory at one Registry version.
///

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsRegistryInventorySnapshot {
    /// Current cache contract, always 1 before 1.0.
    pub schema_version: u32,
    /// Exact Registry acquisition shared by every view.
    #[serde(flatten)]
    pub provenance: NnsRegistryInventoryProvenance,
    /// Assigned nodes with their Registry relations.
    pub nodes: Vec<NnsNodeRow>,
    /// Operators referenced by the collected nodes.
    pub node_operators: Vec<NnsNodeOperatorRow>,
    /// Data centres referenced by the collected operators.
    pub data_centers: Vec<NnsDataCenterRow>,
}

impl crate::cache_file::JsonCacheReport for NnsRegistryInventorySnapshot {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }
    fn network(&self) -> &str {
        &self.provenance.network
    }
}
