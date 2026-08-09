//! Module: sns::report::cache_summary
//!
//! Responsibility: share cache-summary loading, projection, and list-report assembly.
//! Does not own: cache storage, refresh-attempt persistence, or text rendering.
//! Boundary: keeps common cache-summary fields and ordering consistent.

use crate::{
    cache::CacheValidationStatus,
    snapshot_cache::SnapshotEnvelope,
    sns::report::{
        SnsCacheListReport, SnsCacheListRequest, SnsCacheSummary, SnsHostError,
        cache_attempt::read_sns_refresh_attempt_status,
        cache_paths::{sns_attempt_path_for_cache_path, sns_snapshot_network_cache_dir},
        cache_storage::{
            SnsCacheMetadata, SnsCacheStorageFamily, collect_sns_cache_paths, load_sns_cache_at,
        },
        enforce_mainnet_network,
    },
};
use std::path::{Path, PathBuf};

/// Load one SNS snapshot and project either its valid or invalid cache summary.
pub(in crate::sns::report) fn load_sns_cache_summary_at<Family>(
    cache_root: &Path,
    cache_path: PathBuf,
    network: &str,
) -> SnsCacheSummary
where
    Family: SnsCacheStorageFamily,
{
    match load_sns_cache_at::<Family>(cache_root, cache_path.clone(), network) {
        Ok(cache) => valid_sns_cache_summary(cache_root, cache_path, cache),
        Err(error) => invalid_sns_cache_summary(cache_root, cache_path, network, &error),
    }
}

/// Load summaries for a discovered set of SNS snapshot paths.
fn load_sns_cache_summaries<Family>(
    cache_root: &Path,
    paths: impl IntoIterator<Item = PathBuf>,
    network: &str,
) -> Vec<SnsCacheSummary>
where
    Family: SnsCacheStorageFamily,
{
    paths
        .into_iter()
        .map(|path| load_sns_cache_summary_at::<Family>(cache_root, path, network))
        .collect()
}

fn valid_sns_cache_summary<Data>(
    cache_root: &Path,
    cache_path: PathBuf,
    cache: SnapshotEnvelope<SnsCacheMetadata, Data>,
) -> SnsCacheSummary {
    let attempt_path = sns_attempt_path_for_cache_path(&cache_path);
    let latest_attempt = read_sns_refresh_attempt_status(cache_root, &attempt_path, &cache.network);
    SnsCacheSummary {
        id: cache.metadata.id,
        name: cache.metadata.name,
        root_canister_id: cache.metadata.root_canister_id,
        governance_canister_id: cache.metadata.governance_canister_id,
        cache_status: CacheValidationStatus::Valid,
        cache_error: None,
        complete: cache.completeness.is_api_exhausted(),
        row_count: cache.completeness.row_count,
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt,
    }
}

fn invalid_sns_cache_summary(
    cache_root: &Path,
    cache_path: PathBuf,
    network: &str,
    error: &SnsHostError,
) -> SnsCacheSummary {
    let attempt_path = sns_attempt_path_for_cache_path(&cache_path);
    SnsCacheSummary {
        id: 0,
        name: "-".to_string(),
        root_canister_id: root_from_cache_path(&cache_path),
        governance_canister_id: "-".to_string(),
        cache_status: CacheValidationStatus::Invalid,
        cache_error: Some(error.to_string()),
        complete: false,
        row_count: 0,
        page_count: 0,
        page_size: 0,
        fetched_at: "-".to_string(),
        source_endpoint: "-".to_string(),
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: read_sns_refresh_attempt_status(cache_root, &attempt_path, network),
    }
}

/// Build a deterministic cache-list report for one SNS cache family.
pub(in crate::sns::report) fn build_sns_cache_list_report<Family>(
    request: &SnsCacheListRequest,
    schema_version: u32,
) -> Result<SnsCacheListReport, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    enforce_mainnet_network(&request.network)?;
    let network_cache_root = sns_snapshot_network_cache_dir(&request.cache_root, &request.network)
        .display()
        .to_string();
    let paths = collect_sns_cache_paths::<Family>(&request.cache_root, &request.network)?;
    let mut caches =
        load_sns_cache_summaries::<Family>(&request.cache_root, paths, &request.network);
    sort_sns_cache_summaries(&mut caches);
    Ok(SnsCacheListReport {
        schema_version,
        network: request.network.clone(),
        cache_root: network_cache_root,
        cache_count: caches.len(),
        caches,
    })
}

/// Sort SNS cache summaries by stable list id and root principal.
fn sort_sns_cache_summaries(caches: &mut [SnsCacheSummary]) {
    caches.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.root_canister_id.cmp(&right.root_canister_id))
    });
}

/// Find a valid SNS cache summary by id without loading unrelated snapshots.
pub(in crate::sns::report) fn find_sns_cache_summary_by_id(
    paths: impl IntoIterator<Item = PathBuf>,
    id: usize,
    mut read_id: impl FnMut(&Path) -> Result<usize, SnsHostError>,
    mut load_summary: impl FnMut(PathBuf) -> SnsCacheSummary,
) -> Result<Option<SnsCacheSummary>, SnsHostError> {
    let mut matching = None;
    for path in paths {
        let Ok(candidate_id) = read_id(&path) else {
            continue;
        };
        if candidate_id != id {
            continue;
        }
        let summary = load_summary(path);
        if summary.id != id || summary.cache_error.is_some() {
            continue;
        }
        if matching.replace(summary).is_some() {
            return Err(SnsHostError::AmbiguousCacheId { id });
        }
    }
    Ok(matching)
}

/// Recover an SNS root identity from a complete snapshot cache path.
fn root_from_cache_path(cache_path: &Path) -> String {
    cache_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map_or_else(
            || "-".to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
}

#[cfg(test)]
mod tests {
    use super::find_sns_cache_summary_by_id;
    use crate::{
        cache::CacheValidationStatus,
        sns::report::{SnsCacheSummary, SnsHostError},
    };
    use std::{cell::Cell, path::PathBuf};

    fn summary(id: usize, path: PathBuf) -> SnsCacheSummary {
        let path = path.display().to_string();
        SnsCacheSummary {
            id,
            name: "SNS".to_string(),
            root_canister_id: path.clone(),
            governance_canister_id: "governance".to_string(),
            cache_status: CacheValidationStatus::Valid,
            cache_error: None,
            complete: true,
            row_count: 1,
            page_count: 1,
            page_size: 100,
            fetched_at: "2026-08-02T00:00:00Z".to_string(),
            source_endpoint: "https://ic0.app".to_string(),
            cache_path: path,
            refresh_attempt_path: "attempt.json".to_string(),
            latest_attempt: None,
        }
    }

    #[test]
    fn id_lookup_loads_only_the_matching_snapshot() {
        let paths = (1..=100)
            .map(|id| PathBuf::from(id.to_string()))
            .collect::<Vec<_>>();
        let header_reads = Cell::new(0);
        let snapshot_loads = Cell::new(0);

        let summary = find_sns_cache_summary_by_id(
            paths,
            73,
            |path| {
                header_reads.set(header_reads.get() + 1);
                path.to_string_lossy()
                    .parse::<usize>()
                    .map_err(|_| SnsHostError::InvalidLookup {
                        input: path.display().to_string(),
                    })
            },
            |path| {
                snapshot_loads.set(snapshot_loads.get() + 1);
                let id = path
                    .to_string_lossy()
                    .parse()
                    .expect("numeric fixture path");
                summary(id, path)
            },
        )
        .expect("lookup succeeds")
        .expect("matching summary");

        assert_eq!(summary.id, 73);
        assert_eq!(header_reads.get(), 100);
        assert_eq!(snapshot_loads.get(), 1);
    }

    #[test]
    fn id_lookup_rejects_multiple_valid_matching_snapshots() {
        let result = find_sns_cache_summary_by_id(
            [PathBuf::from("a"), PathBuf::from("b")],
            7,
            |_| Ok(7),
            |path| summary(7, path),
        );

        assert!(matches!(
            result,
            Err(SnsHostError::AmbiguousCacheId { id: 7 })
        ));
    }
}
