//! Module: icrc
//!
//! Responsibility: expose generic ICRC ledger request and report APIs.
//! Does not own: SNS lookup, NNS registry cache behavior, or release flow.
//! Boundary: exposes live read-only token metadata, account balance, allowance,
//! index discovery, account and ledger transaction history, block type, and archive reports.

#[cfg(feature = "icrc-host")]
mod account_transaction_cache;
#[cfg(feature = "icrc-host")]
pub(crate) mod ledger;
#[cfg(feature = "icrc-host")]
mod live;
mod model;
mod text;

pub const DEFAULT_ICRC_SOURCE_ENDPOINT: &str = "https://icp-api.io";

#[cfg(feature = "icrc-host")]
pub use account_transaction_cache::{
    DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS,
    build_icrc_account_transaction_cache_status_report, build_icrc_account_transaction_list_report,
    icrc_account_transaction_cache_path, icrc_account_transaction_refresh_attempt_path,
    icrc_account_transaction_refresh_lock_path, load_cached_icrc_account_transactions,
    load_or_refresh_missing_icrc_account_transactions,
    load_or_refresh_missing_icrc_account_transactions_with_source,
    load_or_refresh_stale_icrc_account_transactions,
    load_or_refresh_stale_icrc_account_transactions_with_source,
    refresh_icrc_account_transaction_cache, refresh_icrc_account_transaction_cache_with_progress,
    refresh_icrc_account_transaction_cache_with_source,
};
#[cfg(feature = "icrc-host")]
pub use live::{
    ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE, IcrcAccountTransactionCollectionSource,
    IcrcAccountTransactionPageSource, IcrcAllowanceSource, IcrcArchivesSource, IcrcBalanceSource,
    IcrcBlockTypesSource, IcrcCapabilitiesSource, IcrcIndexSource, IcrcTipCertificateSource,
    IcrcTokenSource, IcrcTransactionsSource, LiveIcrcSource,
    build_icrc_account_transaction_page_report,
    build_icrc_account_transaction_page_report_with_source, build_icrc_allowance_report,
    build_icrc_allowance_report_with_source, build_icrc_archives_report,
    build_icrc_archives_report_with_source, build_icrc_balance_report,
    build_icrc_balance_report_with_source, build_icrc_block_types_report,
    build_icrc_block_types_report_with_source, build_icrc_capabilities_report,
    build_icrc_capabilities_report_with_source, build_icrc_index_report,
    build_icrc_index_report_with_source, build_icrc_tip_certificate_report,
    build_icrc_tip_certificate_report_with_source, build_icrc_token_report,
    build_icrc_token_report_with_source, build_icrc_transactions_report,
    build_icrc_transactions_report_with_source,
};
pub use model::normalize_subaccount_hex;
#[cfg(feature = "icrc-host")]
pub use model::{
    CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionCollectionData,
    IcrcAccountTransactionPageData, IcrcAllowanceData, IcrcArchivesData, IcrcBalanceData,
    IcrcBlockTypesData, IcrcCapabilitiesData, IcrcIndexData, IcrcTipCertificateData, IcrcTokenData,
    IcrcTransactionsData,
};
pub use model::{
    IcrcAccountRow, IcrcAccountTransactionCacheRequest, IcrcAccountTransactionCacheStatusReport,
    IcrcAccountTransactionCacheSummary, IcrcAccountTransactionError,
    IcrcAccountTransactionListReport, IcrcAccountTransactionListRequest,
    IcrcAccountTransactionPageReport, IcrcAccountTransactionPageRequest,
    IcrcAccountTransactionRefreshAttemptStatus, IcrcAccountTransactionRefreshReport,
    IcrcAccountTransactionRefreshRequest, IcrcAccountTransactionRow,
    IcrcAccountTransactionSnapshot, IcrcAccountTransactionSort, IcrcAllowanceReport,
    IcrcAllowanceRequest, IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow,
    IcrcArchivedRangeRow, IcrcArchivesReport, IcrcArchivesRequest, IcrcBalanceReport,
    IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesReport, IcrcCapabilitiesReport,
    IcrcCapabilityRow, IcrcCapabilityStatus, IcrcError, IcrcFollowedArchiveBlockRow,
    IcrcIndexReport, IcrcLedgerRequest, IcrcMetadataValueKind, IcrcTipCertificateReport,
    IcrcTokenMetadataRow, IcrcTokenReport, IcrcTokenStandardRow, IcrcTransactionBlockRow,
    IcrcTransactionsReport, IcrcTransactionsRequest,
};
pub use text::{
    icrc_account_transaction_cache_status_report_text, icrc_account_transaction_list_report_text,
    icrc_account_transaction_page_report_text, icrc_account_transaction_refresh_report_text,
    icrc_allowance_report_text, icrc_archives_report_text, icrc_balance_report_text,
    icrc_block_types_report_text, icrc_capabilities_report_text, icrc_index_report_text,
    icrc_tip_certificate_report_text, icrc_token_report_text, icrc_transactions_report_text,
};

#[cfg(all(test, feature = "icrc-host"))]
mod tests;
