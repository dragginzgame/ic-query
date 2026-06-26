//! Module: nns::proposals::report::cache::reports
//!
//! Responsibility: build NNS proposal cache list and status reports.
//! Does not own: refresh execution, live governance calls, or text rendering.
//! Boundary: loads local complete snapshots and projects cache metadata.

use super::{
    NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION, NNS_PROPOSAL_CACHE_SCHEMA_VERSION,
    NNS_PROPOSAL_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    attempt::read_attempt_status,
    model::{
        NnsProposalCache, NnsProposalCacheListReport, NnsProposalCacheListRequest,
        NnsProposalCacheStatusReport, NnsProposalCacheStatusRequest, NnsProposalCacheSummary,
    },
    paths::{nns_proposal_cache_paths, nns_proposal_cache_root},
};
use crate::{
    cache_file::{LoadJsonCacheErrorMapper, LoadJsonCacheRequest},
    ic_registry::MAINNET_GOVERNANCE_CANISTER_ID,
    nns::proposals::report::{
        NnsProposalHostError,
        assemble::{
            NnsProposalListReportParts, NnsProposalReportParts, NnsProposalReportProvenance,
            nns_proposal_list_report_from_parts, nns_proposal_report_from_parts,
        },
        enforce_mainnet_network,
        model::{
            NnsProposalListReport, NnsProposalListRequest, NnsProposalReport, NnsProposalRequest,
        },
        view::{
            proposal_matches_before, proposal_matches_proposer, proposal_matches_query,
            proposal_matches_reward_status, proposal_matches_status, proposal_matches_topic,
            sort_nns_proposal_rows,
        },
    },
    snapshot_cache::{
        SNAPSHOT_CACHE_STATUS_INVALID, SNAPSHOT_CACHE_STATUS_OK, SnapshotCompleteness,
        SnapshotIdentityMismatch, SnapshotKey, load_complete_snapshot_for_key,
    },
};
use std::{
    io,
    path::{Path, PathBuf},
};

/// Build a local NNS proposal cache list report.
pub fn build_nns_proposal_cache_list_report(
    request: &NnsProposalCacheListRequest,
) -> Result<NnsProposalCacheListReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.icp_root, &request.network);
    let snapshot_path = paths.snapshot_path;
    let caches = if snapshot_path.is_file() {
        vec![load_nns_proposal_cache_summary(
            snapshot_path,
            &request.network,
        )]
    } else {
        Vec::new()
    };
    Ok(NnsProposalCacheListReport {
        schema_version: NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: nns_proposal_cache_root(&request.icp_root, &request.network)
            .display()
            .to_string(),
        cache_count: caches.len(),
        caches,
    })
}

/// Build a local NNS proposal cache status report.
pub fn build_nns_proposal_cache_status_report(
    request: &NnsProposalCacheStatusRequest,
) -> Result<NnsProposalCacheStatusReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.icp_root, &request.network);
    let cache = if paths.snapshot_path.is_file() {
        Some(load_nns_proposal_cache_summary(
            paths.snapshot_path.clone(),
            &request.network,
        ))
    } else {
        None
    };
    let latest_attempt = cache
        .as_ref()
        .and_then(|summary| summary.latest_attempt.clone())
        .or_else(|| read_attempt_status(&paths.refresh_attempt_path));
    Ok(NnsProposalCacheStatusReport {
        schema_version: NNS_PROPOSAL_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: nns_proposal_cache_root(&request.icp_root, &request.network)
            .display()
            .to_string(),
        found: cache.is_some(),
        cache,
        expected_cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        latest_attempt,
    })
}

/// Build an NNS proposal list report from a complete local proposal snapshot.
pub fn build_nns_proposal_list_report_from_cache(
    request: &NnsProposalListRequest,
    icp_root: &Path,
) -> Result<Option<NnsProposalListReport>, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(icp_root, &request.network);
    if !paths.snapshot_path.is_file() {
        return Ok(None);
    }
    let cache = load_nns_proposal_cache(paths.snapshot_path.clone(), &request.network)?;
    Ok(Some(nns_proposal_list_report_from_cache(
        request,
        paths.snapshot_path,
        cache,
    )))
}

/// Build an NNS proposal detail report from a complete local proposal snapshot.
pub fn build_nns_proposal_report_from_cache(
    request: &NnsProposalRequest,
    icp_root: &Path,
) -> Result<Option<NnsProposalReport>, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(icp_root, &request.network);
    if !paths.snapshot_path.is_file() {
        return Ok(None);
    }
    let cache = load_nns_proposal_cache(paths.snapshot_path.clone(), &request.network)?;
    Ok(nns_proposal_report_from_cache(
        request,
        paths.snapshot_path,
        cache,
    ))
}

fn load_nns_proposal_cache_summary(cache_path: PathBuf, network: &str) -> NnsProposalCacheSummary {
    match load_nns_proposal_cache(cache_path.clone(), network) {
        Ok(cache) => nns_proposal_cache_summary(cache_path, cache),
        Err(error) => invalid_nns_proposal_cache_summary(cache_path, error),
    }
}

fn load_nns_proposal_cache(
    cache_path: PathBuf,
    network: &str,
) -> Result<NnsProposalCache, NnsProposalHostError> {
    let key = SnapshotKey::full("nns", network, "governance", "proposals");
    load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            path: cache_path.clone(),
            network,
            expected_schema_version: NNS_PROPOSAL_CACHE_SCHEMA_VERSION,
        },
        &key,
        NnsProposalCacheErrors,
        incomplete_snapshot_error,
        |mismatch| nns_identity_mismatch_error(cache_path, mismatch),
    )
}

fn nns_proposal_list_report_from_cache(
    request: &NnsProposalListRequest,
    cache_path: PathBuf,
    cache: NnsProposalCache,
) -> NnsProposalListReport {
    let cache_complete = cache.completeness.is_api_exhausted();
    let mut proposals = cache
        .data
        .proposals
        .into_iter()
        .filter(|proposal| proposal_matches_before(proposal, request.before_proposal_id))
        .filter(|proposal| proposal_matches_status(proposal, request.status))
        .filter(|proposal| proposal_matches_reward_status(proposal, request.reward_status))
        .filter(|proposal| proposal_matches_topic(proposal, request.topic))
        .filter(|proposal| proposal_matches_proposer(proposal, request.proposer_neuron_id))
        .filter(|proposal| proposal_matches_query(proposal, request.query.as_deref()))
        .collect::<Vec<_>>();
    sort_nns_proposal_rows(&mut proposals, request.sort, request.sort_direction);
    proposals.truncate(usize::try_from(request.limit).unwrap_or(usize::MAX));
    nns_proposal_list_report_from_parts(NnsProposalListReportParts {
        network: cache.network,
        governance_canister_id: cache.metadata.governance_canister_id,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        fetched_by: cache.fetched_by,
        provenance: NnsProposalReportProvenance::cache(&cache_path, cache_complete),
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        reward_status: request.reward_status,
        topic: request.topic,
        proposer_neuron_id: request.proposer_neuron_id,
        query: request.query.clone(),
        sort: request.sort,
        sort_direction: request.sort_direction,
        verbose: request.verbose,
        proposals,
    })
}

fn nns_proposal_report_from_cache(
    request: &NnsProposalRequest,
    cache_path: PathBuf,
    cache: NnsProposalCache,
) -> Option<NnsProposalReport> {
    let cache_complete = cache.completeness.is_api_exhausted();
    let proposal = cache
        .data
        .proposals
        .into_iter()
        .find(|proposal| proposal.proposal_id == Some(request.proposal_id))?;
    Some(nns_proposal_report_from_parts(NnsProposalReportParts {
        network: cache.network,
        governance_canister_id: cache.metadata.governance_canister_id,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        fetched_by: cache.fetched_by,
        provenance: NnsProposalReportProvenance::cache(&cache_path, cache_complete),
        proposal_id: request.proposal_id,
        show_ballots: request.show_ballots,
        verbose: request.verbose,
        proposal,
    }))
}

fn nns_proposal_cache_summary(
    cache_path: PathBuf,
    cache: NnsProposalCache,
) -> NnsProposalCacheSummary {
    let attempt_path = nns_proposal_cache_paths_for_cache_path(&cache_path);
    NnsProposalCacheSummary {
        governance_canister_id: cache.metadata.governance_canister_id,
        cache_status: SNAPSHOT_CACHE_STATUS_OK.to_string(),
        cache_error: None,
        complete: cache.completeness.is_api_exhausted(),
        row_count: cache.completeness.row_count,
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: read_attempt_status(&attempt_path),
    }
}

fn invalid_nns_proposal_cache_summary(
    cache_path: PathBuf,
    error: NnsProposalHostError,
) -> NnsProposalCacheSummary {
    let attempt_path = nns_proposal_cache_paths_for_cache_path(&cache_path);
    NnsProposalCacheSummary {
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        cache_status: SNAPSHOT_CACHE_STATUS_INVALID.to_string(),
        cache_error: Some(error.to_string()),
        complete: false,
        row_count: 0,
        page_count: 0,
        page_size: 0,
        fetched_at: "-".to_string(),
        source_endpoint: "-".to_string(),
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: read_attempt_status(&attempt_path),
    }
}

fn nns_proposal_cache_paths_for_cache_path(cache_path: &Path) -> PathBuf {
    cache_path.with_file_name("full.refresh-attempt.json")
}

fn incomplete_snapshot_error(completeness: &SnapshotCompleteness) -> NnsProposalHostError {
    NnsProposalHostError::IncompleteRefresh {
        pages_fetched: completeness.page_count,
        rows_fetched: completeness.row_count,
        reason: "cached NNS proposals snapshot is not complete".to_string(),
    }
}

struct NnsProposalCacheErrors;

impl LoadJsonCacheErrorMapper for NnsProposalCacheErrors {
    type Error = NnsProposalHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        NnsProposalHostError::MissingProposalCache { path }
    }

    fn read_cache(&self, path: PathBuf, source: io::Error) -> Self::Error {
        NnsProposalHostError::ReadCache { path, source }
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        NnsProposalHostError::ParseCache { path, source }
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        NnsProposalHostError::UnsupportedCacheSchemaVersion { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        NnsProposalHostError::CacheNetworkMismatch { requested, actual }
    }
}

fn nns_identity_mismatch_error(
    path: PathBuf,
    mismatch: SnapshotIdentityMismatch,
) -> NnsProposalHostError {
    NnsProposalHostError::CacheIdentityMismatch {
        path,
        field: mismatch.field,
        expected: mismatch.expected,
        actual: mismatch.actual,
    }
}
