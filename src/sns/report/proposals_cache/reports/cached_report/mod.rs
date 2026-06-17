//! Module: sns::report::proposals_cache::reports::cached_report
//!
//! Responsibility: build proposal list reports from complete local snapshots.
//! Does not own: refresh locking, live fetch paging, command parsing, or rendering.
//! Boundary: coordinates cache load/refresh policy and cache-backed report projection.

mod filter;
mod load;
mod report;

use crate::sns::report::{
    SnsHostError, SnsProposalsReport, SnsProposalsRequest, source::SnsProposalsSource,
};
use std::path::Path;

use load::load_or_refresh_sns_proposals_cache;
use report::sns_proposals_report_from_cache;

/// Build a proposal listing report from cache, refreshing when the cache is missing.
pub(in crate::sns::report) fn build_sns_proposals_report_from_cache_or_refresh(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    let (cache_path, cache) = load_or_refresh_sns_proposals_cache(request, icp_root, source)?;
    Ok(sns_proposals_report_from_cache(request, cache_path, cache))
}
