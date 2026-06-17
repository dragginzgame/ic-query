//! Module: sns::report::proposals_cache::reports::cached_detail
//!
//! Responsibility: build proposal detail reports from complete local snapshots.
//! Does not own: live proposal detail fetches, cache refresh, or text rendering.
//! Boundary: returns an optional report so callers can fall back to live lookup.

use super::super::storage::load_sns_proposals_cache_for_input;
use super::cache_projection::project_sns_proposals_cache;
use crate::sns::report::{
    MainnetSnsProposal, SnsHostError, SnsProposalReport, SnsProposalRequest,
    assemble::{SnsProposalReportParts, sns_proposal_report_from_parts},
};
use std::path::Path;

/// Build a proposal detail report from a complete local proposal snapshot.
pub(in crate::sns::report) fn build_sns_proposal_report_from_cache(
    request: &SnsProposalRequest,
    icp_root: &Path,
) -> Result<Option<SnsProposalReport>, SnsHostError> {
    let cache = match load_sns_proposals_cache_for_input(icp_root, &request.network, &request.input)
    {
        Ok(cache) => cache,
        Err(SnsHostError::MissingProposalsCache { .. }) => return Ok(None),
        Err(err) => return Err(err),
    };
    Ok(sns_proposal_report_from_cache(request, cache))
}

fn sns_proposal_report_from_cache(
    request: &SnsProposalRequest,
    cache: super::super::model::SnsProposalsCache,
) -> Option<SnsProposalReport> {
    let projection = project_sns_proposals_cache(cache);
    let proposal = projection
        .proposals
        .into_iter()
        .find(|proposal| proposal.proposal_id == Some(request.proposal_id))?;
    Some(sns_proposal_report_from_parts(SnsProposalReportParts {
        list: projection.list,
        id: projection.id,
        sns: projection.sns,
        proposal_id: request.proposal_id,
        verbose: request.verbose,
        show_ballots: request.show_ballots,
        proposal: MainnetSnsProposal { proposal },
    }))
}
