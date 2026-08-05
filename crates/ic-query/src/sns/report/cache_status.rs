//! Module: sns::report::cache_status
//!
//! Responsibility: assemble SNS cache status reports across snapshot families.
//! Does not own: cache storage, refresh-attempt persistence, or rendering.
//! Boundary: resolves id/root inputs through family-owned storage operations.

use crate::{
    HostCacheError,
    cache_file::managed_file_exists,
    snapshot_cache::collect_full_collection_attempt_paths,
    sns::report::{
        SNS_CACHE_COMPONENT, SnsCacheStatusReport, SnsCacheStatusRequest, SnsCacheSummary,
        SnsHostError, SnsRefreshAttemptStatus,
        cache_attempt::read_sns_refresh_attempt_status_strict,
        cache_paths::{SnsCacheCollection, SnsSnapshotCachePaths, sns_snapshot_network_cache_dir},
        cache_storage::{SnsCacheStorageFamily, collect_sns_cache_paths, read_sns_cache_header},
        enforce_mainnet_network, find_sns_cache_summary_by_id, load_sns_cache_summary_at,
        parse_sns_root_canister_input,
    },
};
use std::path::{Path, PathBuf};

struct SnsCacheStatusLookup {
    cache_root: String,
    cache: Option<SnsCacheSummary>,
    expected_cache_path: Option<String>,
    refresh_attempt_path: Option<String>,
    latest_attempt: Option<SnsRefreshAttemptStatus>,
}

/// Build a cache-status report for an SNS cache family by list id or root principal.
pub(in crate::sns::report) fn build_sns_cache_status_report<Family>(
    request: &SnsCacheStatusRequest,
    schema_version: u32,
) -> Result<SnsCacheStatusReport, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let lookup = build_sns_cache_status_lookup::<Family>(
        &request.network,
        &request.cache_root,
        &request.input,
    )?;
    Ok(SnsCacheStatusReport {
        schema_version,
        network: request.network.clone(),
        cache_root: lookup.cache_root,
        input: request.input.clone(),
        found: lookup.cache.is_some(),
        cache: lookup.cache,
        expected_cache_path: lookup.expected_cache_path,
        refresh_attempt_path: lookup.refresh_attempt_path,
        latest_attempt: lookup.latest_attempt,
    })
}

fn build_sns_cache_status_lookup<Family>(
    network: &str,
    cache_root: &Path,
    input: &str,
) -> Result<SnsCacheStatusLookup, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    enforce_mainnet_network(network)?;
    let network_cache_root = sns_snapshot_network_cache_dir(cache_root, network)
        .display()
        .to_string();
    if let Ok(id) = input.parse::<usize>() {
        return build_id_cache_status_lookup::<Family>(network, cache_root, network_cache_root, id);
    }
    build_root_cache_status_lookup::<Family>(network, cache_root, input, network_cache_root)
}

fn build_id_cache_status_lookup<Family>(
    network: &str,
    cache_root: &Path,
    network_cache_root: String,
    id: usize,
) -> Result<SnsCacheStatusLookup, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let cache = find_sns_cache_summary_by_id(
        collect_sns_cache_paths::<Family>(cache_root, network)?,
        id,
        |path| {
            read_sns_cache_header::<Family>(cache_root, path, network)
                .map(|header| header.metadata.id)
        },
        |path| load_sns_cache_summary_at::<Family>(cache_root, path, network),
    )?;
    let (refresh_attempt_path, latest_attempt) = match cache.as_ref() {
        Some(cache) => {
            let path = cache.refresh_attempt_path.clone();
            let attempt =
                read_sns_refresh_attempt_status_strict(cache_root, Path::new(&path), network)?;
            (Some(path), attempt)
        }
        None => match find_attempt_by_id::<Family>(network, cache_root, id)? {
            Some((path, attempt)) => (Some(path.display().to_string()), Some(attempt)),
            None => (None, None),
        },
    };
    let expected_cache_path = cache
        .is_none()
        .then(|| {
            refresh_attempt_path
                .as_deref()
                .map(Path::new)
                .map(|path| path.with_file_name("full.json").display().to_string())
        })
        .flatten();
    Ok(SnsCacheStatusLookup {
        cache_root: network_cache_root,
        cache,
        expected_cache_path,
        refresh_attempt_path,
        latest_attempt,
    })
}

fn find_attempt_by_id<Family>(
    network: &str,
    cache_root: &Path,
    id: usize,
) -> Result<Option<(PathBuf, SnsRefreshAttemptStatus)>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let network_dir = sns_snapshot_network_cache_dir(cache_root, network);
    let attempt_paths = collect_full_collection_attempt_paths(
        cache_root,
        &network_dir,
        <Family as SnsCacheCollection>::COLLECTION,
    )
    .map_err(|source| HostCacheError::operation(SNS_CACHE_COMPONENT, source))?;
    let mut matching = Vec::new();
    for path in attempt_paths {
        if let Some(attempt) = read_sns_refresh_attempt_status_strict(cache_root, &path, network)?
            && attempt.id == id
        {
            matching.push((path, attempt));
        }
    }
    match matching.len() {
        0 => Ok(None),
        1 => Ok(matching.pop()),
        _ => Err(SnsHostError::AmbiguousRefreshAttemptId { id }),
    }
}

fn build_root_cache_status_lookup<Family>(
    network: &str,
    cache_root: &Path,
    input: &str,
    network_cache_root: String,
) -> Result<SnsCacheStatusLookup, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let root_canister_id = parse_sns_root_canister_input(input)?;
    let paths = SnsSnapshotCachePaths::<Family>::for_root(cache_root, network, &root_canister_id);
    let cache = if managed_file_exists(cache_root, &paths.cache_path)
        .map_err(|source| HostCacheError::operation(SNS_CACHE_COMPONENT, source))?
    {
        Some(load_sns_cache_summary_at::<Family>(
            cache_root,
            paths.cache_path.clone(),
            network,
        ))
    } else {
        None
    };
    let latest_attempt =
        read_sns_refresh_attempt_status_strict(cache_root, &paths.attempt_path, network)?;
    Ok(SnsCacheStatusLookup {
        cache_root: network_cache_root,
        cache,
        expected_cache_path: Some(paths.cache_path.display().to_string()),
        refresh_attempt_path: Some(paths.attempt_path.display().to_string()),
        latest_attempt,
    })
}
