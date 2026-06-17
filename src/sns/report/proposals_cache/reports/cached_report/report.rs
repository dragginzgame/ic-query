//! Module: sns::report::proposals_cache::reports::cached_report::report
//!
//! Responsibility: project complete proposal caches into list reports.
//! Does not own: cache loading, refresh policy, or live source calls.
//! Boundary: constructs report DTOs after applying cache-backed view filters.

use crate::sns::report::{
    SnsProposalsReport, SnsProposalsRequest,
    assemble::{SnsProposalsReportParts, SnsReportProvenance, sns_proposals_report_from_parts},
    proposals_cache::{
        model::SnsProposalsCache, reports::cache_projection::project_sns_proposals_cache,
    },
    source::MainnetSnsProposals,
    view::{proposal_matches_before, proposal_matches_status, sort_sns_proposal_rows},
};
use std::path::PathBuf;

pub(super) fn sns_proposals_report_from_cache(
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
