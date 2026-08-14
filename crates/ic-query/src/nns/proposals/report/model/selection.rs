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

///
/// NnsProposalVote
///
/// Native NNS Governance ballot vote.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NnsProposalVote {
    /// Unspecified or unrecognized native vote code.
    Unspecified,
    /// Affirmative ballot.
    Yes,
    /// Negative ballot.
    No,
}

impl NnsProposalVote {
    /// Classify one raw native vote code.
    #[must_use]
    pub const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Yes,
            2 => Self::No,
            _ => Self::Unspecified,
        }
    }

    /// Return the canonical native code for this classification.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Unspecified => 0,
            Self::Yes => 1,
            Self::No => 2,
        }
    }

    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::Yes => "yes",
            Self::No => "no",
        }
    }
}

///
/// NnsProposalTopic
///
/// Native NNS Governance proposal topic.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NnsProposalTopic {
    /// Unspecified or unrecognized native topic code.
    Unspecified,
    /// Neuron management.
    NeuronManagement,
    /// Exchange-rate management.
    ExchangeRate,
    /// Network economics.
    NetworkEconomics,
    /// Governance policy.
    Governance,
    /// Node administration.
    NodeAdmin,
    /// Participant management.
    ParticipantManagement,
    /// Subnet management.
    SubnetManagement,
    /// Network-canister management.
    NetworkCanisterManagement,
    /// Know-your-customer policy.
    Kyc,
    /// Node-provider rewards.
    NodeProviderRewards,
    /// IC OS version deployment.
    IcOsVersionDeployment,
    /// IC OS version election.
    IcOsVersionElection,
    /// SNS and Community Fund policy.
    SnsAndCommunityFund,
    /// API boundary-node management.
    ApiBoundaryNodeManagement,
    /// Subnet rental.
    SubnetRental,
    /// Application-canister management.
    ApplicationCanisterManagement,
    /// Protocol-canister management.
    ProtocolCanisterManagement,
}

impl NnsProposalTopic {
    /// Classify one raw native topic code.
    #[must_use]
    pub const fn from_code(code: i32) -> Self {
        match code {
            1 => Self::NeuronManagement,
            2 => Self::ExchangeRate,
            3 => Self::NetworkEconomics,
            4 => Self::Governance,
            5 => Self::NodeAdmin,
            6 => Self::ParticipantManagement,
            7 => Self::SubnetManagement,
            8 => Self::NetworkCanisterManagement,
            9 => Self::Kyc,
            10 => Self::NodeProviderRewards,
            12 => Self::IcOsVersionDeployment,
            13 => Self::IcOsVersionElection,
            14 => Self::SnsAndCommunityFund,
            15 => Self::ApiBoundaryNodeManagement,
            16 => Self::SubnetRental,
            17 => Self::ApplicationCanisterManagement,
            18 => Self::ProtocolCanisterManagement,
            _ => Self::Unspecified,
        }
    }

    /// Return the canonical native code for this classification.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Unspecified => 0,
            Self::NeuronManagement => 1,
            Self::ExchangeRate => 2,
            Self::NetworkEconomics => 3,
            Self::Governance => 4,
            Self::NodeAdmin => 5,
            Self::ParticipantManagement => 6,
            Self::SubnetManagement => 7,
            Self::NetworkCanisterManagement => 8,
            Self::Kyc => 9,
            Self::NodeProviderRewards => 10,
            Self::IcOsVersionDeployment => 12,
            Self::IcOsVersionElection => 13,
            Self::SnsAndCommunityFund => 14,
            Self::ApiBoundaryNodeManagement => 15,
            Self::SubnetRental => 16,
            Self::ApplicationCanisterManagement => 17,
            Self::ProtocolCanisterManagement => 18,
        }
    }

    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::NeuronManagement => "neuron-management",
            Self::ExchangeRate => "exchange-rate",
            Self::NetworkEconomics => "network-economics",
            Self::Governance => "governance",
            Self::NodeAdmin => "node-admin",
            Self::ParticipantManagement => "participant-management",
            Self::SubnetManagement => "subnet-management",
            Self::NetworkCanisterManagement => "network-canister-management",
            Self::Kyc => "kyc",
            Self::NodeProviderRewards => "node-provider-rewards",
            Self::IcOsVersionDeployment => "ic-os-version-deployment",
            Self::IcOsVersionElection => "ic-os-version-election",
            Self::SnsAndCommunityFund => "sns-and-community-fund",
            Self::ApiBoundaryNodeManagement => "api-boundary-node-management",
            Self::SubnetRental => "subnet-rental",
            Self::ApplicationCanisterManagement => "application-canister-management",
            Self::ProtocolCanisterManagement => "protocol-canister-management",
        }
    }
}

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

    #[cfg(any(
        feature = "nns-host",
        all(feature = "canister", target_arch = "wasm32"),
        test
    ))]
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

    #[cfg(any(
        feature = "nns-host",
        all(feature = "canister", target_arch = "wasm32"),
        test
    ))]
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
            Self::Any => "any",
            Self::NeuronManagement => NnsProposalTopic::NeuronManagement.as_str(),
            Self::ExchangeRate => NnsProposalTopic::ExchangeRate.as_str(),
            Self::NetworkEconomics => NnsProposalTopic::NetworkEconomics.as_str(),
            Self::Governance => NnsProposalTopic::Governance.as_str(),
            Self::NodeAdmin => NnsProposalTopic::NodeAdmin.as_str(),
            Self::ParticipantManagement => NnsProposalTopic::ParticipantManagement.as_str(),
            Self::SubnetManagement => NnsProposalTopic::SubnetManagement.as_str(),
            Self::NetworkCanisterManagement => NnsProposalTopic::NetworkCanisterManagement.as_str(),
            Self::Kyc => NnsProposalTopic::Kyc.as_str(),
            Self::NodeProviderRewards => NnsProposalTopic::NodeProviderRewards.as_str(),
            Self::IcOsVersionDeployment => NnsProposalTopic::IcOsVersionDeployment.as_str(),
            Self::IcOsVersionElection => NnsProposalTopic::IcOsVersionElection.as_str(),
            Self::SnsAndCommunityFund => NnsProposalTopic::SnsAndCommunityFund.as_str(),
            Self::ApiBoundaryNodeManagement => NnsProposalTopic::ApiBoundaryNodeManagement.as_str(),
            Self::SubnetRental => NnsProposalTopic::SubnetRental.as_str(),
            Self::ApplicationCanisterManagement => {
                NnsProposalTopic::ApplicationCanisterManagement.as_str()
            }
            Self::ProtocolCanisterManagement => {
                NnsProposalTopic::ProtocolCanisterManagement.as_str()
            }
        }
    }

    pub(in crate::nns) const fn topic_code(self) -> Option<i32> {
        match self {
            Self::Any => None,
            Self::NeuronManagement => Some(NnsProposalTopic::NeuronManagement.code()),
            Self::ExchangeRate => Some(NnsProposalTopic::ExchangeRate.code()),
            Self::NetworkEconomics => Some(NnsProposalTopic::NetworkEconomics.code()),
            Self::Governance => Some(NnsProposalTopic::Governance.code()),
            Self::NodeAdmin => Some(NnsProposalTopic::NodeAdmin.code()),
            Self::ParticipantManagement => Some(NnsProposalTopic::ParticipantManagement.code()),
            Self::SubnetManagement => Some(NnsProposalTopic::SubnetManagement.code()),
            Self::NetworkCanisterManagement => {
                Some(NnsProposalTopic::NetworkCanisterManagement.code())
            }
            Self::Kyc => Some(NnsProposalTopic::Kyc.code()),
            Self::NodeProviderRewards => Some(NnsProposalTopic::NodeProviderRewards.code()),
            Self::IcOsVersionDeployment => Some(NnsProposalTopic::IcOsVersionDeployment.code()),
            Self::IcOsVersionElection => Some(NnsProposalTopic::IcOsVersionElection.code()),
            Self::SnsAndCommunityFund => Some(NnsProposalTopic::SnsAndCommunityFund.code()),
            Self::ApiBoundaryNodeManagement => {
                Some(NnsProposalTopic::ApiBoundaryNodeManagement.code())
            }
            Self::SubnetRental => Some(NnsProposalTopic::SubnetRental.code()),
            Self::ApplicationCanisterManagement => {
                Some(NnsProposalTopic::ApplicationCanisterManagement.code())
            }
            Self::ProtocolCanisterManagement => {
                Some(NnsProposalTopic::ProtocolCanisterManagement.code())
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

    #[test]
    fn proposal_vote_codes_and_labels_round_trip() {
        for (vote, code, label) in [
            (NnsProposalVote::Unspecified, 0, "unspecified"),
            (NnsProposalVote::Yes, 1, "yes"),
            (NnsProposalVote::No, 2, "no"),
        ] {
            assert_eq!(vote.code(), code);
            assert_eq!(NnsProposalVote::from_code(code), vote);
            assert_json_label(vote, label);
        }
        assert_eq!(NnsProposalVote::from_code(99), NnsProposalVote::Unspecified);
    }

    #[test]
    fn proposal_topic_codes_and_labels_round_trip() {
        for (topic, code, label) in [
            (NnsProposalTopic::Unspecified, 0, "unspecified"),
            (NnsProposalTopic::NeuronManagement, 1, "neuron-management"),
            (NnsProposalTopic::ExchangeRate, 2, "exchange-rate"),
            (NnsProposalTopic::NetworkEconomics, 3, "network-economics"),
            (NnsProposalTopic::Governance, 4, "governance"),
            (NnsProposalTopic::NodeAdmin, 5, "node-admin"),
            (
                NnsProposalTopic::ParticipantManagement,
                6,
                "participant-management",
            ),
            (NnsProposalTopic::SubnetManagement, 7, "subnet-management"),
            (
                NnsProposalTopic::NetworkCanisterManagement,
                8,
                "network-canister-management",
            ),
            (NnsProposalTopic::Kyc, 9, "kyc"),
            (
                NnsProposalTopic::NodeProviderRewards,
                10,
                "node-provider-rewards",
            ),
            (
                NnsProposalTopic::IcOsVersionDeployment,
                12,
                "ic-os-version-deployment",
            ),
            (
                NnsProposalTopic::IcOsVersionElection,
                13,
                "ic-os-version-election",
            ),
            (
                NnsProposalTopic::SnsAndCommunityFund,
                14,
                "sns-and-community-fund",
            ),
            (
                NnsProposalTopic::ApiBoundaryNodeManagement,
                15,
                "api-boundary-node-management",
            ),
            (NnsProposalTopic::SubnetRental, 16, "subnet-rental"),
            (
                NnsProposalTopic::ApplicationCanisterManagement,
                17,
                "application-canister-management",
            ),
            (
                NnsProposalTopic::ProtocolCanisterManagement,
                18,
                "protocol-canister-management",
            ),
        ] {
            assert_eq!(topic.code(), code);
            assert_eq!(NnsProposalTopic::from_code(code), topic);
            assert_json_label(topic, label);
        }
        assert_eq!(
            NnsProposalTopic::from_code(11),
            NnsProposalTopic::Unspecified
        );
        assert_eq!(
            NnsProposalTopic::from_code(99),
            NnsProposalTopic::Unspecified
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
