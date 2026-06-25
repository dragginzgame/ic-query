//! Module: sns::report::live::types::deployed
//!
//! Responsibility: SNS-W deployed-SNS and metadata Candid wire types.
//! Does not own: live transport, metadata conversion, or report rendering.
//! Boundary: mirrors SNS-W and SNS root metadata payloads used by fetchers.

use candid::{CandidType, Deserialize, Principal};

///
/// ListDeployedSnsesRequest
///
/// Candid request for SNS-W deployed SNS discovery.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListDeployedSnsesRequest {}

///
/// ListDeployedSnsesResponse
///
/// Candid response containing deployed SNS canister sets.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListDeployedSnsesResponse {
    pub(in crate::sns::report::live) instances: Vec<DeployedSns>,
}

///
/// DeployedSns
///
/// Candid SNS-W canister set for one deployed SNS.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct DeployedSns {
    pub(in crate::sns::report::live) root_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) governance_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) ledger_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) swap_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) index_canister_id: Option<Principal>,
}

///
/// GetMetadataRequest
///
/// Candid request for SNS root metadata.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetMetadataRequest {}

///
/// GetMetadataResponse
///
/// Candid response containing optional SNS root metadata.
///

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetMetadataResponse {
    pub(in crate::sns::report::live) url: Option<String>,
    pub(in crate::sns::report::live) logo: Option<String>,
    pub(in crate::sns::report::live) name: Option<String>,
    pub(in crate::sns::report::live) description: Option<String>,
}
