//! Module: sns::report::model::sorts::proposals
//!
//! Responsibility: SNS proposal report sort and filter models.
//! Does not own: CLI parsing, live proposal request transport, or rendering.
//! Boundary: names report-level view options for proposal lists.

///
/// SnsProposalsSort
///
/// Report-model sort selector for SNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsProposalsSort {
    #[default]
    Api,
    Id,
    Created,
}

impl SnsProposalsSort {
    /// Return the stable label used in text and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Id => "id",
            Self::Created => "created",
        }
    }
}

///
/// SnsProposalStatusFilter
///
/// Report-model status filter for bounded SNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsProposalStatusFilter {
    #[default]
    Any,
    Open,
    Rejected,
    Adopted,
    Executed,
    Failed,
}

impl SnsProposalStatusFilter {
    /// Return the stable label used in text and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Open => "open",
            Self::Rejected => "rejected",
            Self::Adopted => "adopted",
            Self::Executed => "executed",
            Self::Failed => "failed",
        }
    }

    /// Return the SNS governance API status code for concrete filters.
    #[must_use]
    pub const fn governance_status_code(self) -> Option<i32> {
        match self {
            Self::Any => None,
            Self::Open => Some(1),
            Self::Rejected => Some(2),
            Self::Adopted => Some(3),
            Self::Executed => Some(4),
            Self::Failed => Some(5),
        }
    }
}

///
/// SnsProposalTopicFilter
///
/// Report-model topic filter for bounded SNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsProposalTopicFilter {
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

impl SnsProposalTopicFilter {
    /// Return the stable label used in text and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::DaoCommunitySettings => "dao-community-settings",
            Self::SnsFrameworkManagement => "sns-framework-management",
            Self::DappCanisterManagement => "dapp-canister-management",
            Self::ApplicationBusinessLogic => "application-business-logic",
            Self::Governance => "governance",
            Self::TreasuryAssetManagement => "treasury-asset-management",
            Self::CriticalDappOperations => "critical-dapp-operations",
        }
    }
}
