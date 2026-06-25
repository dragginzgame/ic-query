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
pub(in crate::sns::report) struct MainnetSnsList {
    pub(in crate::sns::report) network: String,
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) fetched_by: String,
    pub(in crate::sns::report) source_endpoint: String,
    pub(in crate::sns::report) sns_instances: Vec<MainnetSns>,
}

///
/// MainnetSns
///
/// Source-layer deployed SNS identity and optional metadata.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct MainnetSns {
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) name: String,
    pub(in crate::sns::report) description: Option<String>,
    pub(in crate::sns::report) url: Option<String>,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
    pub(in crate::sns::report) ledger_canister_id: String,
    pub(in crate::sns::report) swap_canister_id: String,
    pub(in crate::sns::report) index_canister_id: String,
    pub(in crate::sns::report) metadata_error: Option<String>,
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
