//! Module: sns::report::live::types::proposals::request
//!
//! Responsibility: SNS governance proposal request and response wire types.
//! Does not own: transport execution, conversion, or text rendering.
//! Boundary: models Candid payloads sent to and received from governance.

use crate::sns::report::live::types::proposals::data::{
    GetProposalResult, SnsGovernanceProposalData, SnsProposalId,
};
use crate::sns::report::live::types::proposals::topic::SnsTopicSelector;
use candid::{CandidType, Deserialize};

///
/// GetProposalRequest
///
/// Candid request for a direct SNS governance proposal lookup.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetProposalRequest {
    pub(in crate::sns::report::live) proposal_id: Option<SnsProposalId>,
}

///
/// GetProposalResponse
///
/// Candid response returned by direct SNS governance proposal lookup.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetProposalResponse {
    pub(in crate::sns::report::live) result: Option<GetProposalResult>,
}

///
/// ListProposalsRequest
///
/// Candid request for bounded SNS governance proposal listings.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListProposalsRequest {
    pub(in crate::sns::report::live) include_reward_status: Vec<i32>,
    pub(in crate::sns::report::live) before_proposal: Option<SnsProposalId>,
    pub(in crate::sns::report::live) limit: u32,
    pub(in crate::sns::report::live) exclude_type: Vec<u64>,
    pub(in crate::sns::report::live) include_status: Vec<i32>,
    pub(in crate::sns::report::live) include_topics: Option<Vec<SnsTopicSelector>>,
}

///
/// ListProposalsResponse
///
/// Candid response containing bounded SNS governance proposal rows.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListProposalsResponse {
    pub(in crate::sns::report::live) proposals: Vec<SnsGovernanceProposalData>,
}
