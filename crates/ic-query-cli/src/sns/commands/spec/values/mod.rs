//! Clap values for SNS list, neuron and proposal commands.

use crate::cli::value_enum_with_conversion;
use ic_query::sns::{
    SnsListSort, SnsProposalEligibilityFilter, SnsProposalStatusFilter, SnsProposalTopicFilter,
    SnsProposalsSort,
};

value_enum_with_conversion! {
    ///
    /// SnsListSortArg
    ///
    /// Command-local clap value accepted by `icq sns list --sort`.
    ///

    pub(in crate::sns::commands) enum SnsListSortArg => SnsListSort {
        #[default]
        Id,
        Name,
    }
}

///
/// SnsNeuronsSortArg
///
/// CLI selector for live API ordering or a locally ordered complete cache.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, clap::ValueEnum)]
pub(in crate::sns::commands) enum SnsNeuronsSortArg {
    #[default]
    Api,
    Id,
    Stake,
    Maturity,
    Created,
}

value_enum_with_conversion! {
    ///
    /// SnsProposalsSortArg
    ///
    /// Command-local clap value accepted by `icq sns proposal list <SNS> --sort`.
    ///

    pub(in crate::sns::commands) enum SnsProposalsSortArg => SnsProposalsSort {
        #[default]
        Api,
        Id,
        Status,
        Topic,
        Proposer,
        Title,
        Action,
        ActionId,
        Yes,
        No,
        TotalVotes,
        TallyTime,
        Ballots,
        Eligible,
        RejectCost,
        RewardRound,
        RewardEnd,
        Created,
        Decided,
        Executed,
        Failed,
    }
}

value_enum_with_conversion! {
    ///
    /// SnsProposalEligibilityArg
    ///
    /// Command-local clap value accepted by `icq sns proposal list <SNS> --eligible`.
    ///

    pub(in crate::sns::commands) enum SnsProposalEligibilityArg => SnsProposalEligibilityFilter {
        #[default]
        Any,
        Yes,
        No,
    }
}

value_enum_with_conversion! {
    ///
    /// SnsProposalStatusArg
    ///
    /// Command-local clap value accepted by `icq sns proposal list <SNS> --status`.
    ///

    pub(in crate::sns::commands) enum SnsProposalStatusArg => SnsProposalStatusFilter {
        #[default]
        Any,
        Open,
        Decided,
        Rejected,
        Adopted,
        Executed,
        Failed,
    }
}

value_enum_with_conversion! {
    ///
    /// SnsProposalTopicArg
    ///
    /// Command-local clap value accepted by `icq sns proposal list <SNS> --topic`.
    ///

    pub(in crate::sns::commands) enum SnsProposalTopicArg => SnsProposalTopicFilter {
        #[default]
        Any,
        DaoCommunitySettings,
        SnsFrameworkManagement,
        DappCanisterManagement,
        ApplicationBusinessLogic,
        Governance,
        TreasuryAssetManagement,
        CriticalDappOperations,
    }
}
