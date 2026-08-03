//! Module: icrc::account_transaction_cache::refresh
//!
//! Responsibility: orchestrate complete account-history refreshes and atomic publication.
//! Does not own: index collection, cache identity, strict cache reads, or report views.
//! Boundary: publishes only validated API-exhausted collections under one refresh lock.

use super::{
    ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, ICRC_ACCOUNT_TRANSACTION_FETCHED_BY,
    attempt::{write_complete_attempt, write_failed_attempt, write_starting_attempt},
    storage::{
        ICRC_ACCOUNT_TRANSACTION_CACHE_SCHEMA_VERSION, cache_paths,
        load_cached_icrc_account_transactions, normalize_cache_request, snapshot_is_stale,
        validate_snapshot,
    },
};
use crate::{
    HostCacheError, QueryProgress,
    cache::CacheCollectionCompleteness,
    cache_file::{load_or_refresh_missing_cache, load_or_refresh_stale_cache},
    icrc::{
        ledger::principal_from_text,
        live::{
            ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE, IcrcAccountTransactionCollectionSource,
            LiveIcrcSource, account_transactions::validate_canonical_account_transactions,
        },
        model::{
            CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionCollectionData,
            IcrcAccountTransactionError, IcrcAccountTransactionRefreshReport,
            IcrcAccountTransactionRefreshRequest, IcrcAccountTransactionSnapshot, IcrcError,
        },
    },
    progress::IgnoreQueryProgress,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SnapshotJsonPaths, publish_snapshot_with_attempt,
        run_snapshot_refresh_with_attempts, with_locked_snapshot_refresh, write_snapshot_json,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use std::path::PathBuf;

const ICRC_ACCOUNT_TRANSACTION_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;

/// Force a complete live refresh and atomically replace its cache.
pub fn refresh_icrc_account_transaction_cache(
    request: &IcrcAccountTransactionRefreshRequest,
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    refresh_icrc_account_transaction_cache_with_source(request, &LiveIcrcSource)
}

/// Force a complete live refresh while emitting structured progress.
pub fn refresh_icrc_account_transaction_cache_with_progress(
    request: &IcrcAccountTransactionRefreshRequest,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    refresh_icrc_account_transaction_cache_with_source_and_progress(
        request,
        &LiveIcrcSource,
        progress,
    )
}

/// Refresh with a caller-supplied complete-history source.
pub fn refresh_icrc_account_transaction_cache_with_source(
    request: &IcrcAccountTransactionRefreshRequest,
    source: &dyn IcrcAccountTransactionCollectionSource,
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    let mut progress = IgnoreQueryProgress;
    refresh_icrc_account_transaction_cache_with_source_and_progress(request, source, &mut progress)
}

fn refresh_icrc_account_transaction_cache_with_source_and_progress(
    request: &IcrcAccountTransactionRefreshRequest,
    source: &dyn IcrcAccountTransactionCollectionSource,
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
    load_or_refresh_missing_icrc_account_transactions_with_source(request, &LiveIcrcSource)
}

/// Load a complete cache or use the supplied source only when it is missing.
pub fn load_or_refresh_missing_icrc_account_transactions_with_source(
    request: &IcrcAccountTransactionRefreshRequest,
    source: &dyn IcrcAccountTransactionCollectionSource,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    load_or_refresh_missing_cache(
        || load_cached_icrc_account_transactions(&request.cache),
        missing_account_transaction_cache_path,
        |_| {
            refresh_icrc_account_transaction_cache_with_source(request, source)?;
            Ok(())
        },
    )
}

/// Load a complete cache, refreshing when it is missing or stale.
pub fn load_or_refresh_stale_icrc_account_transactions(
    request: &IcrcAccountTransactionRefreshRequest,
    stale_after_seconds: u64,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    load_or_refresh_stale_icrc_account_transactions_with_source(
        request,
        stale_after_seconds,
        &LiveIcrcSource,
    )
}

/// Load a complete cache or use the supplied source when it is missing or stale.
pub fn load_or_refresh_stale_icrc_account_transactions_with_source(
    request: &IcrcAccountTransactionRefreshRequest,
    stale_after_seconds: u64,
    source: &dyn IcrcAccountTransactionCollectionSource,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    load_or_refresh_stale_cache(
        || load_cached_icrc_account_transactions(&request.cache),
        |snapshot| {
            snapshot_is_stale(
                &snapshot.snapshot,
                request.now_unix_secs,
                stale_after_seconds,
            )
        },
        missing_account_transaction_cache_path,
        |_| {
            refresh_icrc_account_transaction_cache_with_source(request, source)?;
            Ok(())
        },
    )
}

fn publish_complete_snapshot(
    request: &IcrcAccountTransactionRefreshRequest,
    paths: &SnapshotJsonPaths,
    replaced_existing_cache: bool,
    complete: IcrcAccountTransactionCollectionData,
) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError> {
    validate_collection_data(request, &complete)?;
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
        completeness: CacheCollectionCompleteness::api_exhausted(
            request.page_size,
            complete.page_count,
            complete.transactions.len(),
            false,
        ),
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

fn validate_collection_data(
    request: &IcrcAccountTransactionRefreshRequest,
    complete: &IcrcAccountTransactionCollectionData,
) -> Result<(), IcrcAccountTransactionError> {
    if complete.page_count == 0 {
        return Err(incomplete_collection_error(
            complete,
            "source returned a complete collection with zero pages",
        ));
    }
    let actual_index =
        principal_from_text::<IcrcError>(&complete.index_canister_id, "index_canister_id")?;
    if let Some(expected_index) = request.index_canister_id.as_deref()
        && expected_index != actual_index.to_text()
    {
        return Err(IcrcAccountTransactionError::CollectionIndexMismatch {
            expected_index_canister_id: expected_index.to_string(),
            actual_index_canister_id: actual_index.to_text(),
        });
    }
    let expected_last_cursor = complete
        .transactions
        .last()
        .map(|transaction| transaction.id.as_str());
    if complete.last_cursor.as_deref() != expected_last_cursor {
        return Err(incomplete_collection_error(
            complete,
            "final cursor does not match the oldest collected transaction",
        ));
    }
    validate_canonical_account_transactions(&complete.transactions)
        .map_err(|reason| incomplete_collection_error(complete, reason))
}

fn incomplete_collection_error(
    complete: &IcrcAccountTransactionCollectionData,
    reason: impl Into<String>,
) -> IcrcAccountTransactionError {
    IcrcAccountTransactionError::IncompleteCollection {
        index_canister_id: Some(complete.index_canister_id.clone()),
        pages_fetched: complete.page_count,
        rows_fetched: complete.transactions.len(),
        last_cursor: complete.last_cursor.clone(),
        reason: reason.into(),
    }
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
        .map(|value| principal_from_text::<IcrcError>(value, "index_canister_id"))
        .transpose()?
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

fn cache_operation_error(source: crate::CacheFileError) -> IcrcAccountTransactionError {
    HostCacheError::operation(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, source).into()
}

fn missing_account_transaction_cache_path(
    error: IcrcAccountTransactionError,
) -> Result<PathBuf, IcrcAccountTransactionError> {
    match error {
        IcrcAccountTransactionError::Cache(HostCacheError::MissingCache { path, .. }) => Ok(path),
        error => Err(error),
    }
}
