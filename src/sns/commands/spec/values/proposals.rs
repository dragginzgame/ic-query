//! Module: sns::commands::spec::values::proposals
//!
//! Responsibility: clap value enums for SNS proposal filters.
//! Does not own: live proposal request mapping or report filtering behavior.
//! Boundary: converts CLI filter values into report-model filter values.

use crate::sns::report::{SnsProposalStatusFilter, SnsProposalTopicFilter};
use clap::ValueEnum;

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
