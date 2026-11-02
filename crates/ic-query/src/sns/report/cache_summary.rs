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
                id: cache.metadata.id,
                name: cache.metadata.name,
                root_canister_id: cache.metadata.root_canister_id,
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

/// Sort SNS cache summaries by stable list id and root principal.
fn sort_sns_cache_summaries(caches: &mut [SnsCacheSummary]) {
    caches.sort_by(|left, right| {
        let left_id = left.valid().map(|data| data.id);
        let right_id = right.valid().map(|data| data.id);
        left_id
            .cmp(&right_id)
            .then_with(|| left.cache_path.cmp(&right.cache_path))
    });
}

#[cfg(test)]
mod tests {
    use super::load_sns_cache_summaries;
    use crate::cache::CacheValidationStatus;
    use crate::{
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
    fn listing_respects_native_non_utf8_cache_root_support() {
        let parent = temp_dir("ic-query-sns-non-utf8-root");
        let root = parent.join(OsString::from_vec(b"cache-\xff".to_vec()));
        let path = root.join("sns/ic/aaaaa-aa/proposals/full.json");
        let written = write_managed_text_atomically(&root, &path, "{}");
        // macOS filesystems may reject the name with EILSEQ before a root exists.
        #[cfg(target_os = "macos")]
        if let Err(crate::cache_file::CacheFileError::CreateDirectory { path, source }) = &written
            && source.raw_os_error() == Some(92)
        {
            assert_eq!(path, &root);
            assert!(!root.exists());
            std::fs::remove_dir_all(parent).unwrap();
            return;
        }
        written.unwrap();
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
        let error = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
            &root,
            [path.clone()],
            "ic",
            &mut ManagedReadBudget::new(32),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            crate::sns::SnsHostError::Cache(crate::HostCacheError::ParseCache { .. })
        ));
        std::fs::remove_file(path.with_file_name("full.refresh-attempt.json")).unwrap();
        let summaries = load_sns_cache_summaries::<SnsProposalsCacheCollection>(
            &root,
            [path],
            "ic",
            &mut ManagedReadBudget::new(32),
        )
        .unwrap();
        assert_eq!(summaries[0].cache_status(), CacheValidationStatus::Invalid);
        assert!(summaries[0].latest_attempt.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
