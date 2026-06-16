#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsNeuronsSort {
    #[default]
    Api,
    Id,
    Stake,
    Maturity,
    Created,
}

impl SnsNeuronsSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Id => "id",
            Self::Stake => "stake",
            Self::Maturity => "maturity",
            Self::Created => "created",
        }
    }

    #[must_use]
    pub const fn uses_cache(self) -> bool {
        !matches!(self, Self::Api)
    }
}

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsListSort {
    #[default]
    Id,
    Name,
}

impl SnsListSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
    }
}
