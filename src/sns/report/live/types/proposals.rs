use super::super::SnsNeuronId;
use candid::{CandidType, Deserialize};

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetProposalRequest {
    pub(in crate::sns::report::live) proposal_id: Option<SnsProposalId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetProposalResponse {
    pub(in crate::sns::report::live) result: Option<GetProposalResult>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum GetProposalResult {
    Error(SnsGovernanceError),
    Proposal(Box<SnsGovernanceProposalData>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListProposalsRequest {
    pub(in crate::sns::report::live) include_reward_status: Vec<i32>,
    pub(in crate::sns::report::live) before_proposal: Option<SnsProposalId>,
    pub(in crate::sns::report::live) limit: u32,
    pub(in crate::sns::report::live) exclude_type: Vec<u64>,
    pub(in crate::sns::report::live) include_status: Vec<i32>,
    pub(in crate::sns::report::live) include_topics: Option<Vec<SnsTopicSelector>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsTopicSelector {
    pub(in crate::sns::report::live) topic: Option<SnsTopic>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum SnsTopic {
    DaoCommunitySettings,
    SnsFrameworkManagement,
    DappCanisterManagement,
    ApplicationBusinessLogic,
    Governance,
    TreasuryAssetManagement,
    CriticalDappOperations,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListProposalsResponse {
    pub(in crate::sns::report::live) proposals: Vec<SnsGovernanceProposalData>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsProposalId {
    pub(in crate::sns::report::live) id: u64,
}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceProposal {
    pub(in crate::sns::report::live) title: String,
    pub(in crate::sns::report::live) summary: String,
    pub(in crate::sns::report::live) url: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceError {
    pub(in crate::sns::report::live) error_type: i32,
    pub(in crate::sns::report::live) error_message: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceBallot {
    pub(in crate::sns::report::live) vote: i32,
    pub(in crate::sns::report::live) cast_timestamp_seconds: u64,
    pub(in crate::sns::report::live) voting_power: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsGovernanceProposalTally {
    pub(in crate::sns::report::live) timestamp_seconds: u64,
    pub(in crate::sns::report::live) yes: u64,
    pub(in crate::sns::report::live) no: u64,
    pub(in crate::sns::report::live) total: u64,
}

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
