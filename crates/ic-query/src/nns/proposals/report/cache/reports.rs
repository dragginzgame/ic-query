//! Module: nns::proposals::report::cache::reports
//!
//! Responsibility: build NNS proposal cache list and status reports.
//! Does not own: refresh execution, live governance calls, or text rendering.
//! Boundary: loads local complete snapshots and projects cache metadata.

use super::{
    NNS_PROPOSAL_CACHE_COMPONENT, NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION,
    NNS_PROPOSAL_CACHE_SCHEMA_VERSION,
    model::{NNS_PROPOSAL_CACHE_FIELDS, NnsProposalCache, NnsProposalCacheListReport},
    paths::{nns_proposal_cache_paths, nns_proposal_cache_root},
};
use crate::{
    cache::{CacheCollectionCompleteness, validate_cache_collection_completeness},
    cache_file::{LoadJsonCacheRequest, managed_file_exists},
    nns::{
        NnsGovernanceCacheRequest,
        governance::{
            NnsGovernanceCacheStatusReport, NnsGovernanceCacheSummary, NnsGovernanceReportContext,
            NnsGovernanceSourceProvenance, read_governance_refresh_attempt_status,
            validate_governance_cache_metadata,
        },
        proposals::report::{
            NnsProposalHostError,
            activity::{
                NnsProposalActivityReport, NnsProposalActivityRequest,
                build_nns_proposal_activity_report, validate_time_window,
            },
            assemble::{
                NnsProposalReportParts, NnsProposalReportProvenance,
                nns_proposal_list_report_from_rows, nns_proposal_report_from_parts,
            },
            collection::validate_collection_state,
            enforce_mainnet_network,
            model::{
                NnsProposalListReport, NnsProposalListRequest, NnsProposalReport,
                NnsProposalRequest,
            },
            view::{
                project_nns_proposal_rows, proposal_matches_before, proposal_matches_reward_status,
                proposal_matches_status,
            },
        },
    },
    snapshot_cache::{SnapshotIdentityMismatch, SnapshotKey, load_complete_snapshot_for_key},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// Build proposal activity from the complete local snapshot without making a network call.
pub fn build_nns_proposal_activity_report_from_cache(
    cache_request: &NnsGovernanceCacheRequest,
    request: &NnsProposalActivityRequest,
) -> Result<NnsProposalActivityReport, NnsProposalHostError> {
    enforce_mainnet_network(&cache_request.network)?;
    validate_time_window(request)?;
    let path =
        nns_proposal_cache_paths(&cache_request.cache_root, &cache_request.network).snapshot_path;
    let cache = load_nns_proposal_cache(&cache_request.cache_root, path, &cache_request.network)?;
    Ok(build_nns_proposal_activity_report(
        request,
        &cache.data.collection_state,
        &cache.data.proposals,
    )?)
}

/// Build a local NNS proposal cache list report.
pub fn build_nns_proposal_cache_list_report(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsProposalCacheListReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.cache_root, &request.network);
    let found = proposal_cache_exists(&request.cache_root, &paths.snapshot_path)?;
    let caches = if found {
        vec![load_nns_proposal_cache_summary(
            &request.cache_root,
            paths.snapshot_path.clone(),
            &request.network,
        )]
    } else {
        Vec::new()
    };
    let latest_attempt = if found {
        read_governance_refresh_attempt_status(
            &request.cache_root,
            &paths.refresh_attempt_path,
            &request.network,
            NNS_PROPOSAL_CACHE_COMPONENT,
        )
        .ok()
        .flatten()
    } else {
        None
    };
    Ok(NnsProposalCacheListReport {
        schema_version: NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        collection: "proposals".to_string(),
        cache_root: nns_proposal_cache_root(&request.cache_root, &request.network)
            .display()
            .to_string(),
        cache_count: caches.len(),
        caches,
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        latest_attempt,
    })
}

/// Build a local NNS proposal cache status report.
pub fn build_nns_proposal_cache_status_report(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsGovernanceCacheStatusReport, NnsProposalHostError> {
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
    let latest_attempt = read_governance_refresh_attempt_status(
        &request.cache_root,
        &paths.refresh_attempt_path,
        &request.network,
        NNS_PROPOSAL_CACHE_COMPONENT,
    )?;
    Ok(NnsGovernanceCacheStatusReport::from_cache(
        request,
        "proposals",
        &paths,
        cache,
        latest_attempt,
    ))
}

/// Build an NNS proposal list report from a complete local proposal snapshot.
pub fn build_nns_proposal_list_report_from_cache(
    request: &NnsProposalListRequest,
    cache_root: &Path,
) -> Result<Option<NnsProposalListReport>, NnsProposalHostError> {
    enforce_mainnet_network(&request.governance.network)?;
    let paths = nns_proposal_cache_paths(cache_root, &request.governance.network);
    let cache = match load_nns_proposal_cache(
        cache_root,
        paths.snapshot_path.clone(),
        &request.governance.network,
    ) {
        Ok(cache) => cache,
        Err(NnsProposalHostError::MissingProposalCache { .. }) => return Ok(None),
        Err(error) => return Err(error),
    };
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
    enforce_mainnet_network(&request.governance.network)?;
    let paths = nns_proposal_cache_paths(cache_root, &request.governance.network);
    let cache = match load_nns_proposal_cache(
        cache_root,
        paths.snapshot_path.clone(),
        &request.governance.network,
    ) {
        Ok(cache) => cache,
        Err(NnsProposalHostError::MissingProposalCache { .. }) => return Ok(None),
        Err(error) => return Err(error),
    };
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
) -> NnsGovernanceCacheSummary {
    match load_nns_proposal_cache(cache_root, cache_path.clone(), network) {
        Ok(cache) => NnsGovernanceCacheSummary::from_snapshot(&cache_path, &cache),
        Err(error) => NnsGovernanceCacheSummary::invalid(&cache_path, error.to_string()),
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
            component: NNS_PROPOSAL_CACHE_COMPONENT,
            cache_root,
            path: cache_path.clone(),
            network,
            expected_schema_version: NNS_PROPOSAL_CACHE_SCHEMA_VERSION,
            maximum_bytes: crate::cache_file::MAX_COLLECTION_SNAPSHOT_BYTES,
        },
        None,
        &key,
        NNS_PROPOSAL_CACHE_FIELDS,
        missing_proposal_cache_error,
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
    let state = &cache.data.collection_state;
    validate_collection_state(state).map_err(|error| invalid(error.to_string()))?;
    let source = NnsGovernanceSourceProvenance::ReplicaQuery {
        endpoint: cache.source_endpoint.clone(),
        fetched_by: cache.fetched_by.clone(),
    };
    if !state.is_complete()
        || state.network() != cache.network
        || state.governance_canister_id() != cache.metadata.governance_canister_id
        || state.source() != Some(&source)
        || state.updated_at() != cache.fetched_at
        || state.pages_fetched() != cache.completeness.page_count
        || state.page_size() != cache.completeness.page_size
        || state.proposals_fetched() != cache.completeness.row_count as u64
    {
        return Err(invalid(
            "collection_state disagrees with snapshot identity, provenance, or completeness"
                .to_string(),
        ));
    }
    let mut proposal_ids = HashSet::new();
    for proposal in &cache.data.proposals {
        let proposal_id = proposal
            .proposal_id
            .ok_or_else(|| invalid("cache contains a proposal without an id".to_string()))?;
        if proposal_id == 0 {
            return Err(invalid("cache contains proposal id zero".to_string()));
        }
        if !proposal_ids.insert(proposal_id) {
            return Err(invalid(format!("duplicate proposal id {proposal_id}")));
        }
        super::super::source::validate_proposal_ballots(proposal)
            .map_err(|error| invalid(error.to_string()))?;
    }
    Ok(())
}

fn nns_proposal_list_report_from_cache(
    request: &NnsProposalListRequest,
    cache_path: PathBuf,
    cache: NnsProposalCache,
) -> NnsProposalListReport {
    let cache_complete = cache.completeness.is_api_exhausted();
    let context = nns_proposal_cache_report_context(&cache);
    let proposals = cache
        .data
        .proposals
        .into_iter()
        .filter(|proposal| proposal_matches_before(proposal, request.before_proposal_id))
        .filter(|proposal| proposal_matches_status(proposal, request.status))
        .filter(|proposal| proposal_matches_reward_status(proposal, request.reward_status));
    let mut proposals = project_nns_proposal_rows(proposals, request);
    proposals.truncate(usize::try_from(request.limit).unwrap_or(usize::MAX));
    nns_proposal_list_report_from_rows(
        request,
        context,
        NnsProposalReportProvenance::cache(&cache_path, cache_complete),
        proposals,
    )
}

fn nns_proposal_report_from_cache(
    request: &NnsProposalRequest,
    cache_path: PathBuf,
    cache: NnsProposalCache,
) -> Option<NnsProposalReport> {
    let cache_complete = cache.completeness.is_api_exhausted();
    let context = nns_proposal_cache_report_context(&cache);
    let proposal = cache
        .data
        .proposals
        .into_iter()
        .find(|proposal| proposal.proposal_id == Some(request.proposal_id))?;
    Some(nns_proposal_report_from_parts(NnsProposalReportParts {
        context,
        provenance: NnsProposalReportProvenance::cache(&cache_path, cache_complete),
        proposal_id: request.proposal_id,
        show_ballots: request.show_ballots,
        verbose: request.verbose,
        proposal,
    }))
}

fn nns_proposal_cache_report_context(cache: &NnsProposalCache) -> NnsGovernanceReportContext {
    NnsGovernanceReportContext {
        schema_version: 1,
        network: cache.network.clone(),
        governance_canister_id: cache.metadata.governance_canister_id.clone(),
        fetched_at: cache.fetched_at.clone(),
        source: NnsGovernanceSourceProvenance::ReplicaQuery {
            endpoint: cache.source_endpoint.clone(),
            fetched_by: cache.fetched_by.clone(),
        },
    }
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
