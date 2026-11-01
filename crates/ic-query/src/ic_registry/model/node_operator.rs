use crate::nns::node_operator::NnsNodeOperatorRow;
use serde::Serialize;

///
/// MainnetNodeOperatorList
///
/// Complete mainnet node operator inventory and registry metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeOperatorList {
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    pub node_operators: Vec<NnsNodeOperatorRow>,
}
