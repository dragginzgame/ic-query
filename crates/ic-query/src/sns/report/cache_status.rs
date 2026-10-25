//! Shared direct-root inspection for SNS snapshot and refresh-attempt evidence.

use crate::{
    HostCacheError,
    cache_file::managed_file_exists,
    sns::report::{
        SNS_CACHE_COMPONENT, SnsCacheStatusReport, SnsCacheStatusRequest, SnsHostError,
        cache_attempt::read_sns_refresh_attempt_status_strict,
        cache_paths::{sns_snapshot_cache_paths, sns_snapshot_network_cache_dir},
        cache_storage::SnsCacheStorageFamily,
        enforce_mainnet_network, load_sns_cache_summary_at,
        lookup::parse_sns_root_canister_input,
    },
};

/// Inspect one SNS cache family by root principal without discovery or live calls.
pub(in crate::sns::report) fn build_sns_cache_status_report<Family>(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    enforce_mainnet_network(&request.network)?;
    let root_canister_id = parse_sns_root_canister_input(&request.input)?;
    let paths = sns_snapshot_cache_paths::<Family>(
        &request.cache_root,
        &request.network,
        &root_canister_id,
    );
    let mut cache = if managed_file_exists(&request.cache_root, &paths.snapshot_path)
        .map_err(|source| HostCacheError::operation(SNS_CACHE_COMPONENT, source))?
    {
        Some(load_sns_cache_summary_at::<Family>(
            &request.cache_root,
            paths.snapshot_path.clone(),
            &request.network,
            None,
        )?)
    } else {
        None
    };
    let latest_attempt = read_sns_refresh_attempt_status_strict(
        &request.cache_root,
        &paths.refresh_attempt_path,
        &request.network,
        None,
    )?;
    if let Some(cache) = cache.as_mut() {
        cache.latest_attempt.clone_from(&latest_attempt);
    }
    Ok(SnsCacheStatusReport {
        schema_version: 1,
        network: request.network.clone(),
        cache_root: sns_snapshot_network_cache_dir(&request.cache_root, &request.network)
            .display()
            .to_string(),
        input: request.input.clone(),
        found: cache.is_some(),
        cache,
        expected_cache_path: Some(paths.snapshot_path.display().to_string()),
        refresh_attempt_path: Some(paths.refresh_attempt_path.display().to_string()),
        latest_attempt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::CacheCollectionCompleteness, cache_file::write_managed_text_atomically,
        sns::report::proposals_cache::SnsProposalsCacheCollection, test_support::temp_dir,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    fn write_snapshot(root: &Path, entity: &str) -> PathBuf {
        let path = root.join("sns/ic").join(entity).join("proposals/full.json");
        let cache = serde_json::json!({
            "schema_version": 1, "network": "ic", "domain": "sns", "entity": entity,
            "collection": "proposals", "scope": "full", "id": 7, "name": "Fixture",
            "sns_wasm_canister_id": "qaa6y-5yaaa-aaaaa-aaafa-cai", "root_canister_id": entity,
            "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
            "source_endpoint": "https://icp-api.io", "fetched_at": "2026-10-03T00:00:00Z",
            "fetched_by": "fixture", "completeness": CacheCollectionCompleteness::api_exhausted(100, 1, 0, false),
            "proposals": [],
        });
        write_managed_text_atomically(root, &path, &cache.to_string()).unwrap();
        path
    }

    #[test]
    fn root_status_reads_only_the_selected_snapshot() {
        let root = temp_dir("ic-query-sns-status-budget-invalid");
        write_snapshot(&root, "aaaaa-aa");
        let invalid = root.join("sns/ic/0-invalid/proposals/full.json");
        write_managed_text_atomically(&root, &invalid, "{invalid").unwrap();
        assert!(
            build_sns_cache_status_report::<SnsProposalsCacheCollection>(
                &SnsCacheStatusRequest::new(&root, "ic", "aaaaa-aa"),
            )
            .unwrap()
            .cache
            .is_some()
        );
        fs::remove_dir_all(root).unwrap();
    }
}
