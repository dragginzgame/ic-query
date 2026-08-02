//! Module: icrc::account_transaction_cache::attempt
//!
//! Responsibility: persist and validate complete account-history refresh lifecycle status.
//! Does not own: live collection, cache publication, or text rendering.
//! Boundary: keeps failed collection progress observable without publishing partial rows.

use super::ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT;
use crate::{
    HostCacheError,
    icrc::{
        live::account_transactions::normalize_transaction_cursor,
        model::{
            IcrcAccountTransactionCacheRequest, IcrcAccountTransactionError,
            IcrcAccountTransactionRefreshAttemptStatus, IcrcAccountTransactionRefreshRequest,
        },
    },
    snapshot_cache::{
        SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt,
        SnapshotRefreshAttemptReadError, SnapshotRefreshProgress, current_attempt_timestamp,
        read_snapshot_refresh_attempt_strict, validate_snapshot_refresh_attempt,
        write_snapshot_refresh_attempt,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use candid::Principal;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::Path;

const ATTEMPT_METADATA_FIELDS: &[&str] = &[
    "ledger_canister_id",
    "account_owner",
    "subaccount_hex",
    "index_canister_id",
];

type AccountTransactionRefreshAttempt =
    SnapshotRefreshAttempt<AccountTransactionRefreshAttemptMetadata>;

///
/// AccountTransactionRefreshAttemptMetadata
///
/// Stable target identity and optional requested index recorded in an attempt sidecar.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
struct AccountTransactionRefreshAttemptMetadata {
    ledger_canister_id: String,
    account_owner: String,
    subaccount_hex: Option<String>,
    index_canister_id: Option<String>,
}

pub(super) fn read_refresh_attempt_status(
    path: &Path,
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<Option<IcrcAccountTransactionRefreshAttemptStatus>, IcrcAccountTransactionError> {
    read_snapshot_refresh_attempt_strict::<AccountTransactionRefreshAttempt>(
        path,
        ATTEMPT_METADATA_FIELDS,
    )
    .map_err(map_attempt_read_error)?
    .map(|attempt| {
        validate_attempt(path, request, &attempt)?;
        Ok(IcrcAccountTransactionRefreshAttemptStatus {
            status: attempt.status,
            started_at: attempt.started_at,
            updated_at: attempt.updated_at,
            index_canister_id: attempt.metadata.index_canister_id,
            page_size: attempt.page_size,
            pages_fetched: attempt.pages_fetched,
            rows_fetched: attempt.rows_fetched,
            last_cursor: attempt.last_cursor,
            last_error: attempt.last_error,
        })
    })
    .transpose()
}

pub(super) fn write_starting_attempt(
    path: &Path,
    request: &IcrcAccountTransactionRefreshRequest,
) -> Result<(), IcrcAccountTransactionError> {
    write_attempt(
        path,
        request,
        "running",
        request.index_canister_id.clone(),
        SnapshotRefreshProgress::default(),
        None,
    )
}

pub(super) fn write_complete_attempt(
    path: &Path,
    request: &IcrcAccountTransactionRefreshRequest,
    index_canister_id: &str,
    last_cursor: Option<String>,
    pages_fetched: u32,
    rows_fetched: usize,
) -> Result<(), IcrcAccountTransactionError> {
    write_attempt(
        path,
        request,
        "complete",
        Some(index_canister_id.to_string()),
        SnapshotRefreshProgress::new(pages_fetched, rows_fetched, last_cursor),
        None,
    )
}

pub(super) fn write_failed_attempt(
    path: &Path,
    request: &IcrcAccountTransactionRefreshRequest,
    error: &IcrcAccountTransactionError,
) {
    let (index_canister_id, progress) = collection_error_evidence(error);
    let _ = write_attempt(
        path,
        request,
        "failed",
        index_canister_id.or_else(|| request.index_canister_id.clone()),
        progress,
        Some(error.to_string()),
    );
}

fn collection_error_evidence(
    error: &IcrcAccountTransactionError,
) -> (Option<String>, SnapshotRefreshProgress) {
    match error {
        IcrcAccountTransactionError::IncompleteCollection {
            index_canister_id,
            pages_fetched,
            rows_fetched,
            last_cursor,
            ..
        }
        | IcrcAccountTransactionError::CollectionPage {
            index_canister_id,
            pages_fetched,
            rows_fetched,
            last_cursor,
            ..
        } => (
            index_canister_id.clone(),
            SnapshotRefreshProgress::new(*pages_fetched, *rows_fetched, last_cursor.clone()),
        ),
        _ => (None, SnapshotRefreshProgress::default()),
    }
}

fn write_attempt(
    path: &Path,
    request: &IcrcAccountTransactionRefreshRequest,
    status: &'static str,
    index_canister_id: Option<String>,
    progress: SnapshotRefreshProgress,
    last_error: Option<String>,
) -> Result<(), IcrcAccountTransactionError> {
    let started_at = format_utc_timestamp_secs(request.now_unix_secs);
    let attempt = AccountTransactionRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: request.cache.source_endpoint.clone(),
        started_at: started_at.clone(),
        updated_at: current_attempt_timestamp(&started_at),
        metadata: AccountTransactionRefreshAttemptMetadata {
            ledger_canister_id: request.cache.ledger_canister_id.clone(),
            account_owner: request.cache.account_owner.clone(),
            subaccount_hex: request.cache.subaccount_hex.clone(),
            index_canister_id,
        },
        status: status.to_string(),
        page_size: request.page_size,
        pages_fetched: progress.pages_fetched,
        rows_fetched: progress.rows_fetched,
        last_cursor: progress.last_cursor,
        last_error,
    };
    write_snapshot_refresh_attempt(
        path,
        &attempt,
        |path, source| {
            HostCacheError::serialize_cache(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, path, source)
                .into()
        },
        |source| HostCacheError::operation(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, source).into(),
    )
}

fn validate_attempt(
    path: &Path,
    request: &IcrcAccountTransactionCacheRequest,
    attempt: &AccountTransactionRefreshAttempt,
) -> Result<(), IcrcAccountTransactionError> {
    let invalid = |reason| IcrcAccountTransactionError::InvalidRefreshAttempt {
        path: path.to_path_buf(),
        reason,
    };
    validate_snapshot_refresh_attempt(attempt, MAINNET_NETWORK).map_err(invalid)?;
    if attempt.source_endpoint != request.source_endpoint
        || attempt.metadata.ledger_canister_id != request.ledger_canister_id
        || attempt.metadata.account_owner != request.account_owner
        || attempt.metadata.subaccount_hex != request.subaccount_hex
    {
        return Err(invalid(
            "attempt identity does not match requested endpoint, ledger, or account".to_string(),
        ));
    }
    if let Some(index_canister_id) = attempt.metadata.index_canister_id.as_deref() {
        Principal::from_text(index_canister_id)
            .map_err(|error| invalid(format!("invalid index canister id: {error}")))?;
    }
    if let Some(cursor) = attempt.last_cursor.as_deref() {
        normalize_transaction_cursor(cursor).map_err(|error| invalid(error.to_string()))?;
    }
    Ok(())
}

fn map_attempt_read_error(error: SnapshotRefreshAttemptReadError) -> IcrcAccountTransactionError {
    match error {
        SnapshotRefreshAttemptReadError::Read { path, source } => {
            HostCacheError::read_cache(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, path, source)
                .into()
        }
        SnapshotRefreshAttemptReadError::Parse { path, source } => {
            HostCacheError::parse_cache(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT, path, source)
                .into()
        }
        SnapshotRefreshAttemptReadError::Invalid { path, reason } => {
            IcrcAccountTransactionError::InvalidRefreshAttempt { path, reason }
        }
    }
}
