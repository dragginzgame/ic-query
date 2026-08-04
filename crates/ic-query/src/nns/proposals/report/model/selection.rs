//! Module: nns::proposals::report::model::selection
//!
//! Responsibility: NNS proposal filter, sort, and protocol vocabulary.
//! Does not own: request DTOs, serialized report DTOs, or report rendering.
//! Boundary: keeps public selection types and their canonical labels/codes together.

use serde::{Deserialize, Serialize};

pub(in crate::nns) const NNS_PROPOSAL_SORT_API_LABEL: &str = "api";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ID_LABEL: &str = "id";
pub(in crate::nns) const NNS_PROPOSAL_SORT_STATUS_LABEL: &str = "status";
pub(in crate::nns) const NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL: &str = "reward-status";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TOPIC_LABEL: &str = "topic";
pub(in crate::nns) const NNS_PROPOSAL_SORT_PROPOSER_LABEL: &str = "proposer";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TITLE_LABEL: &str = "title";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ACTION_LABEL: &str = "action";
pub(in crate::nns) const NNS_PROPOSAL_SORT_YES_LABEL: &str = "yes";
pub(in crate::nns) const NNS_PROPOSAL_SORT_NO_LABEL: &str = "no";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TOTAL_VOTES_LABEL: &str = "total-votes";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TALLY_TIME_LABEL: &str = "tally-time";
pub(in crate::nns) const NNS_PROPOSAL_SORT_VOTING_POWER_LABEL: &str = "voting-power";
pub(in crate::nns) const NNS_PROPOSAL_SORT_BALLOTS_LABEL: &str = "ballots";
pub(in crate::nns) const NNS_PROPOSAL_SORT_REJECT_COST_LABEL: &str = "reject-cost";
pub(in crate::nns) const NNS_PROPOSAL_SORT_REWARD_ROUND_LABEL: &str = "reward-round";
pub(in crate::nns) const NNS_PROPOSAL_SORT_PROPOSED_LABEL: &str = "proposed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DEADLINE_LABEL: &str = "deadline";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DECIDED_LABEL: &str = "decided";
pub(in crate::nns) const NNS_PROPOSAL_SORT_EXECUTED_LABEL: &str = "executed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_FAILED_LABEL: &str = "failed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ASC_LABEL: &str = "asc";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DESC_LABEL: &str = "desc";
pub(in crate::nns) const NNS_PROPOSAL_SORT_NONE_LABEL: &str = "none";

///
/// NnsProposalStatus
///
/// Native NNS Governance proposal decision status.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NnsProposalStatus {
    /// Unspecified or unrecognized native status code.
    Unspecified,
    /// Proposal remains open for voting.
    Open,
    /// Proposal was rejected.
    Rejected,
    /// Proposal was adopted but has not completed execution.
    Adopted,
    /// Proposal executed successfully.
    Executed,
    /// Proposal execution failed.
    Failed,
}

impl NnsProposalStatus {
    /// Classify one raw native status code.
    #[must_use]
    pub const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Open,
            2 => Self::Rejected,
            3 => Self::Adopted,
            4 => Self::Executed,
            5 => Self::Failed,
            _ => Self::Unspecified,
        }
    }

    /// Return the canonical native code for this classification.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Unspecified => 0,
            Self::Open => 1,
            Self::Rejected => 2,
            Self::Adopted => 3,
            Self::Executed => 4,
            Self::Failed => 5,
        }
    }

    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::Open => "open",
            Self::Rejected => "rejected",
            Self::Adopted => "adopted",
            Self::Executed => "executed",
            Self::Failed => "failed",
        }
    }
}

///
/// NnsProposalRewardStatus
///
/// Native NNS Governance proposal reward-settlement status.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NnsProposalRewardStatus {
    /// Unspecified or unrecognized native reward-status code.
    Unspecified,
    /// Proposal still accepts votes for reward purposes.
    AcceptVotes,
    /// Proposal is ready for reward settlement.
    ReadyToSettle,
    /// Proposal rewards have settled.
    Settled,
    /// Proposal is not eligible for voting rewards.
    Ineligible,
}

impl NnsProposalRewardStatus {
    /// Classify one raw native reward-status code.
    #[must_use]
    pub const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::AcceptVotes,
            2 => Self::ReadyToSettle,
            3 => Self::Settled,
            4 => Self::Ineligible,
            _ => Self::Unspecified,
        }
    }

    /// Return the canonical native code for this classification.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Unspecified => 0,
            Self::AcceptVotes => 1,
            Self::ReadyToSettle => 2,
            Self::Settled => 3,
            Self::Ineligible => 4,
        }
    }

    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::AcceptVotes => "accept-votes",
            Self::ReadyToSettle => "ready-to-settle",
            Self::Settled => "settled",
            Self::Ineligible => "ineligible",
        }
    }
}

#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_VOTE_UNSPECIFIED_LABEL: &str = "unspecified";
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_VOTE_YES_LABEL: &str = "yes";
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_VOTE_NO_LABEL: &str = "no";
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_VOTE_YES_CODE: i32 = 1;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_VOTE_NO_CODE: i32 = 2;

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
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_UNSPECIFIED_LABEL: &str = "unspecified";
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NEURON_MANAGEMENT_CODE: i32 = 1;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_EXCHANGE_RATE_CODE: i32 = 2;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NETWORK_ECONOMICS_CODE: i32 = 3;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE: i32 = 4;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NODE_ADMIN_CODE: i32 = 5;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_PARTICIPANT_MANAGEMENT_CODE: i32 = 6;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_CODE: i32 = 7;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NETWORK_CANISTER_MANAGEMENT_CODE: i32 = 8;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_KYC_CODE: i32 = 9;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_NODE_PROVIDER_REWARDS_CODE: i32 = 10;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_IC_OS_VERSION_DEPLOYMENT_CODE: i32 = 12;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_IC_OS_VERSION_ELECTION_CODE: i32 = 13;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_SNS_AND_COMMUNITY_FUND_CODE: i32 = 14;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_API_BOUNDARY_NODE_MANAGEMENT_CODE: i32 = 15;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_SUBNET_RENTAL_CODE: i32 = 16;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_APPLICATION_CANISTER_MANAGEMENT_CODE: i32 = 17;
#[cfg(feature = "host")]
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_CODE: i32 = 18;

///
/// NnsProposalListSort
///
/// Report-model sort selector for NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NnsProposalListSort {
    #[default]
    Api,
    Id,
    Status,
    RewardStatus,
    Topic,
    Proposer,
    Title,
    Action,
    Yes,
    No,
    TotalVotes,
    TallyTime,
    VotingPower,
    Ballots,
    RejectCost,
    RewardRound,
    Proposed,
    Deadline,
    Decided,
    Executed,
    Failed,
}

impl NnsProposalListSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => NNS_PROPOSAL_SORT_API_LABEL,
            Self::Id => NNS_PROPOSAL_SORT_ID_LABEL,
            Self::Status => NNS_PROPOSAL_SORT_STATUS_LABEL,
            Self::RewardStatus => NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL,
            Self::Topic => NNS_PROPOSAL_SORT_TOPIC_LABEL,
            Self::Proposer => NNS_PROPOSAL_SORT_PROPOSER_LABEL,
            Self::Title => NNS_PROPOSAL_SORT_TITLE_LABEL,
            Self::Action => NNS_PROPOSAL_SORT_ACTION_LABEL,
            Self::Yes => NNS_PROPOSAL_SORT_YES_LABEL,
            Self::No => NNS_PROPOSAL_SORT_NO_LABEL,
            Self::TotalVotes => NNS_PROPOSAL_SORT_TOTAL_VOTES_LABEL,
            Self::TallyTime => NNS_PROPOSAL_SORT_TALLY_TIME_LABEL,
            Self::VotingPower => NNS_PROPOSAL_SORT_VOTING_POWER_LABEL,
            Self::Ballots => NNS_PROPOSAL_SORT_BALLOTS_LABEL,
            Self::RejectCost => NNS_PROPOSAL_SORT_REJECT_COST_LABEL,
            Self::RewardRound => NNS_PROPOSAL_SORT_REWARD_ROUND_LABEL,
            Self::Proposed => NNS_PROPOSAL_SORT_PROPOSED_LABEL,
            Self::Deadline => NNS_PROPOSAL_SORT_DEADLINE_LABEL,
            Self::Decided => NNS_PROPOSAL_SORT_DECIDED_LABEL,
            Self::Executed => NNS_PROPOSAL_SORT_EXECUTED_LABEL,
            Self::Failed => NNS_PROPOSAL_SORT_FAILED_LABEL,
        }
    }

    #[must_use]
    pub const fn default_direction(self) -> NnsProposalSortDirection {
        match self {
            Self::Status
            | Self::RewardStatus
            | Self::Topic
            | Self::Proposer
            | Self::Title
            | Self::Action => NnsProposalSortDirection::Asc,
            _ => NnsProposalSortDirection::Desc,
        }
    }

    #[must_use]
    pub const fn uses_local_direction(self) -> bool {
        !matches!(self, Self::Api)
    }

    #[must_use]
    pub const fn direction_label(self, direction: NnsProposalSortDirection) -> &'static str {
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
pub enum NnsProposalSortDirection {
    Asc,
    #[default]
    Desc,
}

impl NnsProposalSortDirection {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
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
pub enum NnsProposalStatusFilter {
    #[default]
    Any,
    Open,
    Rejected,
    Adopted,
    Executed,
    Failed,
}

///
/// NnsProposalRewardStatusFilter
///
/// Report-model reward status filter for bounded NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NnsProposalRewardStatusFilter {
    #[default]
    Any,
    AcceptVotes,
    ReadyToSettle,
    Settled,
    Ineligible,
}

impl NnsProposalRewardStatusFilter {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::AcceptVotes => NnsProposalRewardStatus::AcceptVotes.as_str(),
            Self::ReadyToSettle => NnsProposalRewardStatus::ReadyToSettle.as_str(),
            Self::Settled => NnsProposalRewardStatus::Settled.as_str(),
            Self::Ineligible => NnsProposalRewardStatus::Ineligible.as_str(),
        }
    }

    #[cfg(feature = "host")]
    pub(in crate::nns) const fn governance_reward_status_code(self) -> Option<i32> {
        match self {
            Self::Any => None,
            Self::AcceptVotes => Some(NnsProposalRewardStatus::AcceptVotes.code()),
            Self::ReadyToSettle => Some(NnsProposalRewardStatus::ReadyToSettle.code()),
            Self::Settled => Some(NnsProposalRewardStatus::Settled.code()),
            Self::Ineligible => Some(NnsProposalRewardStatus::Ineligible.code()),
        }
    }
}

impl NnsProposalStatusFilter {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Open => NnsProposalStatus::Open.as_str(),
            Self::Rejected => NnsProposalStatus::Rejected.as_str(),
            Self::Adopted => NnsProposalStatus::Adopted.as_str(),
            Self::Executed => NnsProposalStatus::Executed.as_str(),
            Self::Failed => NnsProposalStatus::Failed.as_str(),
        }
    }

    #[cfg(feature = "host")]
    pub(in crate::nns) const fn governance_status_code(self) -> Option<i32> {
        match self {
            Self::Any => None,
            Self::Open => Some(NnsProposalStatus::Open.code()),
            Self::Rejected => Some(NnsProposalStatus::Rejected.code()),
            Self::Adopted => Some(NnsProposalStatus::Adopted.code()),
            Self::Executed => Some(NnsProposalStatus::Executed.code()),
            Self::Failed => Some(NnsProposalStatus::Failed.code()),
        }
    }
}

///
/// NnsProposalTopicFilter
///
/// Report-model topic filter for bounded NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NnsProposalTopicFilter {
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
    #[must_use]
    pub const fn as_str(self) -> &'static str {
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

    #[cfg(feature = "host")]
    pub(in crate::nns) const fn topic_code(self) -> Option<i32> {
        match self {
            Self::Any => None,
            Self::NeuronManagement => Some(NNS_PROPOSAL_TOPIC_NEURON_MANAGEMENT_CODE),
            Self::ExchangeRate => Some(NNS_PROPOSAL_TOPIC_EXCHANGE_RATE_CODE),
            Self::NetworkEconomics => Some(NNS_PROPOSAL_TOPIC_NETWORK_ECONOMICS_CODE),
            Self::Governance => Some(NNS_PROPOSAL_TOPIC_GOVERNANCE_CODE),
            Self::NodeAdmin => Some(NNS_PROPOSAL_TOPIC_NODE_ADMIN_CODE),
            Self::ParticipantManagement => Some(NNS_PROPOSAL_TOPIC_PARTICIPANT_MANAGEMENT_CODE),
            Self::SubnetManagement => Some(NNS_PROPOSAL_TOPIC_SUBNET_MANAGEMENT_CODE),
            Self::NetworkCanisterManagement => {
                Some(NNS_PROPOSAL_TOPIC_NETWORK_CANISTER_MANAGEMENT_CODE)
            }
            Self::Kyc => Some(NNS_PROPOSAL_TOPIC_KYC_CODE),
            Self::NodeProviderRewards => Some(NNS_PROPOSAL_TOPIC_NODE_PROVIDER_REWARDS_CODE),
            Self::IcOsVersionDeployment => Some(NNS_PROPOSAL_TOPIC_IC_OS_VERSION_DEPLOYMENT_CODE),
            Self::IcOsVersionElection => Some(NNS_PROPOSAL_TOPIC_IC_OS_VERSION_ELECTION_CODE),
            Self::SnsAndCommunityFund => Some(NNS_PROPOSAL_TOPIC_SNS_AND_COMMUNITY_FUND_CODE),
            Self::ApiBoundaryNodeManagement => {
                Some(NNS_PROPOSAL_TOPIC_API_BOUNDARY_NODE_MANAGEMENT_CODE)
            }
            Self::SubnetRental => Some(NNS_PROPOSAL_TOPIC_SUBNET_RENTAL_CODE),
            Self::ApplicationCanisterManagement => {
                Some(NNS_PROPOSAL_TOPIC_APPLICATION_CANISTER_MANAGEMENT_CODE)
            }
            Self::ProtocolCanisterManagement => {
                Some(NNS_PROPOSAL_TOPIC_PROTOCOL_CANISTER_MANAGEMENT_CODE)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_status_codes_and_labels_round_trip() {
        for (status, code, label) in [
            (NnsProposalStatus::Unspecified, 0, "unspecified"),
            (NnsProposalStatus::Open, 1, "open"),
            (NnsProposalStatus::Rejected, 2, "rejected"),
            (NnsProposalStatus::Adopted, 3, "adopted"),
            (NnsProposalStatus::Executed, 4, "executed"),
            (NnsProposalStatus::Failed, 5, "failed"),
        ] {
            assert_eq!(status.code(), code);
            assert_eq!(NnsProposalStatus::from_code(code), status);
            assert_json_label(status, label);
        }
        assert_eq!(
            NnsProposalStatus::from_code(99),
            NnsProposalStatus::Unspecified
        );
    }

    #[test]
    fn proposal_reward_status_codes_and_labels_round_trip() {
        for (status, code, label) in [
            (NnsProposalRewardStatus::Unspecified, 0, "unspecified"),
            (NnsProposalRewardStatus::AcceptVotes, 1, "accept-votes"),
            (NnsProposalRewardStatus::ReadyToSettle, 2, "ready-to-settle"),
            (NnsProposalRewardStatus::Settled, 3, "settled"),
            (NnsProposalRewardStatus::Ineligible, 4, "ineligible"),
        ] {
            assert_eq!(status.code(), code);
            assert_eq!(NnsProposalRewardStatus::from_code(code), status);
            assert_json_label(status, label);
        }
        assert_eq!(
            NnsProposalRewardStatus::from_code(99),
            NnsProposalRewardStatus::Unspecified
        );
    }

    fn assert_json_label<T>(value: T, label: &str)
    where
        T: Copy + std::fmt::Debug + Eq + Serialize + serde::de::DeserializeOwned,
    {
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            format!("\"{label}\"")
        );
        assert_eq!(
            serde_json::from_str::<T>(&format!("\"{label}\"")).unwrap(),
            value
        );
    }
}
