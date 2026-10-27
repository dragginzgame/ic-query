//! Module: nns::proposals::values
//!
//! Responsibility: clap value enums for NNS proposal list options.
//! Does not own: live NNS governance calls, report sorting, or text rendering.
//! Boundary: converts CLI option values into NNS proposal report-model values.

use crate::cli::value_enum_with_conversion;
use ic_query::nns::proposals::{
    NnsProposalListSort, NnsProposalRewardStatusFilter, NnsProposalStatusFilter,
    NnsProposalTopicFilter,
};

pub(in crate::nns::proposals) const NNS_PROPOSAL_ID_ARG: &str = "proposal-id";
pub(in crate::nns::proposals) const NNS_PROPOSAL_BALLOTS_FLAG: &str = "ballots";
pub(in crate::nns::proposals) const NNS_PROPOSAL_VERBOSE_FLAG: &str = "verbose";
pub(in crate::nns::proposals) const NNS_PROPOSAL_LIST_REWARD_STATUS_ARG: &str = "reward-status";
pub(in crate::nns) const NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL: &str = "any";
pub(in crate::nns) const NNS_PROPOSAL_SORT_API_LABEL: &str = "api";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ASC_LABEL: &str = "asc";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DESC_LABEL: &str = "desc";
pub(in crate::nns) const NNS_PROPOSAL_STATUS_ANY_LABEL: &str = "any";
pub(in crate::nns) const NNS_PROPOSAL_TOPIC_ANY_LABEL: &str = "any";

value_enum_with_conversion! {
    ///
    /// NnsProposalListSortArg
    ///
    /// Command-local clap value accepted by `icq nns proposal list --sort`.
    ///

    pub(in crate::nns::proposals) enum NnsProposalListSortArg => NnsProposalListSort {
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
}

value_enum_with_conversion! {
    ///
    /// NnsProposalStatusArg
    ///
    /// Command-local clap value accepted by `icq nns proposal list --status`.
    ///

    pub(in crate::nns::proposals) enum NnsProposalStatusArg => NnsProposalStatusFilter {
        #[default]
        Any,
        Open,
        Rejected,
        Adopted,
        Executed,
        Failed,
    }
}

value_enum_with_conversion! {
    ///
    /// NnsProposalRewardStatusArg
    ///
    /// Command-local clap value accepted by `icq nns proposal list --reward-status`.
    ///

    pub(in crate::nns::proposals) enum NnsProposalRewardStatusArg => NnsProposalRewardStatusFilter {
        #[default]
        Any,
        AcceptVotes,
        ReadyToSettle,
        Settled,
        Ineligible,
    }
}

value_enum_with_conversion! {
    ///
    /// NnsProposalTopicArg
    ///
    /// Command-local clap value accepted by `icq nns proposal list --topic`.
    ///

    pub(in crate::nns::proposals) enum NnsProposalTopicArg => NnsProposalTopicFilter {
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
}
