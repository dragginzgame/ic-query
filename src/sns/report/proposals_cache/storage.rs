//! Module: sns::report::proposals_cache::storage
//!
//! Responsibility: proposal snapshot cache loading, lookup, and summaries.
//! Does not own: refresh orchestration, report status assembly, or text rendering.
//! Boundary: reads complete proposal snapshots and maps them to cache summaries.

use super::{
    SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
    errors::SnsProposalsCacheErrors,
    model::{SnsProposalsCache, SnsProposalsCacheHeader, SnsProposalsRefreshAttempt},
    paths::{SnsProposalsCachePaths, attempt_path_for_cache_path, sns_network_cache_dir},
};
use crate::{
    cache_file::LoadJsonCacheRequest,
    snapshot_cache::{load_complete_snapshot, load_snapshot_header, read_snapshot_refresh_attempt},
    sns::report::{SnsHostError, SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus},
};
use std::path::{Path, PathBuf};

/// List summaries for all complete SNS proposal caches under one network.
pub(super) fn list_sns_proposals_cache_summaries(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<SnsProposalsCacheSummary>, SnsHostError> {
    let cache_root = sns_network_cache_dir(icp_root, network);
    if !cache_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut summaries = Vec::new();
    for entry in std::fs::read_dir(&cache_root).map_err(|source| SnsHostError::ReadCache {
        path: cache_root.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| SnsHostError::ReadCache {
            path: cache_root.clone(),
            source,
        })?;
        let root_path = entry.path();
        let cache_path = root_path.join("proposals").join("full.json");
        if cache_path.is_file() {
            summaries.push(sns_proposals_cache_summary(
                cache_path.clone(),
                load_sns_proposals_cache_at(cache_path, network)?,
            ));
        }
    }
    Ok(summaries)
}

/// Find a complete SNS proposal cache by stable SNS list id.
pub(super) fn find_sns_proposals_cache_by_id(
    icp_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<(PathBuf, SnsProposalsCache)>, SnsHostError> {
    let cache_root = sns_network_cache_dir(icp_root, network);
    if !cache_root.is_dir() {
        return Ok(None);
    }
    for entry in std::fs::read_dir(&cache_root).map_err(|source| SnsHostError::ReadCache {
        path: cache_root.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| SnsHostError::ReadCache {
            path: cache_root.clone(),
            source,
        })?;
        let cache_path = entry.path().join("proposals").join("full.json");
        if !cache_path.is_file() {
            continue;
        }
        let header = load_sns_proposals_cache_header(cache_path.clone(), network)?;
        if header.metadata.id == id {
            let cache = load_sns_proposals_cache_at(cache_path.clone(), network)?;
            return Ok(Some((cache_path, cache)));
        }
    }
    Ok(None)
}

/// Build a public cache summary from one loaded complete proposal snapshot.
pub(super) fn sns_proposals_cache_summary(
    cache_path: PathBuf,
    cache: SnsProposalsCache,
) -> SnsProposalsCacheSummary {
    let latest_attempt =
        read_sns_proposals_attempt_status(&attempt_path_for_cache_path(&cache_path));
    SnsProposalsCacheSummary {
        id: cache.metadata.id,
        name: cache.metadata.name,
        root_canister_id: cache.metadata.root_canister_id,
        governance_canister_id: cache.metadata.governance_canister_id,
        complete: cache.completeness.is_api_exhausted(),
        row_count: cache.completeness.row_count,
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        refresh_attempt_path: attempt_path_for_cache_path(&cache_path)
            .display()
            .to_string(),
        cache_path: cache_path.display().to_string(),
        latest_attempt,
    }
}

/// Load one complete SNS proposal snapshot from a concrete cache path.
pub(super) fn load_sns_proposals_cache_at(
    cache_path: PathBuf,
    network: &str,
) -> Result<SnsProposalsCache, SnsHostError> {
    load_complete_snapshot(
        LoadJsonCacheRequest {
            path: cache_path,
            network,
            expected_schema_version: SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        },
        SnsProposalsCacheErrors,
        |completeness| SnsHostError::IncompleteRefresh {
            pages_fetched: completeness.page_count,
            rows_fetched: completeness.row_count,
            reason: "cached SNS proposals snapshot is not complete".to_string(),
        },
    )
}

/// Read the latest proposal refresh-attempt status when present.
pub(super) fn read_sns_proposals_attempt_status(
    path: &Path,
) -> Option<SnsProposalsRefreshAttemptStatus> {
    let attempt = read_snapshot_refresh_attempt::<SnsProposalsRefreshAttempt>(path)?;
    Some(SnsProposalsRefreshAttemptStatus {
        status: attempt.status,
        started_at: attempt.started_at,
        updated_at: attempt.updated_at,
        page_size: attempt.page_size,
        pages_fetched: attempt.pages_fetched,
        rows_fetched: attempt.rows_fetched,
        last_cursor: attempt.last_cursor,
        last_error: attempt.last_error,
    })
}

fn load_sns_proposals_cache_header(
    cache_path: PathBuf,
    network: &str,
) -> Result<SnsProposalsCacheHeader, SnsHostError> {
    load_snapshot_header(
        LoadJsonCacheRequest {
            path: cache_path,
            network,
            expected_schema_version: SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        },
        SnsProposalsCacheErrors,
    )
}

/// Build the expected complete proposal cache path for one SNS root.
pub(super) fn expected_cache_path_for_root(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    SnsProposalsCachePaths::for_root(icp_root, network, root_canister_id).cache_path
}
