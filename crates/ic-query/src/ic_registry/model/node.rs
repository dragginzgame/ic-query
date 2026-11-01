use crate::nns::node::NnsNodeRow;
use serde::Serialize;

///
/// MainnetNodeList
///
/// Complete mainnet node inventory and registry metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeList {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub nodes: Vec<NnsNodeRow>,
}
