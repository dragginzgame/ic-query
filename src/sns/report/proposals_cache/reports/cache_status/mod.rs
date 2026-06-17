//! Module: sns::report::proposals_cache::reports::cache_status
//!
//! Responsibility: build proposal cache status reports.
//! Does not own: cache loading internals, refresh orchestration, or rendering.
//! Boundary: routes id/root status lookups into public status report DTOs.

mod id;
mod report;
mod root;

use super::super::paths::sns_network_cache_dir;
use crate::sns::report::{
    SnsHostError, SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest,
    enforce_mainnet_network,
};

/// Build a local SNS proposal cache status report.
pub fn build_sns_proposals_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    if let Ok(id) = request.input.parse::<usize>() {
        return id::build_id_cache_status_report(request, cache_root.display().to_string(), id);
    }
    root::build_root_cache_status_report(request, cache_root.display().to_string())
}
