//! Module: sns::report::proposals_cache::reports::cached_report
//!
//! Responsibility: build proposal list reports from complete local snapshots.
//! Does not own: refresh locking, live fetch paging, command parsing, or rendering.
//! Boundary: loads or auto-refreshes proposal snapshots and applies view filters.

use super::super::{
    SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE,
    model::SnsProposalsCache,
    paths::sns_network_cache_dir,
    refresh_sns_proposals_cache_with_source,
    storage::{
        expected_cache_path_for_root, find_sns_proposals_cache_by_id, load_sns_proposals_cache_at,
    },
};
use crate::{
    cache_file::load_or_refresh_missing_cache,
    sns::report::{
        SnsHostError, SnsProposalRow, SnsProposalStatusFilter, SnsProposalsRefreshRequest,
        SnsProposalsReport, SnsProposalsRequest,
        assemble::{SnsProposalsReportParts, sns_proposals_report_from_parts},
        source::{MainnetSns, MainnetSnsList, MainnetSnsProposals, SnsProposalsSource},
    },
};
use candid::Principal;
use std::path::Path;

/// Build a proposal listing report from cache, refreshing when the cache is missing.
pub(in crate::sns::report) fn build_sns_proposals_report_from_cache_or_refresh(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    let cache = load_or_refresh_sns_proposals_cache(request, icp_root, source)?;
    Ok(sns_proposals_report_from_cache(request, cache))
}

fn load_or_refresh_sns_proposals_cache(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsCache, SnsHostError> {
    load_or_refresh_missing_cache(
        "SNS proposals",
        &request.source_endpoint,
        || load_sns_proposals_cache_for_input(request, icp_root),
        || {
            refresh_sns_proposals_cache_with_source(
                &SnsProposalsRefreshRequest {
                    network: request.network.clone(),
                    source_endpoint: request.source_endpoint.clone(),
                    now_unix_secs: request.now_unix_secs,
                    input: request.input.clone(),
                    icp_root: icp_root.to_path_buf(),
                    page_size: SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE,
                    max_pages: None,
                },
                source,
            )
            .map(|_| ())
        },
        |err| match err {
            SnsHostError::MissingProposalsCache { path } => Ok(path),
            err => Err(err),
        },
    )
}

fn load_sns_proposals_cache_for_input(
    request: &SnsProposalsRequest,
    icp_root: &Path,
) -> Result<SnsProposalsCache, SnsHostError> {
    if let Ok(id) = request.input.parse::<usize>() {
        return find_sns_proposals_cache_by_id(icp_root, &request.network, id)?.map_or_else(
            || {
                Err(SnsHostError::MissingProposalsCache {
                    path: sns_network_cache_dir(icp_root, &request.network),
                })
            },
            |(_path, cache)| Ok(cache),
        );
    }

    let root_canister_id = Principal::from_text(&request.input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: request.input.clone(),
        })?
        .to_text();
    let cache_path = expected_cache_path_for_root(icp_root, &request.network, &root_canister_id);
    if !cache_path.is_file() {
        return Err(SnsHostError::MissingProposalsCache { path: cache_path });
    }
    load_sns_proposals_cache_at(cache_path, &request.network)
}

fn sns_proposals_report_from_cache(
    request: &SnsProposalsRequest,
    cache: SnsProposalsCache,
) -> SnsProposalsReport {
    let metadata = cache.metadata;
    let list = MainnetSnsList {
        network: request.network.clone(),
        sns_wasm_canister_id: metadata.sns_wasm_canister_id.clone(),
        fetched_at: cache.fetched_at,
        fetched_by: cache.fetched_by,
        source_endpoint: cache.source_endpoint,
        sns_instances: Vec::new(),
    };
    let sns = MainnetSns {
        id: metadata.id,
        name: metadata.name,
        description: None,
        url: None,
        root_canister_id: metadata.root_canister_id,
        governance_canister_id: metadata.governance_canister_id,
        ledger_canister_id: String::new(),
        swap_canister_id: String::new(),
        index_canister_id: String::new(),
        metadata_error: None,
    };
    let proposals = cache
        .data
        .proposals
        .into_iter()
        .filter(|proposal| proposal_matches_before(proposal, request.before_proposal_id))
        .filter(|proposal| proposal_matches_status(proposal, request.status))
        .take(usize::try_from(request.limit).unwrap_or(usize::MAX))
        .collect::<Vec<_>>();
    sns_proposals_report_from_parts(SnsProposalsReportParts {
        list,
        id: sns.id,
        sns,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        topic: request.topic,
        verbose: request.verbose,
        proposals: MainnetSnsProposals { proposals },
    })
}

fn proposal_matches_before(proposal: &SnsProposalRow, before_proposal_id: Option<u64>) -> bool {
    before_proposal_id.is_none_or(|before| {
        proposal
            .proposal_id
            .is_some_and(|proposal_id| proposal_id < before)
    })
}

fn proposal_matches_status(proposal: &SnsProposalRow, status: SnsProposalStatusFilter) -> bool {
    match status {
        SnsProposalStatusFilter::Any => true,
        SnsProposalStatusFilter::Open => proposal.decision_state == "open",
        SnsProposalStatusFilter::Executed => proposal.decision_state == "executed",
        SnsProposalStatusFilter::Failed => proposal.decision_state == "failed",
        SnsProposalStatusFilter::Rejected | SnsProposalStatusFilter::Adopted => false,
    }
}
