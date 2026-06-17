//! Module: sns::report::live::types::proposals::data
//!
//! Responsibility: SNS governance proposal result and data wire types.
//! Does not own: request construction, live transport, or report rendering.
//! Boundary: mirrors Candid fields converted into SNS proposal report rows.

use crate::sns::report::source::SnsNeuronId;
use candid::{CandidType, Deserialize};

///
/// GetProposalResult
///
/// Candid result variant returned by direct SNS proposal lookup.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum GetProposalResult {
    Error(SnsGovernanceError),
    Proposal(Box<SnsGovernanceProposalData>),
}

///
/// SnsProposalId
///
/// Candid SNS governance proposal identifier.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsProposalId {
    pub(in crate::sns::report::live) id: u64,
}

///
/// SnsGovernanceProposal
///
/// Candid SNS governance proposal metadata embedded in proposal data.
///

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceProposal {
    pub(in crate::sns::report::live) title: String,
    pub(in crate::sns::report::live) summary: String,
    pub(in crate::sns::report::live) url: String,
}

///
/// SnsGovernanceError
///
/// Candid SNS governance error embedded in proposal responses.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceError {
    pub(in crate::sns::report::live) error_type: i32,
    pub(in crate::sns::report::live) error_message: String,
}

///
/// SnsGovernanceBallot
///
/// Candid SNS governance ballot row embedded in proposal data.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceBallot {
    pub(in crate::sns::report::live) vote: i32,
    pub(in crate::sns::report::live) cast_timestamp_seconds: u64,
    pub(in crate::sns::report::live) voting_power: u64,
}

///
/// SnsGovernanceProposalTally
///
/// Candid SNS governance proposal tally embedded in proposal data.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceProposalTally {
    pub(in crate::sns::report::live) timestamp_seconds: u64,
    pub(in crate::sns::report::live) yes: u64,
    pub(in crate::sns::report::live) no: u64,
    pub(in crate::sns::report::live) total: u64,
}

///
/// SnsGovernanceProposalData
///
/// Candid SNS governance proposal row converted into report data.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceProposalData {
    pub(in crate::sns::report::live) id: Option<SnsProposalId>,
    pub(in crate::sns::report::live) payload_text_rendering: Option<String>,
    pub(in crate::sns::report::live) action: u64,
    pub(in crate::sns::report::live) failure_reason: Option<SnsGovernanceError>,
    pub(in crate::sns::report::live) ballots: Vec<(String, SnsGovernanceBallot)>,
    pub(in crate::sns::report::live) reward_event_round: u64,
    pub(in crate::sns::report::live) failed_timestamp_seconds: u64,
    pub(in crate::sns::report::live) reward_event_end_timestamp_seconds: Option<u64>,
    pub(in crate::sns::report::live) proposal_creation_timestamp_seconds: u64,
    pub(in crate::sns::report::live) reject_cost_e8s: u64,
    pub(in crate::sns::report::live) latest_tally: Option<SnsGovernanceProposalTally>,
    pub(in crate::sns::report::live) decided_timestamp_seconds: u64,
    pub(in crate::sns::report::live) proposal: Option<SnsGovernanceProposal>,
    pub(in crate::sns::report::live) proposer: Option<SnsNeuronId>,
    pub(in crate::sns::report::live) is_eligible_for_rewards: bool,
    pub(in crate::sns::report::live) executed_timestamp_seconds: u64,
}
