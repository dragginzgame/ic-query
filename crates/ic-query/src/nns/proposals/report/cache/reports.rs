//! Module: nns::proposals::report::cache::reports
//!
//! Responsibility: build NNS proposal cache list and status reports.
//! Does not own: refresh execution, transport execution, or text rendering.
//! Boundary: loads local complete snapshots and projects cache metadata.

use super::{
    NNS_PROPOSAL_CACHE_COMPONENT, NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION,
    NNS_PROPOSAL_CACHE_SCHEMA_VERSION,
    model::{NnsProposalCache, NnsProposalCacheListReport},
    paths::nns_proposal_cache_paths,
};
use crate::nns::proposals::report::source::{
    NnsProposalSource, build_nns_proposal_list_report_with_source,
    build_nns_proposal_report_with_source,
};
use crate::report::ReportProvenance;
use crate::{
    cache::{CacheCollectionCompleteness, validate_cache_collection_completeness},
    cache_file::LoadJsonCacheRequest,
    nns::{
        NnsGovernanceCacheRequest,
        governance::{
            NnsGovernanceCacheStatusReport, NnsGovernanceCacheSummary, NnsGovernanceReportContext,
            build_governance_cache_status_report, validate_governance_cache_metadata,
        },
        proposals::report::{
            NnsProposalHostError,
            activity::{
                NnsProposalActivityReport, NnsProposalActivityRequest, build_activity_from_cache,
                validate_time_window,
            },
            assemble::{nns_proposal_list_report_from_rows, nns_proposal_report_from_row},
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
    fs::File,
    path::{Path, PathBuf},
};

///
/// AdmittedProposalCache
///
/// Immutable complete snapshot admitted by the family cache reader.
///

pub(in crate::nns::proposals::report) struct AdmittedProposalCache {
    snapshot: NnsProposalCache,
}

impl AdmittedProposalCache {
    pub(in crate::nns::proposals::report) const fn collection_state(
        &self,
    ) -> &crate::nns::proposals::report::NnsProposalCollectionState {
        &self.snapshot.data.collection_state
    }

    pub(in crate::nns::proposals::report) fn rows(
        &self,
    ) -> &[crate::nns::proposals::report::model::NnsProposalRow] {
        &self.snapshot.data.proposals
    }
}

/// Prefer an admitted local snapshot; query the supplied source when the requested report is absent.
pub async fn build_nns_proposal_list_report_with_cache_and_source(
    request: &NnsProposalListRequest,
    cache_root: &Path,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalListReport, NnsProposalHostError> {
    if let Some(report) = build_nns_proposal_list_report_from_cache(request, cache_root)? {
        return Ok(report);
    }
    Ok(build_nns_proposal_list_report_with_source(request, source).await?)
}

/// Prefer an admitted local snapshot; query the supplied source when the requested report is absent.
pub async fn build_nns_proposal_report_with_cache_and_source(
    request: &NnsProposalRequest,
    cache_root: &Path,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalReport, NnsProposalHostError> {
    if let Some(report) = build_nns_proposal_report_from_cache(request, cache_root)? {
        return Ok(report);
    }
    Ok(build_nns_proposal_report_with_source(request, source).await?)
}

/// Build proposal activity from the complete local snapshot without making a network call.
pub fn build_nns_proposal_activity_report_from_cache(
    cache_request: &NnsGovernanceCacheRequest,
    request: &NnsProposalActivityRequest,
) -> Result<NnsProposalActivityReport, NnsProposalHostError> {
    enforce_mainnet_network(&cache_request.network)?;
    validate_time_window(request)?;
    let path =
        nns_proposal_cache_paths(&cache_request.cache_root, &cache_request.network).snapshot_path;
    let cache = load_nns_proposal_cache(
        &cache_request.cache_root,
        path,
        &cache_request.network,
        None,
    )?;
    Ok(build_activity_from_cache(request, &cache)?)
}

/// Build a local NNS proposal cache list report.
pub fn build_nns_proposal_cache_list_report(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsProposalCacheListReport, NnsProposalHostError> {
    let status = build_nns_proposal_cache_status_report(request)?;
    let caches = status.cache.into_iter().collect::<Vec<_>>();
    Ok(NnsProposalCacheListReport {
        schema_version: NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: status.network,
        collection: status.collection,
        cache_root: status.cache_root,
        cache_count: caches.len(),
        caches,
        refresh_attempt_path: status.refresh_attempt_path,
        latest_attempt: status.latest_attempt,
    })
}

/// Build a local NNS proposal cache status report.
pub fn build_nns_proposal_cache_status_report(
    request: &NnsGovernanceCacheRequest,
) -> Result<NnsGovernanceCacheStatusReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.cache_root, &request.network);
    build_governance_cache_status_report(
        request,
        "proposals",
        NNS_PROPOSAL_CACHE_COMPONENT,
        &paths,
        |file| {
            load_nns_proposal_cache_summary(
                &request.cache_root,
                paths.snapshot_path.clone(),
                &request.network,
                file,
            )
        },
    )
}

/// Build an NNS proposal list report from a complete local proposal snapshot.
pub fn build_nns_proposal_list_report_from_cache(
    request: &NnsProposalListRequest,
    cache_root: &Path,
) -> Result<Option<NnsProposalListReport>, NnsProposalHostError> {
    enforce_mainnet_network(&request.governance.network)?;
    let Some((path, cache)) = load_optional_cache(cache_root, &request.governance.network)? else {
        return Ok(None);
    };
    Ok(Some(nns_proposal_list_report_from_cache(
        request, path, cache,
    )))
}

/// Build an NNS proposal detail report from a complete local proposal snapshot.
pub fn build_nns_proposal_report_from_cache(
    request: &NnsProposalRequest,
    cache_root: &Path,
) -> Result<Option<NnsProposalReport>, NnsProposalHostError> {
    enforce_mainnet_network(&request.governance.network)?;
    let Some((path, cache)) = load_optional_cache(cache_root, &request.governance.network)? else {
        return Ok(None);
    };
    Ok(nns_proposal_report_from_cache(request, path, cache))
}

fn load_nns_proposal_cache_summary(
    cache_root: &Path,
    cache_path: PathBuf,
    network: &str,
    file: File,
) -> NnsGovernanceCacheSummary {
    match load_nns_proposal_cache(cache_root, cache_path.clone(), network, Some(file)) {
        Ok(cache) => NnsGovernanceCacheSummary::from_snapshot(&cache_path, &cache.snapshot),
        Err(error) => NnsGovernanceCacheSummary::invalid(&cache_path, error.to_string()),
    }
}

fn load_nns_proposal_cache(
    cache_root: &Path,
    cache_path: PathBuf,
    network: &str,
    opened_file: Option<File>,
) -> Result<AdmittedProposalCache, NnsProposalHostError> {
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
        missing_proposal_cache_error,
        incomplete_snapshot_error,
        |mismatch| nns_identity_mismatch_error(cache_path.clone(), mismatch),
        opened_file,
    )?;
    validate_nns_proposal_cache(&cache_path, &cache)?;
    Ok(AdmittedProposalCache { snapshot: cache })
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
    if !crate::nns::governance::collection::governance_snapshot_matches!(
        cache,
        state,
        proposals_fetched
    ) {
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
    }
    Ok(())
}

fn nns_proposal_list_report_from_cache(
    request: &NnsProposalListRequest,
    cache_path: PathBuf,
    cache: NnsProposalCache,
) -> NnsProposalListReport {
    let context = NnsGovernanceReportContext::from_snapshot(&cache);
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
        ReportProvenance::cache(&cache_path),
        proposals,
    )
}

fn nns_proposal_report_from_cache(
    request: &NnsProposalRequest,
    cache_path: PathBuf,
    cache: NnsProposalCache,
) -> Option<NnsProposalReport> {
    let context = NnsGovernanceReportContext::from_snapshot(&cache);
    let proposal = cache
        .data
        .proposals
        .into_iter()
        .find(|proposal| proposal.proposal_id == Some(request.proposal_id))?;
    Some(nns_proposal_report_from_row(
        request,
        context,
        ReportProvenance::cache(&cache_path),
        proposal,
    ))
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

fn load_optional_cache(
    cache_root: &Path,
    network: &str,
) -> Result<Option<(PathBuf, NnsProposalCache)>, NnsProposalHostError> {
    let path = nns_proposal_cache_paths(cache_root, network).snapshot_path;
    match load_nns_proposal_cache(cache_root, path.clone(), network, None) {
        Ok(cache) => Ok(Some((path, cache.snapshot))),
        Err(NnsProposalHostError::MissingProposalCache { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}
