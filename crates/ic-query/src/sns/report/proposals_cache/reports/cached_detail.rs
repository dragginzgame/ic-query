//! Module: sns::report::proposals_cache::reports::cached_detail
//!
//! Responsibility: build proposal detail reports from complete local snapshots.
//! Does not own: live proposal detail fetches, cache refresh, or text rendering.
//! Boundary: returns an optional report so callers can fall back to live lookup.

use crate::sns::report::{
    SnsHostError, SnsProposalReport, SnsProposalRequest,
    assemble::{SnsReportContext, SnsReportProvenance, sns_proposal_report_from_parts},
    cache_storage::load_sns_cache_for_root,
    proposals_cache::{model::SnsProposalsCache, paths::SnsProposalsCacheCollection},
};
use std::path::{Path, PathBuf};

/// Build a proposal detail report from a complete local proposal snapshot.
pub(in crate::sns::report) fn build_sns_proposal_report_from_cache(
    request: &SnsProposalRequest,
    cache_root: &Path,
) -> Result<Option<SnsProposalReport>, SnsHostError> {
    let (cache_path, cache) = match load_sns_cache_for_root::<SnsProposalsCacheCollection>(
        cache_root,
        &request.lookup.network,
        &request.lookup.input,
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
    let (context, data) = SnsReportContext::from_snapshot(cache);
    let proposal = data
        .proposals
        .into_iter()
        .find(|proposal| proposal.proposal_id == request.proposal_id)?;
    Some(sns_proposal_report_from_parts(
        request,
        context,
        SnsReportProvenance::cache(&cache_path),
        proposal,
    ))
}
