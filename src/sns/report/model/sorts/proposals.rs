//! Module: sns::report::model::sorts::proposals
//!
//! Responsibility: SNS proposal report sort and filter models.
//! Does not own: CLI parsing, live proposal request transport, or rendering.
//! Boundary: names report-level view options for proposal lists.

pub(in crate::sns::report) const SNS_PROPOSAL_STATUS_ADOPTED_CODE: i32 = 3;
pub(in crate::sns::report) const SNS_PROPOSAL_STATUS_EXECUTED_CODE: i32 = 4;
pub(in crate::sns::report) const SNS_PROPOSAL_STATUS_FAILED_CODE: i32 = 5;
pub(in crate::sns::report) const SNS_PROPOSAL_STATUS_OPEN_CODE: i32 = 1;
pub(in crate::sns::report) const SNS_PROPOSAL_STATUS_REJECTED_CODE: i32 = 2;

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
    Status,
    Proposer,
    Title,
    Action,
    Yes,
    No,
    TotalVotes,
    Ballots,
    RejectCost,
    RewardRound,
    Created,
    Decided,
    Executed,
    Failed,
}

impl SnsProposalsSort {
    /// Return the stable label used in text and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Id => "id",
            Self::Status => "status",
            Self::Proposer => "proposer",
            Self::Title => "title",
            Self::Action => "action",
            Self::Yes => "yes",
            Self::No => "no",
            Self::TotalVotes => "total-votes",
            Self::Ballots => "ballots",
            Self::RejectCost => "reject-cost",
            Self::RewardRound => "reward-round",
            Self::Created => "created",
            Self::Decided => "decided",
            Self::Executed => "executed",
            Self::Failed => "failed",
        }
    }

    /// Return the natural default direction for this local sort value.
    #[must_use]
    pub const fn default_direction(self) -> SnsProposalSortDirection {
        match self {
            Self::Status | Self::Proposer | Self::Title | Self::Action => {
                SnsProposalSortDirection::Asc
            }
            _ => SnsProposalSortDirection::Desc,
        }
    }

    /// Return whether this sort applies a local direction.
    #[must_use]
    pub const fn uses_local_direction(self) -> bool {
        !matches!(self, Self::Api)
    }

    /// Return the stable direction label used in text and JSON reports.
    #[must_use]
    pub const fn direction_label(self, direction: SnsProposalSortDirection) -> &'static str {
        match self {
            Self::Api => "none",
            _ => direction.as_str(),
        }
    }
}

///
/// SnsProposalSortDirection
///
/// Report-model direction selector for local SNS proposal sorting.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsProposalSortDirection {
    Asc,
    #[default]
    Desc,
}

impl SnsProposalSortDirection {
    /// Return the stable label used in text and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
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
    Decided,
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
            Self::Decided => "decided",
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
            Self::Any | Self::Decided => None,
            Self::Open => Some(SNS_PROPOSAL_STATUS_OPEN_CODE),
            Self::Rejected => Some(SNS_PROPOSAL_STATUS_REJECTED_CODE),
            Self::Adopted => Some(SNS_PROPOSAL_STATUS_ADOPTED_CODE),
            Self::Executed => Some(SNS_PROPOSAL_STATUS_EXECUTED_CODE),
            Self::Failed => Some(SNS_PROPOSAL_STATUS_FAILED_CODE),
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

    /// Return the row topic label for concrete topic filters.
    #[must_use]
    pub const fn topic_label(self) -> Option<&'static str> {
        match self {
            Self::Any => None,
            Self::DaoCommunitySettings => Some(Self::DaoCommunitySettings.as_str()),
            Self::SnsFrameworkManagement => Some(Self::SnsFrameworkManagement.as_str()),
            Self::DappCanisterManagement => Some(Self::DappCanisterManagement.as_str()),
            Self::ApplicationBusinessLogic => Some(Self::ApplicationBusinessLogic.as_str()),
            Self::Governance => Some(Self::Governance.as_str()),
            Self::TreasuryAssetManagement => Some(Self::TreasuryAssetManagement.as_str()),
            Self::CriticalDappOperations => Some(Self::CriticalDappOperations.as_str()),
        }
    }
}
