//! Module: sns::report::cache_status
//!
//! Responsibility: assemble SNS cache status reports across snapshot families.
//! Does not own: cache storage, refresh-attempt persistence, or rendering.
//! Boundary: resolves id/root inputs through family-owned storage operations.

use crate::{
    HostCacheError,
    cache_file::{ManagedReadBudget, managed_file_exists},
    snapshot_cache::collect_full_collection_attempt_paths,
    sns::report::{
        SNS_CACHE_COMPONENT, SnsCacheStatusReport, SnsCacheStatusRequest, SnsCacheSummary,
        SnsHostError, SnsRefreshAttemptStatus,
        cache_attempt::read_sns_refresh_attempt_status_strict,
        cache_paths::{SnsCacheCollection, SnsSnapshotCachePaths, sns_snapshot_network_cache_dir},
        cache_storage::{
            SNS_CACHE_SCAN_MAXIMUM_BYTES, SnsCacheStorageFamily, collect_sns_cache_paths,
            read_sns_cache_header,
        },
        enforce_mainnet_network, find_sns_cache_summary_by_id, load_sns_cache_summary_at,
        lookup::parse_sns_root_canister_input,
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
        cache: lookup.cache.map(|mut cache| {
            cache.latest_attempt.clone_from(&lookup.latest_attempt);
            cache
        }),
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
        return build_id_cache_status_lookup::<Family>(
            network,
            cache_root,
            network_cache_root,
            id,
            &mut ManagedReadBudget::new(SNS_CACHE_SCAN_MAXIMUM_BYTES),
        );
    }
    build_root_cache_status_lookup::<Family>(network, cache_root, input, network_cache_root)
}

fn build_id_cache_status_lookup<Family>(
    network: &str,
    cache_root: &Path,
    network_cache_root: String,
    id: usize,
    budget: &mut ManagedReadBudget,
) -> Result<SnsCacheStatusLookup, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let cache = find_sns_cache_summary_by_id(
        collect_sns_cache_paths::<Family>(cache_root, network)?,
        id,
        budget,
        |path, budget| {
            read_sns_cache_header::<Family>(cache_root, path, network, budget)
                .map(|header| header.id)
        },
        |path, budget| load_sns_cache_summary_at::<Family>(cache_root, path, network, Some(budget)),
    )?;
    let (refresh_attempt_path, latest_attempt) = match cache.as_ref() {
        Some(cache) => {
            let path = cache.refresh_attempt_path.clone();
            let attempt = read_sns_refresh_attempt_status_strict(
                cache_root,
                Path::new(&path),
                network,
                Some(budget),
            )?;
            (Some(path), attempt)
        }
        None => match find_attempt_by_id::<Family>(network, cache_root, id, budget)? {
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
    budget: &mut ManagedReadBudget,
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
        if let Some(attempt) =
            read_sns_refresh_attempt_status_strict(cache_root, &path, network, Some(budget))?
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
            None,
        )?)
    } else {
        None
    };
    let latest_attempt =
        read_sns_refresh_attempt_status_strict(cache_root, &paths.attempt_path, network, None)?;
    Ok(SnsCacheStatusLookup {
        cache_root: network_cache_root,
        cache,
        expected_cache_path: Some(paths.cache_path.display().to_string()),
        refresh_attempt_path: Some(paths.attempt_path.display().to_string()),
        latest_attempt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::CacheCollectionCompleteness,
        cache_file::write_managed_text_atomically,
        sns::report::{
            cache_storage::sns_cache_scan_limit_exceeded,
            proposals_cache::SnsProposalsCacheCollection,
        },
        test_support::temp_dir,
    };
    use std::fs;

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
    fn numeric_status_never_reports_uniqueness_after_budget_exhaustion() {
        let root = temp_dir("ic-query-sns-status-budget-ambiguity");
        let first = write_snapshot(&root, "2vxsx-fae");
        write_snapshot(&root, "aaaaa-aa");
        let mut budget = ManagedReadBudget::new(2 * fs::metadata(&first).unwrap().len() + 1);
        let error = build_id_cache_status_lookup::<SnsProposalsCacheCollection>(
            "ic",
            &root,
            String::new(),
            7,
            &mut budget,
        )
        .err()
        .unwrap();
        assert!(sns_cache_scan_limit_exceeded(&error));
        assert!(matches!(
            build_id_cache_status_lookup::<SnsProposalsCacheCollection>(
                "ic",
                &root,
                String::new(),
                7,
                &mut ManagedReadBudget::new(16_384)
            ),
            Err(SnsHostError::AmbiguousCacheId { id: 7 })
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_candidates_consume_budget_but_root_status_remains_direct() {
        let root = temp_dir("ic-query-sns-status-budget-invalid");
        write_snapshot(&root, "aaaaa-aa");
        let invalid = root.join("sns/ic/0-invalid/proposals/full.json");
        write_managed_text_atomically(&root, &invalid, "{invalid").unwrap();
        let error = build_id_cache_status_lookup::<SnsProposalsCacheCollection>(
            "ic",
            &root,
            String::new(),
            7,
            &mut ManagedReadBudget::new(9),
        )
        .err()
        .unwrap();
        assert!(sns_cache_scan_limit_exceeded(&error));
        assert!(
            build_root_cache_status_lookup::<SnsProposalsCacheCollection>(
                "ic",
                &root,
                "aaaaa-aa",
                String::new()
            )
            .unwrap()
            .cache
            .is_some()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn attempt_fallback_shares_the_candidate_byte_allowance() {
        let root = temp_dir("ic-query-sns-status-budget-attempt");
        let invalid = root.join("sns/ic/aaaaa-aa/proposals/full.json");
        write_managed_text_atomically(&root, &invalid, "{invalid").unwrap();
        let attempt = invalid.with_file_name("full.refresh-attempt.json");
        let evidence = serde_json::json!({
            "schema_version": 1, "network": "ic", "source_endpoint": "https://icp-api.io",
            "started_at": "2026-10-03T00:00:00Z", "updated_at": "2026-10-03T00:00:00Z",
            "id": 7, "root_canister_id": "aaaaa-aa", "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
            "status": "failed", "page_size": 100, "pages_fetched": 0, "rows_fetched": 0,
            "last_cursor": null, "last_error": "fixture failure",
        });
        write_managed_text_atomically(&root, &attempt, &evidence.to_string()).unwrap();
        let error = build_id_cache_status_lookup::<SnsProposalsCacheCollection>(
            "ic",
            &root,
            String::new(),
            7,
            &mut ManagedReadBudget::new(9),
        )
        .err()
        .unwrap();
        assert!(sns_cache_scan_limit_exceeded(&error));
        let lookup = build_id_cache_status_lookup::<SnsProposalsCacheCollection>(
            "ic",
            &root,
            String::new(),
            7,
            &mut ManagedReadBudget::new(16_384),
        )
        .unwrap();
        assert!(lookup.cache.is_none());
        assert_eq!(lookup.latest_attempt.unwrap().id, 7);
        fs::remove_dir_all(root).unwrap();
    }
}
