use crate::nns::node::NnsNodeRow;
use serde::Serialize;

///
/// MainnetNodeList
///
/// Complete mainnet node inventory and registry metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeList {
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    pub nodes: Vec<NnsNodeRow>,
}
