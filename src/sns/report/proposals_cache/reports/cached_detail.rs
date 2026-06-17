//! Module: sns::report::proposals_cache::reports::cached_detail
//!
//! Responsibility: build proposal detail reports from complete local snapshots.
//! Does not own: live proposal detail fetches, cache refresh, or text rendering.
//! Boundary: returns an optional report so callers can fall back to live lookup.

use super::cache_projection::project_sns_proposals_cache;
use crate::sns::report::{
    MainnetSnsProposal, SnsHostError, SnsProposalReport, SnsProposalRequest,
    assemble::{SnsProposalReportParts, SnsReportProvenance, sns_proposal_report_from_parts},
    proposals_cache::{
        model::SnsProposalsCache, storage::load_sns_proposals_cache_for_input_with_path,
    },
};
use std::path::{Path, PathBuf};

/// Build a proposal detail report from a complete local proposal snapshot.
pub(in crate::sns::report) fn build_sns_proposal_report_from_cache(
    request: &SnsProposalRequest,
    icp_root: &Path,
) -> Result<Option<SnsProposalReport>, SnsHostError> {
    let (cache_path, cache) = match load_sns_proposals_cache_for_input_with_path(
        icp_root,
        &request.network,
        &request.input,
    ) {
        Ok(cache) => cache,
        Err(SnsHostError::MissingProposalsCache { .. }) => return Ok(None),
        Err(err) => return Err(err),
    };
    Ok(sns_proposal_report_from_cache(request, cache_path, cache))
}

fn sns_proposal_report_from_cache(
    request: &SnsProposalRequest,
    cache_path: PathBuf,
    cache: SnsProposalsCache,
) -> Option<SnsProposalReport> {
    let cache_complete = cache.completeness.is_api_exhausted();
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
        provenance: SnsReportProvenance::cache(&cache_path, cache_complete),
        proposal: MainnetSnsProposal { proposal },
    }))
}
