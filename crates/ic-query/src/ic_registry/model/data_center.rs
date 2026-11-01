use crate::nns::data_center::NnsDataCenterRow;
use serde::Serialize;

///
/// MainnetDataCenterList
///
/// Complete mainnet data center inventory and registry metadata.
///

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MainnetDataCenterList {
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    pub data_centers: Vec<NnsDataCenterRow>,
}
