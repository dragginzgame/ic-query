use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListDeployedSnsesRequest {}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListDeployedSnsesResponse {
    pub(in crate::sns::report::live) instances: Vec<DeployedSns>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct DeployedSns {
    pub(in crate::sns::report::live) root_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) governance_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) ledger_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) swap_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) index_canister_id: Option<Principal>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetMetadataRequest {}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetMetadataResponse {
    pub(in crate::sns::report::live) url: Option<String>,
    pub(in crate::sns::report::live) logo: Option<String>,
    pub(in crate::sns::report::live) name: Option<String>,
    pub(in crate::sns::report::live) description: Option<String>,
}
