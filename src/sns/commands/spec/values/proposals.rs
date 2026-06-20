//! Module: sns::commands::spec::values::proposals
//!
//! Responsibility: clap value enums for SNS proposal list options.
//! Does not own: live proposal request mapping or report view behavior.
//! Boundary: converts CLI option values into report-model values.

use crate::sns::report::{SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsSort};
use clap::ValueEnum;

pub(in crate::sns::commands) const SNS_PROPOSALS_SORT_VALUE_NAME: &str = concat!(
    "api|id|status|topic|proposer|title|action|action-id|yes|no|total-votes|",
    "tally-time|ballots|eligible|reject-cost|reward-round|reward-end|created|decided|",
    "executed|failed"
);
pub(in crate::sns::commands) const SNS_PROPOSALS_LOCAL_SORT_VALUE_NAME: &str = concat!(
    "id|status|topic|proposer|title|action|action-id|yes|no|total-votes|",
    "tally-time|ballots|eligible|reject-cost|reward-round|reward-end|created|decided|",
    "executed|failed"
);

///
/// SnsProposalsSortArg
///
/// Command-local clap value accepted by `icq sns proposals --sort`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::sns::commands) enum SnsProposalsSortArg {
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

impl From<SnsProposalsSortArg> for SnsProposalsSort {
    fn from(value: SnsProposalsSortArg) -> Self {
        match value {
            SnsProposalsSortArg::Api => Self::Api,
            SnsProposalsSortArg::Id => Self::Id,
            SnsProposalsSortArg::Status => Self::Status,
            SnsProposalsSortArg::Topic => Self::Topic,
            SnsProposalsSortArg::Proposer => Self::Proposer,
            SnsProposalsSortArg::Title => Self::Title,
            SnsProposalsSortArg::Action => Self::Action,
            SnsProposalsSortArg::ActionId => Self::ActionId,
            SnsProposalsSortArg::Yes => Self::Yes,
            SnsProposalsSortArg::No => Self::No,
            SnsProposalsSortArg::TotalVotes => Self::TotalVotes,
            SnsProposalsSortArg::TallyTime => Self::TallyTime,
            SnsProposalsSortArg::Ballots => Self::Ballots,
            SnsProposalsSortArg::Eligible => Self::Eligible,
            SnsProposalsSortArg::RejectCost => Self::RejectCost,
            SnsProposalsSortArg::RewardRound => Self::RewardRound,
            SnsProposalsSortArg::RewardEnd => Self::RewardEnd,
            SnsProposalsSortArg::Created => Self::Created,
            SnsProposalsSortArg::Decided => Self::Decided,
            SnsProposalsSortArg::Executed => Self::Executed,
            SnsProposalsSortArg::Failed => Self::Failed,
        }
    }
}

///
/// SnsProposalStatusArg
///
/// Command-local clap value accepted by `icq sns proposals --status`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::sns::commands) enum SnsProposalStatusArg {
    #[default]
    Any,
    Open,
    Decided,
    Rejected,
    Adopted,
    Executed,
    Failed,
}

impl From<SnsProposalStatusArg> for SnsProposalStatusFilter {
    fn from(value: SnsProposalStatusArg) -> Self {
        match value {
            SnsProposalStatusArg::Any => Self::Any,
            SnsProposalStatusArg::Open => Self::Open,
            SnsProposalStatusArg::Decided => Self::Decided,
            SnsProposalStatusArg::Rejected => Self::Rejected,
            SnsProposalStatusArg::Adopted => Self::Adopted,
            SnsProposalStatusArg::Executed => Self::Executed,
            SnsProposalStatusArg::Failed => Self::Failed,
        }
    }
}

///
/// SnsProposalTopicArg
///
/// Command-local clap value accepted by `icq sns proposals --topic`.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(in crate::sns::commands) enum SnsProposalTopicArg {
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

impl From<SnsProposalTopicArg> for SnsProposalTopicFilter {
    fn from(value: SnsProposalTopicArg) -> Self {
        match value {
            SnsProposalTopicArg::Any => Self::Any,
            SnsProposalTopicArg::DaoCommunitySettings => Self::DaoCommunitySettings,
            SnsProposalTopicArg::SnsFrameworkManagement => Self::SnsFrameworkManagement,
            SnsProposalTopicArg::DappCanisterManagement => Self::DappCanisterManagement,
            SnsProposalTopicArg::ApplicationBusinessLogic => Self::ApplicationBusinessLogic,
            SnsProposalTopicArg::Governance => Self::Governance,
            SnsProposalTopicArg::TreasuryAssetManagement => Self::TreasuryAssetManagement,
            SnsProposalTopicArg::CriticalDappOperations => Self::CriticalDappOperations,
        }
    }
}
