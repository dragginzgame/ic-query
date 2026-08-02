//! Module: icrc::account_transaction_cache
//!
//! Responsibility: expose complete ICRC account-history cache operations behind one facade.
//! Does not own: index wire decoding, CLI parsing, or process output.
//! Boundary: keeps storage, refresh publication, attempt evidence, and views distinct.

mod attempt;
mod refresh;
mod reports;
mod storage;
#[cfg(test)]
mod tests;

pub use refresh::{
    load_or_refresh_missing_icrc_account_transactions,
    load_or_refresh_missing_icrc_account_transactions_with_source,
    load_or_refresh_stale_icrc_account_transactions,
    load_or_refresh_stale_icrc_account_transactions_with_source,
    refresh_icrc_account_transaction_cache, refresh_icrc_account_transaction_cache_with_progress,
    refresh_icrc_account_transaction_cache_with_source,
};
pub use reports::{
    build_icrc_account_transaction_cache_status_report, build_icrc_account_transaction_list_report,
};
pub use storage::{
    icrc_account_transaction_cache_path, icrc_account_transaction_refresh_attempt_path,
    icrc_account_transaction_refresh_lock_path, load_cached_icrc_account_transactions,
};

/// Default age after which a complete account-history refresh lock is stale.
pub const DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

const ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT: &str = "ICRC account transactions";
const ICRC_ACCOUNT_TRANSACTION_COMPLETENESS_STATUS: &str = "api_exhausted";
const ICRC_ACCOUNT_TRANSACTION_FETCHED_BY: &str = "ic-query";
