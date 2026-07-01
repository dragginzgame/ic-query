//! Module: sns::report::source::model::list
//!
//! Responsibility: source-layer deployed SNS list models.
//! Does not own: SNS-W transport, metadata conversion, or report rendering.
//! Boundary: carries resolved mainnet SNS identity data into builders.

///
/// MainnetSnsList
///
/// Source-layer deployed SNS list fetched from mainnet SNS-W.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsList {
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub sns_instances: Vec<MainnetSns>,
}

///
/// MainnetSns
///
/// Source-layer deployed SNS identity and optional metadata.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSns {
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}

///
/// MainnetSnsCanisters
///
/// Source-layer canister ids for one deployed SNS.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSnsCanisters {
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
    pub(in crate::sns::report) ledger_canister_id: String,
    pub(in crate::sns::report) swap_canister_id: String,
    pub(in crate::sns::report) index_canister_id: String,
}
