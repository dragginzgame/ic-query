//! Module: sns::commands::spec::values::proposals
//!
//! Responsibility: clap value enums for SNS proposal list options.
//! Does not own: live proposal request mapping or report view behavior.
//! Boundary: converts CLI option values into report-model values.

use crate::sns::report::{SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsSort};
use clap::ValueEnum;

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
