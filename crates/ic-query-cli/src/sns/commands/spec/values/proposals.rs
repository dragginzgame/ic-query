//! Module: sns::commands::spec::values::proposals
//!
//! Responsibility: clap value enums for SNS proposal list options.
//! Does not own: live proposal request mapping or report view behavior.
//! Boundary: converts CLI option values into report-model values.

use crate::cli::value_enum_with_conversion;
use ic_query::sns::{
    SnsProposalEligibilityFilter, SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsSort,
};

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
