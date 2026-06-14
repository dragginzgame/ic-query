use super::SnsNeuronId;
use candid::{CandidType, Deserialize, Int, Nat, Principal};

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ListDeployedSnsesRequest {}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ListDeployedSnsesResponse {
    pub(super) instances: Vec<DeployedSns>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct DeployedSns {
    pub(super) root_canister_id: Option<Principal>,
    pub(super) governance_canister_id: Option<Principal>,
    pub(super) ledger_canister_id: Option<Principal>,
    pub(super) swap_canister_id: Option<Principal>,
    pub(super) index_canister_id: Option<Principal>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct GetMetadataRequest {}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub(super) struct GetMetadataResponse {
    pub(super) url: Option<String>,
    pub(super) logo: Option<String>,
    pub(super) name: Option<String>,
    pub(super) description: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct IcrcAccount {
    pub(super) owner: Principal,
    pub(super) subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report) enum IcrcMetadataValue {
    Nat(Nat),
    Int(Int),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct IcrcSupportedStandard {
    pub(super) name: String,
    pub(super) url: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) enum GetIndexPrincipalResult {
    Ok(Principal),
    Err(GetIndexPrincipalError),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) enum GetIndexPrincipalError {
    IndexPrincipalNotSet,
    GenericError {
        error_code: Nat,
        description: String,
    },
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ListNeuronsRequest {
    pub(super) of_principal: Option<Principal>,
    pub(super) limit: u32,
    pub(super) start_page_at: Option<SnsNeuronId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ListNeuronsResponse {
    pub(super) neurons: Vec<SnsGovernanceNeuron>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct GetProposalRequest {
    pub(super) proposal_id: Option<SnsProposalId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct GetProposalResponse {
    pub(super) result: Option<GetProposalResult>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) enum GetProposalResult {
    Error(SnsGovernanceError),
    Proposal(Box<SnsGovernanceProposalData>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ListProposalsRequest {
    pub(super) include_reward_status: Vec<i32>,
    pub(super) before_proposal: Option<SnsProposalId>,
    pub(super) limit: u32,
    pub(super) exclude_type: Vec<u64>,
    pub(super) include_status: Vec<i32>,
    pub(super) include_topics: Option<Vec<SnsTopicSelector>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SnsTopicSelector {
    pub(super) topic: Option<SnsTopic>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) enum SnsTopic {
    DaoCommunitySettings,
    SnsFrameworkManagement,
    DappCanisterManagement,
    ApplicationBusinessLogic,
    Governance,
    TreasuryAssetManagement,
    CriticalDappOperations,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ListProposalsResponse {
    pub(super) proposals: Vec<SnsGovernanceProposalData>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SnsProposalId {
    pub(super) id: u64,
}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub(super) struct SnsGovernanceProposal {
    pub(super) title: String,
    pub(super) summary: String,
    pub(super) url: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SnsGovernanceError {
    pub(super) error_type: i32,
    pub(super) error_message: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SnsGovernanceBallot {
    pub(super) vote: i32,
    pub(super) cast_timestamp_seconds: u64,
    pub(super) voting_power: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SnsGovernanceProposalTally {
    pub(super) timestamp_seconds: u64,
    pub(super) yes: u64,
    pub(super) no: u64,
    pub(super) total: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SnsGovernanceProposalData {
    pub(super) id: Option<SnsProposalId>,
    pub(super) payload_text_rendering: Option<String>,
    pub(super) action: u64,
    pub(super) failure_reason: Option<SnsGovernanceError>,
    pub(super) ballots: Vec<(String, SnsGovernanceBallot)>,
    pub(super) reward_event_round: u64,
    pub(super) failed_timestamp_seconds: u64,
    pub(super) reward_event_end_timestamp_seconds: Option<u64>,
    pub(super) proposal_creation_timestamp_seconds: u64,
    pub(super) reject_cost_e8s: u64,
    pub(super) latest_tally: Option<SnsGovernanceProposalTally>,
    pub(super) decided_timestamp_seconds: u64,
    pub(super) proposal: Option<SnsGovernanceProposal>,
    pub(super) proposer: Option<SnsNeuronId>,
    pub(super) is_eligible_for_rewards: bool,
    pub(super) executed_timestamp_seconds: u64,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SnsGovernanceNeuron {
    pub(super) id: Option<SnsNeuronId>,
    pub(super) staked_maturity_e8s_equivalent: Option<u64>,
    pub(super) maturity_e8s_equivalent: u64,
    pub(super) cached_neuron_stake_e8s: u64,
    pub(super) created_timestamp_seconds: u64,
}
