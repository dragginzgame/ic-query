//! Module: nns::proposals::values
//!
//! Responsibility: clap value enums for NNS proposal list options.
//! Does not own: live NNS governance calls, report sorting, or text rendering.
//! Boundary: converts CLI option values into NNS proposal report-model values.

use crate::nns::proposals::report::{
    NnsProposalListSort, NnsProposalRewardStatusFilter, NnsProposalStatusFilter,
    NnsProposalTopicFilter,
};
use clap::ValueEnum;

pub(in crate::nns::proposals) const NNS_PROPOSAL_ID_ARG: &str = "proposal-id";
pub(in crate::nns::proposals) const NNS_PROPOSAL_BALLOTS_FLAG: &str = "ballots";
pub(in crate::nns::proposals) const NNS_PROPOSAL_VERBOSE_FLAG: &str = "verbose";
pub(in crate::nns::proposals) const NNS_PROPOSAL_LIST_REWARD_STATUS_ARG: &str = "reward-status";
pub(in crate::nns::proposals) const NNS_PROPOSAL_LIST_SORT_VALUE_NAME: &str = concat!(
    "api|id|status|reward-status|topic|proposer|title|action|yes|no|total-votes|",
    "voting-power|ballots|reject-cost|reward-round|proposed|deadline|decided|",
    "executed|failed"
);
pub(in crate::nns::proposals) const NNS_PROPOSAL_LIST_LOCAL_SORT_VALUE_NAME: &str = concat!(
    "id|status|reward-status|topic|proposer|title|action|yes|no|total-votes|",
    "voting-power|ballots|reject-cost|reward-round|proposed|deadline|decided|",
    "executed|failed"
);

///
/// NnsProposalListSortArg
///
/// Command-local clap value accepted by `icq nns proposal list --sort`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::nns::proposals) enum NnsProposalListSortArg {
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

impl From<NnsProposalListSortArg> for NnsProposalListSort {
    fn from(value: NnsProposalListSortArg) -> Self {
        match value {
            NnsProposalListSortArg::Api => Self::Api,
            NnsProposalListSortArg::Id => Self::Id,
            NnsProposalListSortArg::Status => Self::Status,
            NnsProposalListSortArg::RewardStatus => Self::RewardStatus,
            NnsProposalListSortArg::Topic => Self::Topic,
            NnsProposalListSortArg::Proposer => Self::Proposer,
            NnsProposalListSortArg::Title => Self::Title,
            NnsProposalListSortArg::Action => Self::Action,
            NnsProposalListSortArg::Yes => Self::Yes,
            NnsProposalListSortArg::No => Self::No,
            NnsProposalListSortArg::TotalVotes => Self::TotalVotes,
            NnsProposalListSortArg::VotingPower => Self::VotingPower,
            NnsProposalListSortArg::Ballots => Self::Ballots,
            NnsProposalListSortArg::RejectCost => Self::RejectCost,
            NnsProposalListSortArg::RewardRound => Self::RewardRound,
            NnsProposalListSortArg::Proposed => Self::Proposed,
            NnsProposalListSortArg::Deadline => Self::Deadline,
            NnsProposalListSortArg::Decided => Self::Decided,
            NnsProposalListSortArg::Executed => Self::Executed,
            NnsProposalListSortArg::Failed => Self::Failed,
        }
    }
}

///
/// NnsProposalStatusArg
///
/// Command-local clap value accepted by `icq nns proposal list --status`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::nns::proposals) enum NnsProposalStatusArg {
    #[default]
    Any,
    Open,
    Rejected,
    Adopted,
    Executed,
    Failed,
}

impl From<NnsProposalStatusArg> for NnsProposalStatusFilter {
    fn from(value: NnsProposalStatusArg) -> Self {
        match value {
            NnsProposalStatusArg::Any => Self::Any,
            NnsProposalStatusArg::Open => Self::Open,
            NnsProposalStatusArg::Rejected => Self::Rejected,
            NnsProposalStatusArg::Adopted => Self::Adopted,
            NnsProposalStatusArg::Executed => Self::Executed,
            NnsProposalStatusArg::Failed => Self::Failed,
        }
    }
}

///
/// NnsProposalRewardStatusArg
///
/// Command-local clap value accepted by `icq nns proposal list --reward-status`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::nns::proposals) enum NnsProposalRewardStatusArg {
    #[default]
    Any,
    AcceptVotes,
    ReadyToSettle,
    Settled,
    Ineligible,
}

impl From<NnsProposalRewardStatusArg> for NnsProposalRewardStatusFilter {
    fn from(value: NnsProposalRewardStatusArg) -> Self {
        match value {
            NnsProposalRewardStatusArg::Any => Self::Any,
            NnsProposalRewardStatusArg::AcceptVotes => Self::AcceptVotes,
            NnsProposalRewardStatusArg::ReadyToSettle => Self::ReadyToSettle,
            NnsProposalRewardStatusArg::Settled => Self::Settled,
            NnsProposalRewardStatusArg::Ineligible => Self::Ineligible,
        }
    }
}

///
/// NnsProposalTopicArg
///
/// Command-local clap value accepted by `icq nns proposal list --topic`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::nns::proposals) enum NnsProposalTopicArg {
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

impl From<NnsProposalTopicArg> for NnsProposalTopicFilter {
    fn from(value: NnsProposalTopicArg) -> Self {
        match value {
            NnsProposalTopicArg::Any => Self::Any,
            NnsProposalTopicArg::NeuronManagement => Self::NeuronManagement,
            NnsProposalTopicArg::ExchangeRate => Self::ExchangeRate,
            NnsProposalTopicArg::NetworkEconomics => Self::NetworkEconomics,
            NnsProposalTopicArg::Governance => Self::Governance,
            NnsProposalTopicArg::NodeAdmin => Self::NodeAdmin,
            NnsProposalTopicArg::ParticipantManagement => Self::ParticipantManagement,
            NnsProposalTopicArg::SubnetManagement => Self::SubnetManagement,
            NnsProposalTopicArg::NetworkCanisterManagement => Self::NetworkCanisterManagement,
            NnsProposalTopicArg::Kyc => Self::Kyc,
            NnsProposalTopicArg::NodeProviderRewards => Self::NodeProviderRewards,
            NnsProposalTopicArg::IcOsVersionDeployment => Self::IcOsVersionDeployment,
            NnsProposalTopicArg::IcOsVersionElection => Self::IcOsVersionElection,
            NnsProposalTopicArg::SnsAndCommunityFund => Self::SnsAndCommunityFund,
            NnsProposalTopicArg::ApiBoundaryNodeManagement => Self::ApiBoundaryNodeManagement,
            NnsProposalTopicArg::SubnetRental => Self::SubnetRental,
            NnsProposalTopicArg::ApplicationCanisterManagement => {
                Self::ApplicationCanisterManagement
            }
            NnsProposalTopicArg::ProtocolCanisterManagement => Self::ProtocolCanisterManagement,
        }
    }
}
