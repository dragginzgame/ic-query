//! Module: sns::report::proposals_cache::reports::cached_report
//!
//! Responsibility: build proposal list reports from complete local snapshots.
//! Does not own: refresh locking, live fetch paging, command parsing, or rendering.
//! Boundary: coordinates cache load/refresh policy and cache-backed report projection.

use crate::{
    cache_file::load_or_refresh_missing_cache,
    sns::report::{
        SnsHostError, SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsRefreshRequest,
        SnsProposalsReport, SnsProposalsRequest,
        assemble::{SnsProposalsReportParts, SnsReportProvenance, sns_proposals_report_from_parts},
        proposals_cache::{
            SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE, model::SnsProposalsCache,
            refresh_sns_proposals_cache_with_source,
            reports::cache_projection::project_sns_proposals_cache,
            storage::load_sns_proposals_cache_for_input_with_path,
        },
        source::{MainnetSnsProposals, SnsProposalsSource},
        view::{
            proposal_matches_before, proposal_matches_eligibility, proposal_matches_proposer,
            proposal_matches_status, proposal_matches_topic, sort_sns_proposal_rows,
        },
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
    let (cache_path, cache) = load_or_refresh_missing_cache(
        "SNS proposals",
        &request.source_endpoint,
        || load_sns_proposals_cache_for_input_with_path(icp_root, &request.network, &request.input),
        || {
            refresh_sns_proposals_cache_with_source(
                &proposals_refresh_request_from_list_request(request, icp_root),
                source,
            )
            .map(|_| ())
        },
        |err| match err {
            SnsHostError::MissingProposalsCache { path } => Ok(path),
            err => Err(err),
        },
    )?;
    if cache_lacks_fields_required_for_view(&cache, request) {
        eprintln!(
            "SNS proposals cache at {} lacks fields required for this proposal view; calling {} to refresh cache",
            cache_path.display(),
            request.source_endpoint
        );
        refresh_sns_proposals_cache_with_source(
            &proposals_refresh_request_from_list_request(request, icp_root),
            source,
        )?;
        let refreshed = load_sns_proposals_cache_for_input_with_path(
            icp_root,
            &request.network,
            &request.input,
        )?;
        if cache_lacks_fields_required_for_view(&refreshed.1, request) {
            return Err(SnsHostError::UnsupportedProposalView {
                reason: "refreshed SNS proposal cache lacks fields required for this proposal view"
                    .to_string(),
            });
        }
        return Ok(refreshed);
    }
    Ok((cache_path, cache))
}

fn proposals_refresh_request_from_list_request(
    request: &SnsProposalsRequest,
    icp_root: &Path,
) -> SnsProposalsRefreshRequest {
    SnsProposalsRefreshRequest {
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
        input: request.input.clone(),
        icp_root: icp_root.to_path_buf(),
        page_size: SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE,
        max_pages: None,
    }
}

const fn proposal_status_requires_raw_status(status: SnsProposalStatusFilter) -> bool {
    matches!(
        status,
        SnsProposalStatusFilter::Rejected | SnsProposalStatusFilter::Adopted
    )
}

fn cache_lacks_raw_status(cache: &SnsProposalsCache) -> bool {
    cache
        .data
        .proposals
        .iter()
        .any(|proposal| proposal.status.is_none())
}

const fn proposal_topic_requires_cached_topic(topic: SnsProposalTopicFilter) -> bool {
    !matches!(topic, SnsProposalTopicFilter::Any)
}

fn cache_lacks_cached_topic(cache: &SnsProposalsCache) -> bool {
    cache
        .data
        .proposals
        .iter()
        .any(|proposal| proposal.topic.is_none())
}

fn cache_lacks_fields_required_for_view(
    cache: &SnsProposalsCache,
    request: &SnsProposalsRequest,
) -> bool {
    (proposal_status_requires_raw_status(request.status) && cache_lacks_raw_status(cache))
        || (proposal_topic_requires_cached_topic(request.topic) && cache_lacks_cached_topic(cache))
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
        .filter(|proposal| proposal_matches_topic(proposal, request.topic))
        .filter(|proposal| proposal_matches_eligibility(proposal, request.eligibility))
        .filter(|proposal| {
            proposal_matches_proposer(proposal, request.proposer_neuron_id.as_deref())
        })
        .collect::<Vec<_>>();
    sort_sns_proposal_rows(&mut proposals, request.sort, request.sort_direction);
    proposals.truncate(usize::try_from(request.limit).unwrap_or(usize::MAX));
    sns_proposals_report_from_parts(SnsProposalsReportParts {
        list: projection.list,
        id: projection.id,
        sns: projection.sns,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        topic: request.topic,
        eligibility: request.eligibility,
        proposer_neuron_id: request.proposer_neuron_id.clone(),
        sort: request.sort,
        sort_direction: request.sort_direction,
        verbose: request.verbose,
        provenance: SnsReportProvenance::cache(&cache_path, cache_complete),
        proposals: MainnetSnsProposals { proposals },
    })
}
