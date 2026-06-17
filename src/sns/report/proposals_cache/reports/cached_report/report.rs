//! Module: sns::report::proposals_cache::reports::cached_report::report
//!
//! Responsibility: project complete proposal caches into list reports.
//! Does not own: cache loading, refresh policy, or live source calls.
//! Boundary: constructs report DTOs after applying cache-backed view filters.

use super::super::super::model::SnsProposalsCache;
use super::super::cache_projection::project_sns_proposals_cache;
use super::filter::{proposal_matches_before, proposal_matches_status};
use crate::sns::report::{
    SnsProposalsReport, SnsProposalsRequest,
    assemble::{SnsProposalsReportParts, sns_proposals_report_from_parts},
    source::MainnetSnsProposals,
};

pub(super) fn sns_proposals_report_from_cache(
    request: &SnsProposalsRequest,
    cache: SnsProposalsCache,
) -> SnsProposalsReport {
    let projection = project_sns_proposals_cache(cache);
    let proposals = projection
        .proposals
        .into_iter()
        .filter(|proposal| proposal_matches_before(proposal, request.before_proposal_id))
        .filter(|proposal| proposal_matches_status(proposal, request.status))
        .take(usize::try_from(request.limit).unwrap_or(usize::MAX))
        .collect::<Vec<_>>();
    sns_proposals_report_from_parts(SnsProposalsReportParts {
        list: projection.list,
        id: projection.id,
        sns: projection.sns,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        topic: request.topic,
        verbose: request.verbose,
        proposals: MainnetSnsProposals { proposals },
    })
}
