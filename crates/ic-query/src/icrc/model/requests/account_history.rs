//! Module: icrc::model::requests::account_history
//!
//! Responsibility: ICRC account-history page, cache, refresh, and list requests.
//! Does not own: ledger-wide history, live transport, cache mechanics, or reports.
//! Boundary: separates stable collection identity from pagination and local view options.

use super::IcrcLedgerRequest;

use std::path::PathBuf;

///
/// IcrcAccountTransactionPageRequest
///
/// Request accepted by the live ICRC index account-transaction page builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcAccountTransactionPageRequest {
    /// Shared ledger identity, explicit endpoint and caller collection clock.
    pub ledger: IcrcLedgerRequest,
    /// Optional explicit index canister; otherwise ICRC-106 discovery is used.
    pub index_canister_id: Option<String>,
    /// Account owner principal.
    pub account_owner: String,
    /// Optional normalized 32-byte subaccount hex.
    pub subaccount_hex: Option<String>,
    /// Optional exclusive block-index cursor for backward pagination.
    pub start: Option<String>,
    /// Maximum number of account transactions to request.
    pub limit: u32,
}

impl IcrcAccountTransactionPageRequest {
    /// Constructs an account-history request that discovers the index through the ledger.
    #[must_use]
    pub fn new(ledger: IcrcLedgerRequest, account_owner: impl Into<String>, limit: u32) -> Self {
        Self {
            ledger,
            index_canister_id: None,
            account_owner: account_owner.into(),
            subaccount_hex: None,
            start: None,
            limit,
        }
    }
}

///
/// IcrcAccountTransactionCacheRequest
///
/// Stable account-history cache identity independent of page and view options.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcAccountTransactionCacheRequest {
    /// Root directory containing the shared cache.
    pub cache_root: PathBuf,
    /// IC API endpoint whose indexed history is cached.
    pub source_endpoint: String,
    /// Ledger canister whose account history is cached.
    pub ledger_canister_id: String,
    /// Account owner principal.
    pub account_owner: String,
    /// Optional normalized 32-byte subaccount hex.
    pub subaccount_hex: Option<String>,
}

impl IcrcAccountTransactionCacheRequest {
    /// Constructs a cache identity for the default subaccount.
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        source_endpoint: impl Into<String>,
        ledger_canister_id: impl Into<String>,
        account_owner: impl Into<String>,
    ) -> Self {
        Self {
            cache_root: cache_root.into(),
            source_endpoint: source_endpoint.into(),
            ledger_canister_id: ledger_canister_id.into(),
            account_owner: account_owner.into(),
            subaccount_hex: None,
        }
    }
}

///
/// IcrcAccountTransactionRefreshRequest
///
/// Request for a forced complete account-history refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcAccountTransactionRefreshRequest {
    /// Stable cache identity.
    pub cache: IcrcAccountTransactionCacheRequest,
    /// Collection start clock in Unix seconds; completion adds monotonic elapsed time.
    pub now_unix_secs: u64,
    /// Optional explicit index canister; otherwise ICRC-106 discovery is used.
    pub index_canister_id: Option<String>,
    /// Maximum transactions requested per index page.
    pub page_size: u32,
    /// Optional diagnostic bound that fails rather than publishing a partial cache.
    pub max_pages: Option<u32>,
}

impl IcrcAccountTransactionRefreshRequest {
    /// Constructs a complete refresh request.
    #[must_use]
    pub const fn new(
        cache: IcrcAccountTransactionCacheRequest,
        now_unix_secs: u64,
        page_size: u32,
    ) -> Self {
        Self {
            cache,
            now_unix_secs,
            index_canister_id: None,
            page_size,
            max_pages: None,
        }
    }
}

///
/// IcrcAccountTransactionSort
///
/// Supported cached account-history ordering.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IcrcAccountTransactionSort {
    /// Highest transaction id first.
    Newest,
    /// Lowest transaction id first.
    Oldest,
}

impl IcrcAccountTransactionSort {
    /// All supported choices in their public presentation order.
    pub const ALL: &'static [Self] = &[Self::Newest, Self::Oldest];

    /// Stable JSON/text name for this ordering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Newest => "newest",
            Self::Oldest => "oldest",
        }
    }
}

///
/// IcrcAccountTransactionListRequest
///
/// Cache-only account-history list view.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcrcAccountTransactionListRequest {
    /// Stable cache identity.
    pub cache: IcrcAccountTransactionCacheRequest,
    /// Maximum cached rows returned by this view.
    pub limit: u32,
    /// Requested cached-row ordering.
    pub sort: IcrcAccountTransactionSort,
}

impl IcrcAccountTransactionListRequest {
    /// Constructs a newest-first cached list view.
    #[must_use]
    pub const fn new(cache: IcrcAccountTransactionCacheRequest, limit: u32) -> Self {
        Self {
            cache,
            limit,
            sort: IcrcAccountTransactionSort::Newest,
        }
    }
}
