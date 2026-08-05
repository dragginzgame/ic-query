//! Module: nns::proposals::report::cache::reports
//!
//! Responsibility: build NNS proposal cache list and status reports.
//! Does not own: refresh execution, live governance calls, or text rendering.
//! Boundary: loads local complete snapshots and projects cache metadata.

use super::{
    NNS_PROPOSAL_CACHE_COMPONENT, NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION,
    NNS_PROPOSAL_CACHE_SCHEMA_VERSION, NNS_PROPOSAL_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    attempt::{read_attempt_status, read_attempt_status_strict},
    model::{
        NNS_PROPOSAL_CACHE_FIELDS, NnsProposalCache, NnsProposalCacheListReport,
        NnsProposalCacheStatusReport, NnsProposalCacheSummary,
    },
    paths::{nns_proposal_cache_paths, nns_proposal_cache_root},
};
use crate::{
    cache::{CacheCollectionCompleteness, validate_cache_collection_completeness},
    cache_file::{LoadJsonCacheRequest, OwnerJsonCacheErrorMapper, managed_file_exists},
    ic_registry::MAINNET_GOVERNANCE_CANISTER_ID,
    nns::{
        NnsGovernanceCacheRequest,
        governance::validate_governance_cache_metadata,
        proposals::report::{
            NnsProposalHostError,
            assemble::{
                NnsProposalListReportParts, NnsProposalReportParts, NnsProposalReportProvenance,
                nns_proposal_list_report_from_parts, nns_proposal_report_from_parts,
            },
            enforce_mainnet_network,
            model::{
                NnsProposalListReport, NnsProposalListRequest, NnsProposalReport,
                NnsProposalRequest,
            },
            view::{
                proposal_matches_before, proposal_matches_proposer, proposal_matches_query,
                proposal_matches_reward_status, proposal_matches_status, proposal_matches_topic,
                sort_nns_proposal_rows,
            },
        },
    },
    snapshot_cache::{SnapshotIdentityMismatch, SnapshotKey, load_complete_snapshot_for_key},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// Build a local NNS proposal cache list report.
pub fn build_nns_proposal_cache_list_report(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsProposalCacheListReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.cache_root, &request.network);
    let snapshot_path = paths.snapshot_path;
    let caches = if proposal_cache_exists(&request.cache_root, &snapshot_path)? {
        vec![load_nns_proposal_cache_summary(
            &request.cache_root,
            snapshot_path,
            &request.network,
        )]
    } else {
        Vec::new()
    };
    Ok(NnsProposalCacheListReport {
        schema_version: NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: nns_proposal_cache_root(&request.cache_root, &request.network)
            .display()
            .to_string(),
        cache_count: caches.len(),
        caches,
    })
}

/// Build a local NNS proposal cache status report.
pub fn build_nns_proposal_cache_status_report(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsProposalCacheStatusReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.cache_root, &request.network);
    let cache = if proposal_cache_exists(&request.cache_root, &paths.snapshot_path)? {
        Some(load_nns_proposal_cache_summary(
            &request.cache_root,
            paths.snapshot_path.clone(),
            &request.network,
        ))
    } else {
        None
    };
    let latest_attempt = read_attempt_status_strict(
        &request.cache_root,
        &paths.refresh_attempt_path,
        &request.network,
    )?;
    Ok(NnsProposalCacheStatusReport {
        schema_version: NNS_PROPOSAL_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: nns_proposal_cache_root(&request.cache_root, &request.network)
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
    cache_root: &Path,
) -> Result<Option<NnsProposalListReport>, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(cache_root, &request.network);
    if !proposal_cache_exists(cache_root, &paths.snapshot_path)? {
        return Ok(None);
    }
    let cache = load_nns_proposal_cache(cache_root, paths.snapshot_path.clone(), &request.network)?;
    Ok(Some(nns_proposal_list_report_from_cache(
        request,
        paths.snapshot_path,
        cache,
    )))
}

/// Build an NNS proposal detail report from a complete local proposal snapshot.
pub fn build_nns_proposal_report_from_cache(
    request: &NnsProposalRequest,
    cache_root: &Path,
) -> Result<Option<NnsProposalReport>, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(cache_root, &request.network);
    if !proposal_cache_exists(cache_root, &paths.snapshot_path)? {
        return Ok(None);
    }
    let cache = load_nns_proposal_cache(cache_root, paths.snapshot_path.clone(), &request.network)?;
    Ok(nns_proposal_report_from_cache(
        request,
        paths.snapshot_path,
        cache,
    ))
}

fn proposal_cache_exists(cache_root: &Path, path: &Path) -> Result<bool, NnsProposalHostError> {
    managed_file_exists(cache_root, path).map_err(|source| {
        NnsProposalHostError::Cache(crate::HostCacheError::operation(
            NNS_PROPOSAL_CACHE_COMPONENT,
            source,
        ))
    })
}

fn load_nns_proposal_cache_summary(
    cache_root: &Path,
    cache_path: PathBuf,
    network: &str,
) -> NnsProposalCacheSummary {
    match load_nns_proposal_cache(cache_root, cache_path.clone(), network) {
        Ok(cache) => nns_proposal_cache_summary(cache_root, cache_path, cache),
        Err(error) => invalid_nns_proposal_cache_summary(cache_root, cache_path, error),
    }
}

fn load_nns_proposal_cache(
    cache_root: &Path,
    cache_path: PathBuf,
    network: &str,
) -> Result<NnsProposalCache, NnsProposalHostError> {
    let key = SnapshotKey::full("nns", network, "governance", "proposals");
    let cache = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            cache_root,
            path: cache_path.clone(),
            network,
            expected_schema_version: NNS_PROPOSAL_CACHE_SCHEMA_VERSION,
        },
        &key,
        NNS_PROPOSAL_CACHE_FIELDS,
        OwnerJsonCacheErrorMapper::new(NNS_PROPOSAL_CACHE_COMPONENT, missing_proposal_cache_error),
        incomplete_snapshot_error,
        |mismatch| nns_identity_mismatch_error(cache_path.clone(), mismatch),
    )?;
    validate_nns_proposal_cache(&cache_path, &cache)?;
    Ok(cache)
}

fn validate_nns_proposal_cache(
    path: &Path,
    cache: &NnsProposalCache,
) -> Result<(), NnsProposalHostError> {
    let invalid = |reason| NnsProposalHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    };
    validate_cache_collection_completeness(&cache.completeness, cache.data.proposals.len())
        .map_err(invalid)?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid(
            "Governance proposal pagination cannot claim a point-in-time guarantee".to_string(),
        ));
    }
    validate_governance_cache_metadata(&cache.metadata).map_err(invalid)?;
    let mut proposal_ids = HashSet::new();
    for proposal in &cache.data.proposals {
        let proposal_id = proposal
            .proposal_id
            .ok_or_else(|| invalid("cache contains a proposal without an id".to_string()))?;
        if !proposal_ids.insert(proposal_id) {
            return Err(invalid(format!("duplicate proposal id {proposal_id}")));
        }
    }
    Ok(())
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
    cache_root: &Path,
    cache_path: PathBuf,
    cache: NnsProposalCache,
) -> NnsProposalCacheSummary {
    let attempt_path = nns_proposal_cache_paths_for_cache_path(&cache_path);
    NnsProposalCacheSummary {
        governance_canister_id: cache.metadata.governance_canister_id,
        cache_status: crate::cache::CacheValidationStatus::Valid,
        cache_error: None,
        complete: cache.completeness.is_api_exhausted(),
        row_count: cache.completeness.row_count,
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: read_attempt_status(cache_root, &attempt_path),
    }
}

fn invalid_nns_proposal_cache_summary(
    cache_root: &Path,
    cache_path: PathBuf,
    error: NnsProposalHostError,
) -> NnsProposalCacheSummary {
    let attempt_path = nns_proposal_cache_paths_for_cache_path(&cache_path);
    NnsProposalCacheSummary {
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        cache_status: crate::cache::CacheValidationStatus::Invalid,
        cache_error: Some(error.to_string()),
        complete: false,
        row_count: 0,
        page_count: 0,
        page_size: 0,
        fetched_at: "-".to_string(),
        source_endpoint: "-".to_string(),
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: read_attempt_status(cache_root, &attempt_path),
    }
}

fn nns_proposal_cache_paths_for_cache_path(cache_path: &Path) -> PathBuf {
    cache_path.with_file_name("full.refresh-attempt.json")
}

fn incomplete_snapshot_error(completeness: &CacheCollectionCompleteness) -> NnsProposalHostError {
    NnsProposalHostError::IncompleteRefresh {
        pages_fetched: completeness.page_count,
        rows_fetched: completeness.row_count,
        reason: "cached NNS proposals snapshot is not complete".to_string(),
    }
}

const fn missing_proposal_cache_error(path: PathBuf) -> NnsProposalHostError {
    NnsProposalHostError::MissingProposalCache { path }
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
