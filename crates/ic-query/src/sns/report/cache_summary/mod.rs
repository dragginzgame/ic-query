//! Module: sns::report::cache_summary
//!
//! Responsibility: share cache-summary loading, projection, and list-report assembly.
//! Does not own: cache storage, refresh-attempt persistence, or text rendering.
//! Boundary: keeps common cache-summary fields and ordering consistent.

use crate::{
    cache_file::ManagedReadBudget,
    snapshot_cache::{SnapshotEnvelope, SnapshotJsonPaths},
    sns::report::{
        SnsCacheAdmission, SnsCacheListReport, SnsCacheListRequest, SnsCacheSummary, SnsHostError,
        SnsValidCacheSummary,
        cache_attempt::read_sns_refresh_attempt_status_strict,
        cache_paths::sns_snapshot_network_cache_dir,
        cache_storage::{
            SNS_CACHE_SCAN_MAXIMUM_BYTES, SnsCacheMetadata, SnsCacheStorageFamily,
            collect_sns_cache_paths, load_sns_cache_at, sns_cache_scan_limit_exceeded,
        },
        enforce_mainnet_network,
    },
};
use std::path::{Path, PathBuf};

/// Load one SNS snapshot into a summary; the caller owns refresh-attempt evidence.
pub(in crate::sns::report) fn load_sns_cache_summary_at<Family>(
    cache_root: &Path,
    cache_path: PathBuf,
    network: &str,
    budget: Option<&mut ManagedReadBudget>,
    opened_file: Option<std::fs::File>,
) -> Result<SnsCacheSummary, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    match load_sns_cache_at::<Family>(cache_root, cache_path.clone(), network, budget, opened_file)
    {
        Ok(cache) => Ok(valid_sns_cache_summary(cache_path, cache)),
        Err(error) if sns_cache_scan_limit_exceeded(&error) => Err(error),
        Err(error) => Ok(invalid_sns_cache_summary(cache_path, &error)),
    }
}

/// Load summaries for a discovered set of SNS snapshot paths.
fn load_sns_cache_summaries<Family>(
    cache_root: &Path,
    paths: impl IntoIterator<Item = PathBuf>,
    network: &str,
    budget: &mut ManagedReadBudget,
) -> Result<Vec<SnsCacheSummary>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    paths
        .into_iter()
        .map(|path| {
            let attempt_path = SnapshotJsonPaths::refresh_attempt_for_snapshot(&path);
            let mut summary =
                load_sns_cache_summary_at::<Family>(cache_root, path, network, Some(budget), None)?;
            summary.latest_attempt = read_sns_refresh_attempt_status_strict(
                cache_root,
                &attempt_path,
                network,
                Some(budget),
            )?;
            Ok(summary)
        })
        .collect()
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Consumes a discovered cache path together with its snapshot while constructing one summary."
)]
fn valid_sns_cache_summary<Data>(
    cache_path: PathBuf,
    cache: SnapshotEnvelope<SnsCacheMetadata, Data>,
) -> SnsCacheSummary {
    let attempt_path = SnapshotJsonPaths::refresh_attempt_for_snapshot(&cache_path);
    SnsCacheSummary {
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: None,
        admission: SnsCacheAdmission::Valid {
            data: SnsValidCacheSummary {
                governance_canister_id: cache.metadata.governance_canister_id,
                row_count: cache.completeness.row_count,
                page_count: cache.completeness.page_count,
                page_size: cache.completeness.page_size,
                fetched_at: cache.fetched_at,
                source_endpoint: cache.source_endpoint,
            },
        },
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Consumes a discovered cache path while retaining its invalid-content evidence in one summary."
)]
fn invalid_sns_cache_summary(cache_path: PathBuf, error: &SnsHostError) -> SnsCacheSummary {
    let attempt_path = SnapshotJsonPaths::refresh_attempt_for_snapshot(&cache_path);
    SnsCacheSummary {
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: None,
        admission: SnsCacheAdmission::Invalid {
            cache_error: error.to_string(),
        },
    }
}

/// Build a deterministic cache-list report for one SNS cache family.
pub(in crate::sns::report) fn build_sns_cache_list_report<Family>(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    enforce_mainnet_network(&request.network)?;
    let network_cache_root = sns_snapshot_network_cache_dir(&request.cache_root, &request.network)
        .display()
        .to_string();
    let paths = collect_sns_cache_paths::<Family>(&request.cache_root, &request.network)?;
    let mut budget = ManagedReadBudget::new(SNS_CACHE_SCAN_MAXIMUM_BYTES);
    let mut caches = load_sns_cache_summaries::<Family>(
        &request.cache_root,
        paths,
        &request.network,
        &mut budget,
    )?;
    sort_sns_cache_summaries(&mut caches);
    Ok(SnsCacheListReport {
        schema_version: 1,
        network: request.network.clone(),
        cache_root: network_cache_root,

        caches,
    })
}

/// Sort complete neuron cache summaries by their stable paths.
fn sort_sns_cache_summaries(caches: &mut [SnsCacheSummary]) {
    caches.sort_by(|left, right| left.cache_path.cmp(&right.cache_path));
}
