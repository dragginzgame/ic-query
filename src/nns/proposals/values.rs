//! Module: nns::proposals::values
//!
//! Responsibility: clap value enums for NNS proposal list options.
//! Does not own: live NNS governance calls, report sorting, or text rendering.
//! Boundary: converts CLI option values into NNS proposal report-model values.

use crate::nns::proposals::report::{
    NnsProposalRewardStatusFilter, NnsProposalStatusFilter, NnsProposalTopicFilter,
    NnsProposalsSort,
};
use clap::ValueEnum;

pub(in crate::nns::proposals) const NNS_PROPOSAL_ID_ARG: &str = "proposal-id";
pub(in crate::nns::proposals) const NNS_PROPOSAL_BALLOTS_FLAG: &str = "ballots";
pub(in crate::nns::proposals) const NNS_PROPOSAL_VERBOSE_FLAG: &str = "verbose";
pub(in crate::nns::proposals) const NNS_PROPOSALS_REWARD_STATUS_ARG: &str = "reward-status";
pub(in crate::nns::proposals) const NNS_PROPOSALS_SORT_VALUE_NAME: &str = concat!(
    "api|id|status|topic|proposer|title|action|yes|no|total-votes|",
    "ballots|reject-cost|reward-round|proposed|decided|executed|failed"
);
pub(in crate::nns::proposals) const NNS_PROPOSALS_LOCAL_SORT_VALUE_NAME: &str = concat!(
    "id|status|topic|proposer|title|action|yes|no|total-votes|",
    "ballots|reject-cost|reward-round|proposed|decided|executed|failed"
);

///
/// NnsProposalsSortArg
///
/// Command-local clap value accepted by `icq nns proposal list --sort`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::nns::proposals) enum NnsProposalsSortArg {
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

impl From<NnsProposalsSortArg> for NnsProposalsSort {
    fn from(value: NnsProposalsSortArg) -> Self {
        match value {
            NnsProposalsSortArg::Api => Self::Api,
            NnsProposalsSortArg::Id => Self::Id,
            NnsProposalsSortArg::Status => Self::Status,
            NnsProposalsSortArg::Topic => Self::Topic,
            NnsProposalsSortArg::Proposer => Self::Proposer,
            NnsProposalsSortArg::Title => Self::Title,
            NnsProposalsSortArg::Action => Self::Action,
            NnsProposalsSortArg::Yes => Self::Yes,
            NnsProposalsSortArg::No => Self::No,
            NnsProposalsSortArg::TotalVotes => Self::TotalVotes,
            NnsProposalsSortArg::Ballots => Self::Ballots,
            NnsProposalsSortArg::RejectCost => Self::RejectCost,
            NnsProposalsSortArg::RewardRound => Self::RewardRound,
            NnsProposalsSortArg::Proposed => Self::Proposed,
            NnsProposalsSortArg::Decided => Self::Decided,
            NnsProposalsSortArg::Executed => Self::Executed,
            NnsProposalsSortArg::Failed => Self::Failed,
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
