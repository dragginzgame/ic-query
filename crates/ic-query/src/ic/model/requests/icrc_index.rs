//! Module: ic::model::requests::icrc_index
//!
//! Responsibility: bounded official ICRC account and holder index request contracts.
//! Does not own: native ledger queries, transport, source validation, or reports.
//! Boundary: exposes one exact account lookup or one explicitly bounded cursor page.

use super::IcIcrcAnalyticsRequest;
use serde::Serialize;
use std::fmt;

///
/// IcIcrcAccountSort
///
/// Sort order accepted by the official ICRC account index.
///

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub enum IcIcrcAccountSort {
    /// Account id in ascending order.
    #[serde(rename = "id")]
    Id,
    /// Account id in descending order.
    #[serde(rename = "-id")]
    IdDescending,
    /// Raw balance in ascending order.
    #[serde(rename = "balance")]
    Balance,
    /// Raw balance in descending order.
    #[serde(rename = "-balance")]
    BalanceDescending,
    /// Total transaction count in ascending order.
    #[serde(rename = "total_transactions")]
    TotalTransactions,
    /// Total transaction count in descending order.
    #[serde(rename = "-total_transactions")]
    TotalTransactionsDescending,
    /// Creation timestamp in ascending order.
    #[serde(rename = "created_timestamp")]
    CreatedTimestamp,
    /// Creation timestamp in descending order.
    #[serde(rename = "-created_timestamp")]
    CreatedTimestampDescending,
    /// Owner principal in ascending order.
    #[serde(rename = "owner")]
    Owner,
    /// Owner principal in descending order.
    #[serde(rename = "-owner")]
    OwnerDescending,
}

impl IcIcrcAccountSort {
    /// Return the exact official API query value.
    #[must_use]
    pub const fn as_api_value(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::IdDescending => "-id",
            Self::Balance => "balance",
            Self::BalanceDescending => "-balance",
            Self::TotalTransactions => "total_transactions",
            Self::TotalTransactionsDescending => "-total_transactions",
            Self::CreatedTimestamp => "created_timestamp",
            Self::CreatedTimestampDescending => "-created_timestamp",
            Self::Owner => "owner",
            Self::OwnerDescending => "-owner",
        }
    }
}

impl fmt::Display for IcIcrcAccountSort {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_api_value())
    }
}

///
/// IcIcrcHolderSort
///
/// Sort order accepted by the official ICRC holder index.
///

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub enum IcIcrcHolderSort {
    /// Raw aggregate balance in ascending order.
    #[serde(rename = "balance")]
    Balance,
    /// Raw aggregate balance in descending order.
    #[serde(rename = "-balance")]
    BalanceDescending,
    /// Total transaction count in ascending order.
    #[serde(rename = "total_transactions")]
    TotalTransactions,
    /// Total transaction count in descending order.
    #[serde(rename = "-total_transactions")]
    TotalTransactionsDescending,
    /// Earliest account creation timestamp in ascending order.
    #[serde(rename = "created_timestamp")]
    CreatedTimestamp,
    /// Earliest account creation timestamp in descending order.
    #[serde(rename = "-created_timestamp")]
    CreatedTimestampDescending,
    /// Holder principal in ascending order.
    #[serde(rename = "principal")]
    Principal,
    /// Holder principal in descending order.
    #[serde(rename = "-principal")]
    PrincipalDescending,
}

impl IcIcrcHolderSort {
    /// Return the exact official API query value.
    #[must_use]
    pub const fn as_api_value(self) -> &'static str {
        match self {
            Self::Balance => "balance",
            Self::BalanceDescending => "-balance",
            Self::TotalTransactions => "total_transactions",
            Self::TotalTransactionsDescending => "-total_transactions",
            Self::CreatedTimestamp => "created_timestamp",
            Self::CreatedTimestampDescending => "-created_timestamp",
            Self::Principal => "principal",
            Self::PrincipalDescending => "-principal",
        }
    }
}

impl fmt::Display for IcIcrcHolderSort {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_api_value())
    }
}

///
/// IcIcrcAccountListQuery
///
/// One explicitly bounded account-index page query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcAccountListQuery {
    /// Optional canonical owner principal filter.
    pub owner: Option<String>,
    /// Opaque exclusive forward cursor returned by an earlier page.
    pub after: Option<String>,
    /// Opaque exclusive backward cursor returned by an earlier page.
    pub before: Option<String>,
    /// Maximum rows requested from the official API.
    pub limit: u16,
    /// Stable upstream sort order applied to the page.
    pub sort_by: IcIcrcAccountSort,
}

impl IcIcrcAccountListQuery {
    /// Construct one account-index page query.
    #[must_use]
    pub const fn new(limit: u16, sort_by: IcIcrcAccountSort) -> Self {
        Self {
            owner: None,
            after: None,
            before: None,
            limit,
            sort_by,
        }
    }

    /// Restrict the page to one owner principal.
    #[must_use]
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    /// Continue after one opaque cursor.
    #[must_use]
    pub fn with_after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    /// Continue before one opaque cursor.
    #[must_use]
    pub fn with_before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }
}

///
/// IcIcrcHolderListQuery
///
/// One explicitly bounded holder-index page query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcIcrcHolderListQuery {
    /// Opaque exclusive forward cursor returned by an earlier page.
    pub after: Option<String>,
    /// Opaque exclusive backward cursor returned by an earlier page.
    pub before: Option<String>,
    /// Maximum rows requested from the official API.
    pub limit: u16,
    /// Stable upstream sort order applied to the page.
    pub sort_by: IcIcrcHolderSort,
}

impl IcIcrcHolderListQuery {
    /// Construct one holder-index page query.
    #[must_use]
    pub const fn new(limit: u16, sort_by: IcIcrcHolderSort) -> Self {
        Self {
            after: None,
            before: None,
            limit,
            sort_by,
        }
    }

    /// Continue after one opaque cursor.
    #[must_use]
    pub fn with_after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    /// Continue before one opaque cursor.
    #[must_use]
    pub fn with_before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }
}

///
/// IcIcrcAccountListRequest
///
/// Request accepted by the bounded official ICRC account-list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcAccountListRequest {
    /// Shared analytics endpoint, collection time, and ledger identity.
    pub analytics: IcIcrcAnalyticsRequest,
    /// Explicitly bounded account page query.
    pub query: IcIcrcAccountListQuery,
}

impl IcIcrcAccountListRequest {
    /// Construct one bounded live account-list request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        query: IcIcrcAccountListQuery,
    ) -> Self {
        Self {
            analytics: IcIcrcAnalyticsRequest::new(
                source_endpoint,
                now_unix_secs,
                ledger_canister_id,
            ),
            query,
        }
    }
}

///
/// IcIcrcAccountInfoRequest
///
/// Request accepted by the exact official ICRC account-detail report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcAccountInfoRequest {
    /// Shared analytics endpoint, collection time, and ledger identity.
    pub analytics: IcIcrcAnalyticsRequest,
    /// Exact opaque account id requested from the index.
    pub account_id: String,
}

impl IcIcrcAccountInfoRequest {
    /// Construct one exact live account-detail request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        account_id: impl Into<String>,
    ) -> Self {
        Self {
            analytics: IcIcrcAnalyticsRequest::new(
                source_endpoint,
                now_unix_secs,
                ledger_canister_id,
            ),
            account_id: account_id.into(),
        }
    }
}

///
/// IcIcrcHolderListRequest
///
/// Request accepted by the bounded official ICRC holder-list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcIcrcHolderListRequest {
    /// Shared analytics endpoint, collection time, and ledger identity.
    pub analytics: IcIcrcAnalyticsRequest,
    /// Explicitly bounded holder page query.
    pub query: IcIcrcHolderListQuery,
}

impl IcIcrcHolderListRequest {
    /// Construct one bounded live holder-list request.
    #[must_use]
    pub fn new(
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        ledger_canister_id: impl Into<String>,
        query: IcIcrcHolderListQuery,
    ) -> Self {
        Self {
            analytics: IcIcrcAnalyticsRequest::new(
                source_endpoint,
                now_unix_secs,
                ledger_canister_id,
            ),
            query,
        }
    }
}
