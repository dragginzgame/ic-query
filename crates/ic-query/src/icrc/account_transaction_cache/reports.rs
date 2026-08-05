//! Module: icrc::account_transaction_cache::reports
//!
//! Responsibility: project complete account-history caches and attempt evidence into reports.
//! Does not own: live collection, refresh publication, cache identity, or strict JSON decoding.
//! Boundary: list and status views remain local-only and never trigger network access.

use super::{
    attempt::read_refresh_attempt_status,
    storage::{
        cache_paths, load_cached_icrc_account_transactions, load_snapshot_at,
        normalize_cache_request,
    },
};
use crate::{
    cache::CacheValidationStatus,
    icrc::model::{
        IcrcAccountTransactionCacheRequest, IcrcAccountTransactionCacheStatusReport,
        IcrcAccountTransactionCacheSummary, IcrcAccountTransactionError,
        IcrcAccountTransactionListReport, IcrcAccountTransactionListRequest,
        IcrcAccountTransactionSort,
    },
};
use std::path::Path;

const ICRC_ACCOUNT_TRANSACTION_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
const ICRC_ACCOUNT_TRANSACTION_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;

/// Build a cache-only transaction list view.
pub fn build_icrc_account_transaction_list_report(
    request: &IcrcAccountTransactionListRequest,
) -> Result<IcrcAccountTransactionListReport, IcrcAccountTransactionError> {
    if request.limit == 0 {
        return Err(IcrcAccountTransactionError::InvalidListLimit {
            limit: request.limit,
        });
    }
    let cached = load_cached_icrc_account_transactions(&request.cache)?;
    let snapshot = cached.snapshot;
    let total_transaction_count = snapshot.transactions.len();
    let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
    let transactions: Vec<_> = match request.sort {
        IcrcAccountTransactionSort::Newest => {
            snapshot.transactions.into_iter().take(limit).collect()
        }
        IcrcAccountTransactionSort::Oldest => snapshot
            .transactions
            .into_iter()
            .rev()
            .take(limit)
            .collect(),
    };
    let returned_transaction_count = transactions.len();

    Ok(IcrcAccountTransactionListReport {
        schema_version: ICRC_ACCOUNT_TRANSACTION_LIST_REPORT_SCHEMA_VERSION,
        ledger_canister_id: snapshot.ledger_canister_id,
        index_canister_id: snapshot.index_canister_id,
        account_owner: snapshot.account_owner,
        subaccount_hex: snapshot.subaccount_hex,
        requested_limit: request.limit,
        sort: request.sort.as_str().to_string(),
        total_transaction_count,
        returned_transaction_count,
        newest_transaction_id: snapshot.newest_transaction_id,
        oldest_transaction_id: snapshot.oldest_transaction_id,
        balance: snapshot.balance,
        token_symbol: snapshot.token_symbol,
        decimals: snapshot.decimals,
        collection_started_at: snapshot.collection_started_at,
        collection_completed_at: snapshot.collection_completed_at,
        source_endpoint: snapshot.source_endpoint,
        fetched_by: snapshot.fetched_by,
        complete: snapshot.completeness.is_api_exhausted(),
        point_in_time_guaranteed: snapshot.completeness.point_in_time_guaranteed,
        page_size: snapshot.completeness.page_size,
        page_count: snapshot.completeness.page_count,
        cache_path: cached.path.display().to_string(),
        transactions,
    })
}

/// Build local cache and latest-attempt status without making a network request.
pub fn build_icrc_account_transaction_cache_status_report(
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<IcrcAccountTransactionCacheStatusReport, IcrcAccountTransactionError> {
    let request = normalize_cache_request(request)?;
    let paths = cache_paths(&request);
    let cache = crate::cache_file::managed_file_exists(&request.cache_root, &paths.snapshot_path)
        .map_err(|source| {
            crate::HostCacheError::operation(
                super::ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT,
                source,
            )
        })?
        .then(|| load_cache_summary(&paths.snapshot_path, &request));
    let latest_attempt = read_refresh_attempt_status(&paths.refresh_attempt_path, &request)?;
    Ok(IcrcAccountTransactionCacheStatusReport {
        schema_version: ICRC_ACCOUNT_TRANSACTION_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        ledger_canister_id: request.ledger_canister_id,
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        source_endpoint: request.source_endpoint,
        found: cache.is_some(),
        cache,
        expected_cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        refresh_lock_path: paths.refresh_lock_path.display().to_string(),
        latest_attempt,
    })
}

fn load_cache_summary(
    path: &Path,
    request: &IcrcAccountTransactionCacheRequest,
) -> IcrcAccountTransactionCacheSummary {
    match load_snapshot_at(path, request) {
        Ok(cached) => IcrcAccountTransactionCacheSummary {
            cache_status: CacheValidationStatus::Valid,
            cache_error: None,
            index_canister_id: Some(cached.snapshot.index_canister_id),
            transaction_count: cached.snapshot.transactions.len(),
            newest_transaction_id: cached.snapshot.newest_transaction_id,
            oldest_transaction_id: cached.snapshot.oldest_transaction_id,
            page_size: cached.snapshot.completeness.page_size,
            page_count: cached.snapshot.completeness.page_count,
            complete: true,
            point_in_time_guaranteed: cached.snapshot.completeness.point_in_time_guaranteed,
            collection_started_at: cached.snapshot.collection_started_at,
            collection_completed_at: cached.snapshot.collection_completed_at,
            cache_path: cached.path.display().to_string(),
        },
        Err(error) => IcrcAccountTransactionCacheSummary {
            cache_status: CacheValidationStatus::Invalid,
            cache_error: Some(error.to_string()),
            index_canister_id: None,
            transaction_count: 0,
            newest_transaction_id: None,
            oldest_transaction_id: None,
            page_size: 0,
            page_count: 0,
            complete: false,
            point_in_time_guaranteed: false,
            collection_started_at: "-".to_string(),
            collection_completed_at: "-".to_string(),
            cache_path: path.display().to_string(),
        },
    }
}
