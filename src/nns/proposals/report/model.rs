//! Module: nns::proposals::report::model
//!
//! Responsibility: NNS proposal request and report DTOs.
//! Does not own: live governance transport, command parsing, or text rendering.
//! Boundary: defines stable JSON shapes for NNS proposal output.

use serde::{Deserialize, Serialize};

pub(in crate::nns) const NNS_PROPOSAL_SORT_API_LABEL: &str = "api";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ID_LABEL: &str = "id";
pub(in crate::nns) const NNS_PROPOSAL_SORT_STATUS_LABEL: &str = "status";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TOPIC_LABEL: &str = "topic";
pub(in crate::nns) const NNS_PROPOSAL_SORT_PROPOSER_LABEL: &str = "proposer";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TITLE_LABEL: &str = "title";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ACTION_LABEL: &str = "action";
pub(in crate::nns) const NNS_PROPOSAL_SORT_YES_LABEL: &str = "yes";
pub(in crate::nns) const NNS_PROPOSAL_SORT_NO_LABEL: &str = "no";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TOTAL_VOTES_LABEL: &str = "total-votes";
pub(in crate::nns) const NNS_PROPOSAL_SORT_BALLOTS_LABEL: &str = "ballots";
pub(in crate::nns) const NNS_PROPOSAL_SORT_REJECT_COST_LABEL: &str = "reject-cost";
pub(in crate::nns) const NNS_PROPOSAL_SORT_REWARD_ROUND_LABEL: &str = "reward-round";
pub(in crate::nns) const NNS_PROPOSAL_SORT_PROPOSED_LABEL: &str = "proposed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DECIDED_LABEL: &str = "decided";
pub(in crate::nns) const NNS_PROPOSAL_SORT_EXECUTED_LABEL: &str = "executed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_FAILED_LABEL: &str = "failed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ASC_LABEL: &str = "asc";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DESC_LABEL: &str = "desc";
pub(in crate::nns) const NNS_PROPOSAL_SORT_NONE_LABEL: &str = "none";

pub(in crate::nns) const NNS_PROPOSAL_STATUS_ANY_LABEL: &str = "any";
pub(in crate::nns) const NNS_PROPOSAL_STATUS_OPEN_LABEL: &str = "open";
pub(in crate::nns) const NNS_PROPOSAL_STATUS_REJECTED_LABEL: &str = "rejected";
pub(in crate::nns) const NNS_PROPOSAL_STATUS_ADOPTED_LABEL: &str = "adopted";
pub(in crate::nns) const NNS_PROPOSAL_STATUS_EXECUTED_LABEL: &str = "executed";
pub(in crate::nns) const NNS_PROPOSAL_STATUS_FAILED_LABEL: &str = "failed";
pub(in crate::nns) const NNS_PROPOSAL_STATUS_UNSPECIFIED_LABEL: &str = "unspecified";

pub(in crate::nns) const NNS_PROPOSAL_REWARD_STATUS_ACCEPT_VOTES_LABEL: &str = "accept-votes";
pub(in crate::nns) const NNS_PROPOSAL_REWARD_STATUS_READY_TO_SETTLE_LABEL: &str = "ready-to-settle";
pub(in crate::nns) const NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL: &str = "settled";
pub(in crate::nns) const NNS_PROPOSAL_REWARD_STATUS_INELIGIBLE_LABEL: &str = "ineligible";

pub(in crate::nns) const NNS_PROPOSAL_TOPIC_ANY_LABEL: &str = "any";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NEURON_MANAGEMENT_LABEL: &str = "neuron-management";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_EXCHANGE_RATE_LABEL: &str = "exchange-rate";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NETWORK_ECONOMICS_LABEL: &str = "network-economics";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL: &str = "governance";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NODE_ADMIN_LABEL: &str = "node-admin";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_PARTICIPANT_MANAGEMENT_LABEL: &str =
    "participant-management";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_LABEL: &str = "subnet-management";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NETWORK_CANISTER_MANAGEMENT_LABEL: &str =
    "network-canister-management";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_KYC_LABEL: &str = "kyc";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NODE_PROVIDER_REWARDS_LABEL: &str =
    "node-provider-rewards";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_IC_OS_VERSION_DEPLOYMENT_LABEL: &str =
    "ic-os-version-deployment";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_IC_OS_VERSION_ELECTION_LABEL: &str =
    "ic-os-version-election";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_SNS_AND_COMMUNITY_FUND_LABEL: &str =
    "sns-and-community-fund";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_API_BOUNDARY_NODE_MANAGEMENT_LABEL: &str =
    "api-boundary-node-management";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_SUBNET_RENTAL_LABEL: &str = "subnet-rental";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_APPLICATION_CANISTER_MANAGEMENT_LABEL: &str =
    "application-canister-management";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL: &str =
    "protocol-canister-management";

///
/// NnsProposalsRequest
///
/// Request accepted by the NNS proposal list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalsRequest {
    pub(in crate::nns::proposals) network: String,
    pub(in crate::nns::proposals) source_endpoint: String,
    pub(in crate::nns::proposals) now_unix_secs: u64,
    pub(in crate::nns::proposals) limit: u32,
    pub(in crate::nns::proposals) before_proposal_id: Option<u64>,
    pub(in crate::nns::proposals) status: NnsProposalStatusFilter,
    pub(in crate::nns::proposals) topic: NnsProposalTopicFilter,
    pub(in crate::nns::proposals) sort: NnsProposalsSort,
    pub(in crate::nns::proposals) sort_direction: NnsProposalSortDirection,
    pub(in crate::nns::proposals) verbose: bool,
}

///
/// NnsProposalRequest
///
/// Request accepted by the NNS proposal detail report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsProposalRequest {
    pub(in crate::nns::proposals) network: String,
    pub(in crate::nns::proposals) source_endpoint: String,
    pub(in crate::nns::proposals) now_unix_secs: u64,
    pub(in crate::nns::proposals) proposal_id: u64,
}

///
/// NnsProposalsReport
///
/// Serializable report for a bounded NNS governance proposal listing.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalsReport {
    pub schema_version: u32,
    pub network: String,
    pub governance_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub requested_limit: u32,
    pub before_proposal_id: Option<u64>,
    pub status_filter: String,
    pub topic_filter: String,
    pub sort: String,
    pub sort_direction: String,
    pub verbose: bool,
    pub proposal_count: usize,
    pub proposals: Vec<NnsProposalRow>,
}

///
/// NnsProposalReport
///
/// Serializable report for one NNS governance proposal detail lookup.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalReport {
    pub schema_version: u32,
    pub network: String,
    pub governance_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub proposal_id: u64,
    pub proposal: NnsProposalRow,
}

///
/// NnsProposalRow
///
/// Serializable row for one NNS governance proposal.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalRow {
    pub proposal_id: Option<u64>,
    pub proposer_neuron_id: Option<u64>,
    pub topic: i32,
    pub topic_text: String,
    pub status: i32,
    pub status_text: String,
    pub reward_status: i32,
    pub reward_status_text: String,
    pub title: Option<String>,
    pub summary: String,
    pub url: String,
    pub action_text: Option<String>,
    pub reject_cost_e8s: u64,
    pub proposal_timestamp_seconds: u64,
    pub proposed_at: String,
    pub deadline_timestamp_seconds: Option<u64>,
    pub deadline_at: Option<String>,
    pub decided_timestamp_seconds: u64,
    pub decided_at: Option<String>,
    pub executed_timestamp_seconds: u64,
    pub executed_at: Option<String>,
    pub failed_timestamp_seconds: u64,
    pub failed_at: Option<String>,
    pub reward_event_round: u64,
    pub total_potential_voting_power: Option<u64>,
    pub latest_tally: Option<NnsProposalTally>,
    pub ballot_count: usize,
}

///
/// NnsProposalTally
///
/// Serializable NNS proposal vote tally.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(in crate::nns) struct NnsProposalTally {
    pub timestamp_seconds: u64,
    pub yes: u64,
    pub no: u64,
    pub total: u64,
}

///
/// NnsProposalsSort
///
/// Report-model sort selector for NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::nns) enum NnsProposalsSort {
    #[default]
    Api,
    Id,
    Status,
    Topic,
    Proposer,
    Title,
    Action,
    Yes,
    No,
    TotalVotes,
    Ballots,
    RejectCost,
    RewardRound,
    Proposed,
    Decided,
    Executed,
    Failed,
}

impl NnsProposalsSort {
    pub(in crate::nns) const fn as_str(self) -> &'static str {
        match self {
            Self::Api => NNS_PROPOSAL_SORT_API_LABEL,
            Self::Id => NNS_PROPOSAL_SORT_ID_LABEL,
            Self::Status => NNS_PROPOSAL_SORT_STATUS_LABEL,
            Self::Topic => NNS_PROPOSAL_SORT_TOPIC_LABEL,
            Self::Proposer => NNS_PROPOSAL_SORT_PROPOSER_LABEL,
            Self::Title => NNS_PROPOSAL_SORT_TITLE_LABEL,
            Self::Action => NNS_PROPOSAL_SORT_ACTION_LABEL,
            Self::Yes => NNS_PROPOSAL_SORT_YES_LABEL,
            Self::No => NNS_PROPOSAL_SORT_NO_LABEL,
            Self::TotalVotes => NNS_PROPOSAL_SORT_TOTAL_VOTES_LABEL,
            Self::Ballots => NNS_PROPOSAL_SORT_BALLOTS_LABEL,
            Self::RejectCost => NNS_PROPOSAL_SORT_REJECT_COST_LABEL,
            Self::RewardRound => NNS_PROPOSAL_SORT_REWARD_ROUND_LABEL,
            Self::Proposed => NNS_PROPOSAL_SORT_PROPOSED_LABEL,
            Self::Decided => NNS_PROPOSAL_SORT_DECIDED_LABEL,
            Self::Executed => NNS_PROPOSAL_SORT_EXECUTED_LABEL,
            Self::Failed => NNS_PROPOSAL_SORT_FAILED_LABEL,
        }
    }

    pub(in crate::nns) const fn default_direction(self) -> NnsProposalSortDirection {
        match self {
            Self::Status | Self::Topic | Self::Proposer | Self::Title | Self::Action => {
                NnsProposalSortDirection::Asc
            }
            _ => NnsProposalSortDirection::Desc,
        }
    }

    pub(in crate::nns) const fn uses_local_direction(self) -> bool {
        !matches!(self, Self::Api)
    }

    pub(in crate::nns) const fn direction_label(
        self,
        direction: NnsProposalSortDirection,
    ) -> &'static str {
        match self {
            Self::Api => NNS_PROPOSAL_SORT_NONE_LABEL,
            _ => direction.as_str(),
        }
    }
}

///
/// NnsProposalSortDirection
///
/// Report-model direction selector for local NNS proposal sorting.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::nns) enum NnsProposalSortDirection {
    Asc,
    #[default]
    Desc,
}

impl NnsProposalSortDirection {
    pub(in crate::nns) const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => NNS_PROPOSAL_SORT_ASC_LABEL,
            Self::Desc => NNS_PROPOSAL_SORT_DESC_LABEL,
        }
    }
}

///
/// NnsProposalStatusFilter
///
/// Report-model status filter for bounded NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::nns) enum NnsProposalStatusFilter {
    #[default]
    Any,
    Open,
    Rejected,
    Adopted,
    Executed,
    Failed,
}

impl NnsProposalStatusFilter {
    pub(in crate::nns) const fn as_str(self) -> &'static str {
        match self {
            Self::Any => NNS_PROPOSAL_STATUS_ANY_LABEL,
            Self::Open => NNS_PROPOSAL_STATUS_OPEN_LABEL,
            Self::Rejected => NNS_PROPOSAL_STATUS_REJECTED_LABEL,
            Self::Adopted => NNS_PROPOSAL_STATUS_ADOPTED_LABEL,
            Self::Executed => NNS_PROPOSAL_STATUS_EXECUTED_LABEL,
            Self::Failed => NNS_PROPOSAL_STATUS_FAILED_LABEL,
        }
    }

    pub(in crate::nns) const fn governance_status_code(self) -> Option<i32> {
        match self {
            Self::Any => None,
            Self::Open => Some(1),
            Self::Rejected => Some(2),
            Self::Adopted => Some(3),
            Self::Executed => Some(4),
            Self::Failed => Some(5),
        }
    }
}

///
/// NnsProposalTopicFilter
///
/// Report-model topic filter for bounded NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::nns) enum NnsProposalTopicFilter {
    #[default]
    Any,
    NeuronManagement,
    ExchangeRate,
    NetworkEconomics,
    Governance,
    NodeAdmin,
    ParticipantManagement,
    SubnetManagement,
    NetworkCanisterManagement,
    Kyc,
    NodeProviderRewards,
    IcOsVersionDeployment,
    IcOsVersionElection,
    SnsAndCommunityFund,
    ApiBoundaryNodeManagement,
    SubnetRental,
    ApplicationCanisterManagement,
    ProtocolCanisterManagement,
}

impl NnsProposalTopicFilter {
    pub(in crate::nns) const fn as_str(self) -> &'static str {
        match self {
            Self::Any => NNS_PROPOSAL_TOPIC_ANY_LABEL,
            Self::NeuronManagement => NNS_PROPOSAL_TOPIC_NEURON_MANAGEMENT_LABEL,
            Self::ExchangeRate => NNS_PROPOSAL_TOPIC_EXCHANGE_RATE_LABEL,
            Self::NetworkEconomics => NNS_PROPOSAL_TOPIC_NETWORK_ECONOMICS_LABEL,
            Self::Governance => NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
            Self::NodeAdmin => NNS_PROPOSAL_TOPIC_NODE_ADMIN_LABEL,
            Self::ParticipantManagement => NNS_PROPOSAL_TOPIC_PARTICIPANT_MANAGEMENT_LABEL,
            Self::SubnetManagement => NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_LABEL,
            Self::NetworkCanisterManagement => NNS_PROPOSAL_TOPIC_NETWORK_CANISTER_MANAGEMENT_LABEL,
            Self::Kyc => NNS_PROPOSAL_TOPIC_KYC_LABEL,
            Self::NodeProviderRewards => NNS_PROPOSAL_TOPIC_NODE_PROVIDER_REWARDS_LABEL,
            Self::IcOsVersionDeployment => NNS_PROPOSAL_TOPIC_IC_OS_VERSION_DEPLOYMENT_LABEL,
            Self::IcOsVersionElection => NNS_PROPOSAL_TOPIC_IC_OS_VERSION_ELECTION_LABEL,
            Self::SnsAndCommunityFund => NNS_PROPOSAL_TOPIC_SNS_AND_COMMUNITY_FUND_LABEL,
            Self::ApiBoundaryNodeManagement => {
                NNS_PROPOSAL_TOPIC_API_BOUNDARY_NODE_MANAGEMENT_LABEL
            }
            Self::SubnetRental => NNS_PROPOSAL_TOPIC_SUBNET_RENTAL_LABEL,
            Self::ApplicationCanisterManagement => {
                NNS_PROPOSAL_TOPIC_APPLICATION_CANISTER_MANAGEMENT_LABEL
            }
            Self::ProtocolCanisterManagement => {
                NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_LABEL
            }
        }
    }

    pub(in crate::nns) const fn topic_code(self) -> Option<i32> {
        match self {
            Self::Any => None,
            Self::NeuronManagement => Some(1),
            Self::ExchangeRate => Some(2),
            Self::NetworkEconomics => Some(3),
            Self::Governance => Some(4),
            Self::NodeAdmin => Some(5),
            Self::ParticipantManagement => Some(6),
            Self::SubnetManagement => Some(7),
            Self::NetworkCanisterManagement => Some(8),
            Self::Kyc => Some(9),
            Self::NodeProviderRewards => Some(10),
            Self::IcOsVersionDeployment => Some(12),
            Self::IcOsVersionElection => Some(13),
            Self::SnsAndCommunityFund => Some(14),
            Self::ApiBoundaryNodeManagement => Some(15),
            Self::SubnetRental => Some(16),
            Self::ApplicationCanisterManagement => Some(17),
            Self::ProtocolCanisterManagement => Some(18),
        }
    }
}
