//! Module: icrc::account_transaction_cache::storage
//!
//! Responsibility: own account-history cache identity, paths, strict loading, and validation.
//! Does not own: live collection, refresh publication, attempt evidence, or report projection.
//! Boundary: accepts only normalized identities and complete API-exhausted snapshots.

use super::ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT;
use crate::{
    cache::validate_cache_collection_completeness,
    cache_file::{
        HostJsonCacheErrorMapper, JsonCacheReport, LoadJsonCacheRequest, load_json_cache_strict,
    },
    freshness::freshness_facts,
    icrc::{
        ledger::principal_from_text,
        live::account_transactions::validate_canonical_account_transactions,
        model::{
            CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionCacheRequest,
            IcrcAccountTransactionError, IcrcAccountTransactionSnapshot, IcrcError,
            normalize_optional_subaccount_hex,
        },
    },
    snapshot_cache::{SnapshotJsonPaths, SnapshotKey},
    subnet_catalog::{MAINNET_NETWORK, parse_utc_timestamp_secs},
};
use candid::Principal;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(super) const ICRC_ACCOUNT_TRANSACTION_CACHE_SCHEMA_VERSION: u32 = 1;

const ICRC_ACCOUNT_TRANSACTION_CACHE_DOMAIN: &str = "icrc";
const ICRC_ACCOUNT_TRANSACTION_CACHE_COLLECTION: &str = "transactions";
const ICRC_ACCOUNT_TRANSACTION_CACHE_FIELDS: &[&str] = &[
    "schema_version",
    "source_endpoint",
    "collection_started_at",
    "collection_completed_at",
    "fetched_by",
    "ledger_canister_id",
    "index_canister_id",
    "account_owner",
    "subaccount_hex",
    "balance",
    "token_symbol",
    "decimals",
    "newest_transaction_id",
    "oldest_transaction_id",
    "completeness",
    "transactions",
];

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

pub(super) fn load_snapshot_at(
    path: &Path,
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError> {
    let cached = load_json_cache_strict(
        LoadJsonCacheRequest {
            cache_root: &request.cache_root,
            path: path.to_path_buf(),
            network: MAINNET_NETWORK,
            expected_schema_version: ICRC_ACCOUNT_TRANSACTION_CACHE_SCHEMA_VERSION,
        },
        ICRC_ACCOUNT_TRANSACTION_CACHE_FIELDS,
        HostJsonCacheErrorMapper::new(ICRC_ACCOUNT_TRANSACTION_CACHE_COMPONENT),
    )
    .map_err(IcrcAccountTransactionError::from)?;
    validate_snapshot(path, &cached.report, request)?;
    Ok(CachedIcrcAccountTransactionSnapshot {
        path: cached.path,
        snapshot: cached.report,
    })
}

pub(super) fn validate_snapshot(
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
    validate_cache_collection_completeness(&snapshot.completeness, snapshot.transactions.len())
        .map_err(invalid)?;
    if snapshot.completeness.point_in_time_guaranteed {
        return Err(invalid(
            "index account history cannot claim a point-in-time guarantee".to_string(),
        ));
    }
    validate_canonical_account_transactions(&snapshot.transactions).map_err(invalid)?;
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

pub(super) fn normalize_cache_request(
    request: &IcrcAccountTransactionCacheRequest,
) -> Result<IcrcAccountTransactionCacheRequest, IcrcAccountTransactionError> {
    if request.source_endpoint.trim().is_empty() {
        return Err(IcrcAccountTransactionError::InvalidSourceEndpoint {
            value: request.source_endpoint.clone(),
            reason: "endpoint must not be empty".to_string(),
        });
    }
    let ledger_canister_id =
        principal_from_text::<IcrcError>(&request.ledger_canister_id, "ledger_canister_id")?;
    let account_owner = principal_from_text::<IcrcError>(&request.account_owner, "account_owner")?;
    Ok(IcrcAccountTransactionCacheRequest {
        cache_root: request.cache_root.clone(),
        source_endpoint: request.source_endpoint.clone(),
        ledger_canister_id: ledger_canister_id.to_text(),
        account_owner: account_owner.to_text(),
        subaccount_hex: normalize_optional_subaccount_hex(request.subaccount_hex.as_deref())?,
    })
}

pub(super) fn cache_paths(request: &IcrcAccountTransactionCacheRequest) -> SnapshotJsonPaths {
    SnapshotJsonPaths::for_key(
        &request.cache_root,
        &SnapshotKey::full(
            ICRC_ACCOUNT_TRANSACTION_CACHE_DOMAIN,
            MAINNET_NETWORK,
            cache_entity(request),
            ICRC_ACCOUNT_TRANSACTION_CACHE_COLLECTION,
        ),
    )
}

pub(super) fn cache_entity(request: &IcrcAccountTransactionCacheRequest) -> String {
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

pub(super) fn snapshot_is_stale(
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
