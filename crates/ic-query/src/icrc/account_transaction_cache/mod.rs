//! Module: icrc::account_transaction_cache
//!
//! Responsibility: own complete ICRC account-history cache identity, refresh, reads, and views.
//! Does not own: index wire decoding, CLI parsing, or process output.
//! Boundary: publishes only validated API-exhausted snapshots under one atomic lock.

mod attempt;
#[cfg(test)]
mod tests;

use self::attempt::{
    read_refresh_attempt_status, write_complete_attempt, write_failed_attempt,
    write_starting_attempt,
};
pub(super) use super::live::account_transactions::normalize_transaction_cursor;
use super::{
    ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE,
    model::{
        CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionCacheRequest,
        IcrcAccountTransactionCacheStatusReport, IcrcAccountTransactionCacheSummary,
        IcrcAccountTransactionCollectionData, IcrcAccountTransactionCompleteness,
        IcrcAccountTransactionError, IcrcAccountTransactionListReport,
        IcrcAccountTransactionListRequest, IcrcAccountTransactionRefreshReport,
        IcrcAccountTransactionRefreshRequest, IcrcAccountTransactionSnapshot,
        IcrcAccountTransactionSort, IcrcError, normalize_subaccount_hex,
    },
};
use crate::{
    HostCacheError, QueryProgress,
    cache_file::{
        JsonCacheReport, LoadJsonCacheErrorMapper, LoadJsonCacheRequest, load_json_cache,
    },
    freshness::freshness_facts,
    progress::IgnoreQueryProgress,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SNAPSHOT_CACHE_STATUS_INVALID, SNAPSHOT_CACHE_STATUS_OK,
        SnapshotJsonPaths, SnapshotKey, publish_snapshot_with_attempt,
        run_snapshot_refresh_with_attempts, with_locked_snapshot_refresh, write_snapshot_json,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs, parse_utc_timestamp_secs},
};
use candid::Principal;
use sha2::{Digest, Sha256};
use std::{
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

/// Default age after which a complete account-history refresh lock is stale.
pub const DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

const ICRC_ACCOUNT_TRANSACTION_CACHE_SCHEMA_VERSION: u32 = 1;
const ICRC_ACCOUNT_TRANSACTION_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const ICRC_ACCOUNT_TRANSACTION_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
const ICRC_ACCOUNT_TRANSACTION_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
const ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT: &str = "ICRC account transactions";
const ICRC_ACCOUNT_TRANSACTION_CACHE_DOMAIN: &str = "icrc";
const ICRC_ACCOUNT_TRANSACTION_CACHE_COLLECTION: &str = "transactions";
const ICRC_ACCOUNT_TRANSACTION_COMPLETENESS_STATUS: &str = "api_exhausted";
const ICRC_ACCOUNT_TRANSACTION_FETCHED_BY: &str = "ic-query";

impl JsonCacheReport for IcrcAccountTransactionSnapshot {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        MAINNET_NETWORK
    }
}

/// Return the complete account-history cache path for one cache identity.
pub fn icrc_account_transaction_cache_path(
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<PathBuf, IcrcAccountTransactionError> {
    Ok(cache_paths(&normalize_cache_request(request)?).snapshot_path)
}

/// Return the account-history refresh lock path for one cache identity.
pub fn icrc_account_transaction_refresh_lock_path(
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<PathBuf, IcrcAccountTransactionError> {
    Ok(cache_paths(&normalize_cache_request(request)?).refresh_lock_path)
}

/// Return the account-history refresh-attempt path for one cache identity.
pub fn icrc_account_transaction_refresh_attempt_path(
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<PathBuf, IcrcAccountTransactionError> {
    Ok(cache_paths(&normalize_cache_request(request)?).refresh_attempt_path)
}

/// Load and validate one complete snapshot without making a network request.
pub fn load_cached_icrc_account_transactions(
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    let request = normalize_cache_request(request)?;
    let paths = cache_paths(&request);
    load_snapshot_at(&paths.snapshot_path, &request)
}

/// Force a complete live refresh and atomically replace its cache.
pub fn refresh_icrc_account_transaction_cache(
    request: &IcrcAccountTransactionRefreshRequest,
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    refresh_icrc_account_transaction_cache_with_source(request, &super::LiveIcrcSource)
}

/// Force a complete live refresh while emitting structured progress.
pub fn refresh_icrc_account_transaction_cache_with_progress(
    request: &IcrcAccountTransactionRefreshRequest,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    refresh_icrc_account_transaction_cache_with_source_and_progress(
        request,
        &super::LiveIcrcSource,
        progress,
    )
}

/// Refresh with a caller-supplied complete-history source.
pub fn refresh_icrc_account_transaction_cache_with_source(
    request: &IcrcAccountTransactionRefreshRequest,
    source: &dyn super::IcrcAccountTransactionCollectionSource,
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    let mut progress = IgnoreQueryProgress;
    refresh_icrc_account_transaction_cache_with_source_and_progress(request, source, &mut progress)
}

fn refresh_icrc_account_transaction_cache_with_source_and_progress(
    request: &IcrcAccountTransactionRefreshRequest,
    source: &dyn super::IcrcAccountTransactionCollectionSource,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    let request = normalize_refresh_request(request)?;
    let paths = cache_paths(&request.cache);
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
            network: MAINNET_NETWORK,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        cache_operation_error,
        |state| {
            run_snapshot_refresh_with_attempts(
                || write_starting_attempt(&paths.refresh_attempt_path, &request),
                || {
                    let complete =
                        source.fetch_complete_account_transactions(&request, progress)?;
                    publish_complete_snapshot(
                        &request,
                        &paths,
                        state.replaced_existing_snapshot,
                        complete,
                    )
                },
                |error| write_failed_attempt(&paths.refresh_attempt_path, &request, error),
            )
        },
    )
}

/// Load a complete cache, refreshing only when it is missing.
pub fn load_or_refresh_missing_icrc_account_transactions(
    request: &IcrcAccountTransactionRefreshRequest,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    load_or_refresh_missing_icrc_account_transactions_with_source(request, &super::LiveIcrcSource)
}

/// Load a complete cache or use the supplied source only when it is missing.
pub fn load_or_refresh_missing_icrc_account_transactions_with_source(
    request: &IcrcAccountTransactionRefreshRequest,
    source: &dyn super::IcrcAccountTransactionCollectionSource,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    match load_cached_icrc_account_transactions(&request.cache) {
        Ok(snapshot) => Ok(snapshot),
        Err(IcrcAccountTransactionError::Cache(HostCacheError::MissingCache { .. })) => {
            refresh_icrc_account_transaction_cache_with_source(request, source)?;
            load_cached_icrc_account_transactions(&request.cache)
        }
        Err(error) => Err(error),
    }
}

/// Load a complete cache, refreshing when it is missing or stale.
pub fn load_or_refresh_stale_icrc_account_transactions(
    request: &IcrcAccountTransactionRefreshRequest,
    stale_after_seconds: u64,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    load_or_refresh_stale_icrc_account_transactions_with_source(
        request,
        stale_after_seconds,
        &super::LiveIcrcSource,
    )
}

/// Load a complete cache or use the supplied source when it is missing or stale.
pub fn load_or_refresh_stale_icrc_account_transactions_with_source(
    request: &IcrcAccountTransactionRefreshRequest,
    stale_after_seconds: u64,
    source: &dyn super::IcrcAccountTransactionCollectionSource,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    match load_cached_icrc_account_transactions(&request.cache) {
        Ok(snapshot)
            if !snapshot_is_stale(
                &snapshot.snapshot,
                request.now_unix_secs,
                stale_after_seconds,
            ) =>
        {
            Ok(snapshot)
        }
        Ok(_) | Err(IcrcAccountTransactionError::Cache(HostCacheError::MissingCache { .. })) => {
            refresh_icrc_account_transaction_cache_with_source(request, source)?;
            load_cached_icrc_account_transactions(&request.cache)
        }
        Err(error) => Err(error),
    }
}

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
    let mut transactions = snapshot.transactions.clone();
    if request.sort == IcrcAccountTransactionSort::Oldest {
        transactions.reverse();
    }
    transactions.truncate(usize::try_from(request.limit).unwrap_or(usize::MAX));
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
        complete: snapshot.completeness.status == ICRC_ACCOUNT_TRANSACTION_COMPLETENESS_STATUS,
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
    let cache = paths
        .snapshot_path
        .is_file()
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

fn publish_complete_snapshot(
    request: &IcrcAccountTransactionRefreshRequest,
    paths: &SnapshotJsonPaths,
    replaced_existing_cache: bool,
    complete: IcrcAccountTransactionCollectionData,
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    validate_collection_data(&complete)?;
    let collection_started_at = format_utc_timestamp_secs(request.now_unix_secs);
    let collection_completed_at =
        crate::snapshot_cache::current_attempt_timestamp(&collection_started_at);
    let newest_transaction_id = complete
        .transactions
        .first()
        .map(|transaction| transaction.id.clone());
    let oldest_transaction_id = complete
        .transactions
        .last()
        .map(|transaction| transaction.id.clone());
    let snapshot = IcrcAccountTransactionSnapshot {
        schema_version: ICRC_ACCOUNT_TRANSACTION_CACHE_SCHEMA_VERSION,
        source_endpoint: request.cache.source_endpoint.clone(),
        collection_started_at: collection_started_at.clone(),
        collection_completed_at: collection_completed_at.clone(),
        fetched_by: ICRC_ACCOUNT_TRANSACTION_FETCHED_BY.to_string(),
        ledger_canister_id: request.cache.ledger_canister_id.clone(),
        index_canister_id: complete.index_canister_id.clone(),
        account_owner: request.cache.account_owner.clone(),
        subaccount_hex: request.cache.subaccount_hex.clone(),
        balance: complete.balance,
        token_symbol: complete.token_symbol,
        decimals: complete.decimals,
        newest_transaction_id: newest_transaction_id.clone(),
        oldest_transaction_id: oldest_transaction_id.clone(),
        completeness: IcrcAccountTransactionCompleteness {
            status: ICRC_ACCOUNT_TRANSACTION_COMPLETENESS_STATUS.to_string(),
            page_size: request.page_size,
            page_count: complete.page_count,
            row_count: complete.transactions.len(),
            point_in_time_guaranteed: false,
        },
        transactions: complete.transactions,
    };
    validate_snapshot(&paths.snapshot_path, &snapshot, &request.cache)?;
    let transaction_count = snapshot.transactions.len();
    let attempt_finalization_error = publish_snapshot_with_attempt(
        || {
            write_snapshot_json(
                &paths.snapshot_path,
                &snapshot,
                |path, source| {
                    HostCacheError::serialize_cache(
                        ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT,
                        path,
                        source,
                    )
                    .into()
                },
                cache_operation_error,
            )
        },
        || {
            write_complete_attempt(
                &paths.refresh_attempt_path,
                request,
                &snapshot.index_canister_id,
                complete.last_cursor,
                complete.page_count,
                transaction_count,
            )
        },
    )?;

    Ok(IcrcAccountTransactionRefreshReport {
        schema_version: ICRC_ACCOUNT_TRANSACTION_REFRESH_REPORT_SCHEMA_VERSION,
        ledger_canister_id: snapshot.ledger_canister_id,
        index_canister_id: snapshot.index_canister_id,
        account_owner: snapshot.account_owner,
        subaccount_hex: snapshot.subaccount_hex,
        transaction_count,
        newest_transaction_id,
        oldest_transaction_id,
        page_size: snapshot.completeness.page_size,
        page_count: snapshot.completeness.page_count,
        point_in_time_guaranteed: false,
        replaced_existing_cache,
        attempt_finalization_error,
        collection_started_at,
        collection_completed_at,
        source_endpoint: snapshot.source_endpoint,
        fetched_by: snapshot.fetched_by,
        cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        refresh_lock_path: paths.refresh_lock_path.display().to_string(),
    })
}

fn load_snapshot_at(
    path: &Path,
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    let cached = load_json_cache(
        LoadJsonCacheRequest {
            path: path.to_path_buf(),
            network: MAINNET_NETWORK,
            expected_schema_version: ICRC_ACCOUNT_TRANSACTION_CACHE_SCHEMA_VERSION,
        },
        AccountTransactionCacheLoadErrors,
    )?;
    validate_snapshot(path, &cached.report, request)?;
    Ok(CachedIcrcAccountTransactionSnapshot {
        path: cached.path,
        snapshot: cached.report,
    })
}

fn validate_snapshot(
    path: &Path,
    snapshot: &IcrcAccountTransactionSnapshot,
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<(), IcrcAccountTransactionError> {
    let invalid = |reason| IcrcAccountTransactionError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    };
    if snapshot.source_endpoint != request.source_endpoint
        || snapshot.ledger_canister_id != request.ledger_canister_id
        || snapshot.account_owner != request.account_owner
        || snapshot.subaccount_hex != request.subaccount_hex
    {
        return Err(invalid(
            "snapshot identity does not match the requested endpoint, ledger, or account"
                .to_string(),
        ));
    }
    if snapshot.completeness.status != ICRC_ACCOUNT_TRANSACTION_COMPLETENESS_STATUS {
        return Err(invalid(format!(
            "completeness status is {}",
            snapshot.completeness.status
        )));
    }
    if snapshot.completeness.page_size == 0 || snapshot.completeness.page_count == 0 {
        return Err(invalid(
            "completeness page size and page count must be greater than zero".to_string(),
        ));
    }
    if snapshot.completeness.row_count != snapshot.transactions.len() {
        return Err(invalid(format!(
            "completeness row count is {}, actual row count is {}",
            snapshot.completeness.row_count,
            snapshot.transactions.len()
        )));
    }
    if snapshot.completeness.point_in_time_guaranteed {
        return Err(invalid(
            "index account history cannot claim a point-in-time guarantee".to_string(),
        ));
    }
    validate_canonical_transactions(&snapshot.transactions).map_err(invalid)?;
    let newest = snapshot
        .transactions
        .first()
        .map(|transaction| transaction.id.as_str());
    let oldest = snapshot
        .transactions
        .last()
        .map(|transaction| transaction.id.as_str());
    if snapshot.newest_transaction_id.as_deref() != newest
        || snapshot.oldest_transaction_id.as_deref() != oldest
    {
        return Err(invalid(
            "newest or oldest transaction id does not match cached rows".to_string(),
        ));
    }
    if parse_utc_timestamp_secs(&snapshot.collection_started_at).is_none()
        || parse_utc_timestamp_secs(&snapshot.collection_completed_at).is_none()
    {
        return Err(invalid("collection timestamp is invalid".to_string()));
    }
    Principal::from_text(&snapshot.index_canister_id)
        .map_err(|error| invalid(format!("invalid index canister id: {error}")))?;
    Ok(())
}

fn validate_collection_data(
    complete: &IcrcAccountTransactionCollectionData,
) -> Result<(), IcrcAccountTransactionError> {
    if complete.page_count == 0 {
        return Err(IcrcAccountTransactionError::IncompleteCollection {
            pages_fetched: 0,
            rows_fetched: complete.transactions.len(),
            last_cursor: complete.last_cursor.clone(),
            reason: "source returned a complete collection with zero pages".to_string(),
        });
    }
    Principal::from_text(&complete.index_canister_id).map_err(|error| {
        IcrcError::InvalidPrincipal {
            field: "index_canister_id",
            reason: error.to_string(),
        }
    })?;
    let expected_last_cursor = complete
        .transactions
        .last()
        .map(|transaction| transaction.id.as_str());
    if complete.last_cursor.as_deref() != expected_last_cursor {
        return Err(IcrcAccountTransactionError::IncompleteCollection {
            pages_fetched: complete.page_count,
            rows_fetched: complete.transactions.len(),
            last_cursor: complete.last_cursor.clone(),
            reason: "final cursor does not match the oldest collected transaction".to_string(),
        });
    }
    validate_canonical_transactions(&complete.transactions).map_err(|reason| {
        IcrcAccountTransactionError::IncompleteCollection {
            pages_fetched: complete.page_count,
            rows_fetched: complete.transactions.len(),
            last_cursor: complete.last_cursor.clone(),
            reason,
        }
    })
}

fn validate_canonical_transactions(
    transactions: &[super::model::IcrcAccountTransactionRow],
) -> Result<(), String> {
    let mut previous = None;
    for transaction in transactions {
        let normalized =
            normalize_transaction_cursor(&transaction.id).map_err(|error| error.to_string())?;
        if normalized != transaction.id {
            return Err(format!(
                "transaction id {} is not canonical decimal text",
                transaction.id
            ));
        }
        let current = candid::Nat::from_str(&transaction.id)
            .map_err(|error| format!("invalid transaction id {}: {error}", transaction.id))?;
        if let Some(previous) = previous.as_ref()
            && current >= *previous
        {
            return Err("transactions are not unique newest-first rows".to_string());
        }
        previous = Some(current);
    }
    Ok(())
}

fn normalize_cache_request(
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<IcrcAccountTransactionCacheRequest, IcrcAccountTransactionError> {
    if request.source_endpoint.trim().is_empty() {
        return Err(IcrcAccountTransactionError::InvalidSourceEndpoint {
            value: request.source_endpoint.clone(),
            reason: "endpoint must not be empty".to_string(),
        });
    }
    let ledger_canister_id =
        Principal::from_text(&request.ledger_canister_id).map_err(|error| {
            IcrcError::InvalidPrincipal {
                field: "ledger_canister_id",
                reason: error.to_string(),
            }
        })?;
    let account_owner = Principal::from_text(&request.account_owner).map_err(|error| {
        IcrcError::InvalidPrincipal {
            field: "account_owner",
            reason: error.to_string(),
        }
    })?;
    Ok(IcrcAccountTransactionCacheRequest {
        icp_root: request.icp_root.clone(),
        source_endpoint: request.source_endpoint.clone(),
        ledger_canister_id: ledger_canister_id.to_text(),
        account_owner: account_owner.to_text(),
        subaccount_hex: request
            .subaccount_hex
            .as_deref()
            .map(normalize_subaccount_hex)
            .transpose()?,
    })
}

fn normalize_refresh_request(
    request: &IcrcAccountTransactionRefreshRequest,
) -> Result<IcrcAccountTransactionRefreshRequest, IcrcAccountTransactionError> {
    if !(1..=ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE).contains(&request.page_size) {
        return Err(IcrcAccountTransactionError::InvalidPageSize {
            page_size: request.page_size,
            max_page_size: ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE,
        });
    }
    if request.max_pages == Some(0) {
        return Err(IcrcAccountTransactionError::InvalidMaxPages { max_pages: 0 });
    }
    let index_canister_id = request
        .index_canister_id
        .as_deref()
        .map(Principal::from_text)
        .transpose()
        .map_err(|error| IcrcError::InvalidPrincipal {
            field: "index_canister_id",
            reason: error.to_string(),
        })?
        .map(|principal| principal.to_text());
    Ok(IcrcAccountTransactionRefreshRequest {
        cache: normalize_cache_request(&request.cache)?,
        now_unix_secs: request.now_unix_secs,
        index_canister_id,
        page_size: request.page_size,
        max_pages: request.max_pages,
        lock_stale_after_seconds: request.lock_stale_after_seconds,
    })
}

fn cache_paths(request: &IcrcAccountTransactionCacheRequest) -> SnapshotJsonPaths {
    SnapshotJsonPaths::for_key(
        &request.icp_root,
        &SnapshotKey::full(
            ICRC_ACCOUNT_TRANSACTION_CACHE_DOMAIN,
            MAINNET_NETWORK,
            cache_entity(request),
            ICRC_ACCOUNT_TRANSACTION_CACHE_COLLECTION,
        ),
    )
}

fn cache_entity(request: &IcrcAccountTransactionCacheRequest) -> String {
    let mut hasher = Sha256::new();
    for value in [
        request.source_endpoint.as_str(),
        request.ledger_canister_id.as_str(),
        request.account_owner.as_str(),
        request.subaccount_hex.as_deref().unwrap_or("-"),
    ] {
        hasher.update(value.len().to_string().as_bytes());
        hasher.update(b":");
        hasher.update(value.as_bytes());
    }
    format!("account-{}", crate::hex::hex_bytes(&hasher.finalize()))
}

fn load_cache_summary(
    path: &Path,
    request: &IcrcAccountTransactionCacheRequest,
) -> IcrcAccountTransactionCacheSummary {
    match load_snapshot_at(path, request) {
        Ok(cached) => IcrcAccountTransactionCacheSummary {
            cache_status: SNAPSHOT_CACHE_STATUS_OK.to_string(),
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
            cache_status: SNAPSHOT_CACHE_STATUS_INVALID.to_string(),
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

fn snapshot_is_stale(
    snapshot: &IcrcAccountTransactionSnapshot,
    now_unix_secs: u64,
    stale_after_seconds: u64,
) -> bool {
    freshness_facts(
        parse_utc_timestamp_secs(&snapshot.collection_completed_at),
        now_unix_secs,
        stale_after_seconds,
    )
    .stale
}

fn cache_operation_error(source: crate::CacheFileError) -> IcrcAccountTransactionError {
    HostCacheError::operation(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, source).into()
}

struct AccountTransactionCacheLoadErrors;

impl LoadJsonCacheErrorMapper for AccountTransactionCacheLoadErrors {
    type Error = IcrcAccountTransactionError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        HostCacheError::missing_cache(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, path).into()
    }

    fn read_cache(&self, path: PathBuf, source: io::Error) -> Self::Error {
        HostCacheError::read_cache(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, path, source).into()
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        HostCacheError::parse_cache(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, path, source).into()
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        HostCacheError::unsupported_cache_schema_version(
            ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT,
            version,
            expected,
        )
        .into()
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        HostCacheError::network_mismatch(
            ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT,
            requested,
            actual,
        )
        .into()
    }
}
