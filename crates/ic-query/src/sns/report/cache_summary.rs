//! Module: sns::report::cache_summary
//!
//! Responsibility: share cache-summary loading, projection, and list-report assembly.
//! Does not own: cache storage, refresh-attempt persistence, or text rendering.
//! Boundary: keeps common cache-summary fields and ordering consistent.

use crate::{
    cache::CacheValidationStatus,
    cache_file::ManagedReadBudget,
    snapshot_cache::SnapshotEnvelope,
    sns::report::{
        SnsCacheListReport, SnsCacheListRequest, SnsCacheSummary, SnsHostError,
        cache_attempt::read_sns_refresh_attempt_status_strict,
        cache_paths::{
            sns_attempt_path_for_cache_path, sns_root_from_cache_path,
            sns_snapshot_network_cache_dir,
        },
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
) -> Result<SnsCacheSummary, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    match load_sns_cache_at::<Family>(cache_root, cache_path.clone(), network, budget) {
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
            let attempt_path = sns_attempt_path_for_cache_path(&path);
            let mut summary =
                load_sns_cache_summary_at::<Family>(cache_root, path, network, Some(budget))?;
            summary.latest_attempt = match read_sns_refresh_attempt_status_strict(
                cache_root,
                &attempt_path,
                network,
                Some(budget),
            ) {
                Ok(attempt) => attempt,
                Err(error) if sns_cache_scan_limit_exceeded(&error) => return Err(error),
                Err(_) => None,
            };
            Ok(summary)
        })
        .collect()
}

fn valid_sns_cache_summary<Data>(
    cache_path: PathBuf,
    cache: SnapshotEnvelope<SnsCacheMetadata, Data>,
) -> SnsCacheSummary {
    let attempt_path = sns_attempt_path_for_cache_path(&cache_path);
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
        latest_attempt: None,
    }
}

fn invalid_sns_cache_summary(cache_path: PathBuf, error: &SnsHostError) -> SnsCacheSummary {
    let attempt_path = sns_attempt_path_for_cache_path(&cache_path);
    SnsCacheSummary {
        id: 0,
        name: "-".to_string(),
        root_canister_id: sns_root_from_cache_path(&cache_path).map_or_else(
            || "-".to_string(),
            |name| name.to_string_lossy().into_owned(),
        ),
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
        latest_attempt: None,
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

#[cfg(test)]
mod tests {
    use super::load_sns_cache_summaries;
    use crate::{
        cache::CacheValidationStatus,
        cache_file::{ManagedReadBudget, write_managed_text_atomically},
        sns::report::{
            cache_storage::sns_cache_scan_limit_exceeded,
            proposals_cache::SnsProposalsCacheCollection,
        },
        test_support::temp_dir,
    };

    #[cfg(unix)]
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    #[cfg(unix)]
    #[test]
    fn listing_reads_attempts_under_non_utf8_cache_roots() {
        let parent = temp_dir("ic-query-sns-non-utf8-root");
        let root = parent.join(OsString::from_vec(b"cache-\xff".to_vec()));
        let path = root.join("sns/ic/aaaaa-aa/proposals/full.json");
        write_managed_text_atomically(&root, &path, "{}").unwrap();
        let attempt = serde_json::json!({
            "schema_version": 1,
            "network": "ic",
            "source_endpoint": "https://icp-api.io",
            "started_at": "2026-10-24T00:00:00Z",
            "updated_at": "2026-10-24T00:00:00Z",
            "metadata": {
                "id": 1,
                "root_canister_id": "aaaaa-aa",
                "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai"
            },
            "status": "running",
            "page_size": 10,
            "pages_fetched": 0,
            "rows_fetched": 0,
            "last_cursor": null,
            "last_error": null
        });
        write_managed_text_atomically(
            &root,
            &path.with_file_name("full.refresh-attempt.json"),
            &attempt.to_string(),
        )
        .unwrap();
        let summaries = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
            &root,
            [path],
            "ic",
            &mut ManagedReadBudget::new(4096),
        )
        .unwrap();
        assert!(summaries[0].latest_attempt.is_some());
        std::fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn listing_budget_errors_are_not_invalid_rows_or_absent_attempts() {
        let root = temp_dir("ic-query-sns-list-budget");
        let path = root.join("sns/ic/aaaaa-aa/proposals/full.json");
        write_managed_text_atomically(&root, &path, "{}").unwrap();
        write_managed_text_atomically(
            &root,
            &path.with_file_name("full.refresh-attempt.json"),
            "{invalid",
        )
        .unwrap();
        let result = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
            &root,
            [path.clone()],
            "ic",
            &mut ManagedReadBudget::new(3),
        );
        assert!(sns_cache_scan_limit_exceeded(&result.unwrap_err()));
        let summaries = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
            &root,
            [path],
            "ic",
            &mut ManagedReadBudget::new(32),
        )
        .unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].cache_status, CacheValidationStatus::Invalid);
        assert!(summaries[0].latest_attempt.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
