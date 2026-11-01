use crate::nns::data_center::NnsDataCenterRow;
use serde::Serialize;

///
/// MainnetDataCenterList
///
/// Complete mainnet data center inventory and registry metadata.
///

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MainnetDataCenterList {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub data_centers: Vec<NnsDataCenterRow>,
}
