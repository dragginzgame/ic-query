//! Module: sns::report::proposals_cache::reports::cached_report
//!
//! Responsibility: build proposal list reports from complete local snapshots.
//! Does not own: refresh locking, live fetch paging, command parsing, or rendering.
//! Boundary: coordinates cache load/refresh policy and cache-backed report projection.

use crate::{
    cache_file::load_or_refresh_missing_cache,
    sns::report::{
        SnsHostError, SnsProposalsRefreshRequest, SnsProposalsReport, SnsProposalsRequest,
        assemble::{SnsProposalsReportParts, SnsReportProvenance, sns_proposals_report_from_parts},
        proposals_cache::{
            SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE, model::SnsProposalsCache,
            refresh_sns_proposals_cache_with_source,
            reports::cache_projection::project_sns_proposals_cache,
            storage::load_sns_proposals_cache_for_input_with_path,
        },
        source::{MainnetSnsProposals, SnsProposalsSource},
        view::{proposal_matches_before, proposal_matches_status, sort_sns_proposal_rows},
    },
};
use std::path::{Path, PathBuf};

/// Build a proposal listing report from cache, refreshing when the cache is missing.
pub(in crate::sns::report) fn build_sns_proposals_report_from_cache_or_refresh(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    let (cache_path, cache) = load_or_refresh_sns_proposals_cache(request, icp_root, source)?;
    Ok(sns_proposals_report_from_cache(request, cache_path, cache))
}

fn load_or_refresh_sns_proposals_cache(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<(PathBuf, SnsProposalsCache), SnsHostError> {
    load_or_refresh_missing_cache(
        "SNS proposals",
        &request.source_endpoint,
        || load_sns_proposals_cache_for_input_with_path(icp_root, &request.network, &request.input),
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

fn sns_proposals_report_from_cache(
    request: &SnsProposalsRequest,
    cache_path: PathBuf,
    cache: SnsProposalsCache,
) -> SnsProposalsReport {
    let cache_complete = cache.completeness.is_api_exhausted();
    let projection = project_sns_proposals_cache(cache);
    let mut proposals = projection
        .proposals
        .into_iter()
        .filter(|proposal| proposal_matches_before(proposal, request.before_proposal_id))
        .filter(|proposal| proposal_matches_status(proposal, request.status))
        .collect::<Vec<_>>();
    sort_sns_proposal_rows(&mut proposals, request.sort);
    proposals.truncate(usize::try_from(request.limit).unwrap_or(usize::MAX));
    sns_proposals_report_from_parts(SnsProposalsReportParts {
        list: projection.list,
        id: projection.id,
        sns: projection.sns,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        topic: request.topic,
        sort: request.sort,
        verbose: request.verbose,
        provenance: SnsReportProvenance::cache(&cache_path, cache_complete),
        proposals: MainnetSnsProposals { proposals },
    })
}
