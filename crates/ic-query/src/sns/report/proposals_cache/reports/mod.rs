//! Module: sns::report::proposals_cache::reports
//!
//! Responsibility: group proposal cache report builders.
//! Does not own: refresh locking, page fetching, cache loading internals, or rendering.
//! Boundary: exposes cache-backed proposal list, cache list, and status reports.

mod cached_detail;
mod cached_report;

use super::SnsProposalsCacheCollection;
use crate::sns::report::{
    SnsCacheListReport, SnsCacheListRequest, SnsCacheStatusReport, SnsCacheStatusRequest,
    SnsHostError, build_sns_cache_list_report, cache_status::build_sns_cache_status_report,
};

pub(in crate::sns::report) use cached_detail::build_sns_proposal_report_from_cache;
pub(in crate::sns::report) use cached_report::build_sns_proposals_report_from_cache_or_refresh;

/// Build a local SNS proposal cache list report.
pub fn build_sns_proposals_cache_list_report(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError> {
    build_sns_cache_list_report::<SnsProposalsCacheCollection>(request)
}

/// Build an SNS proposal cache status report for one SNS input.
pub fn build_sns_proposals_cache_status_report(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError> {
    build_sns_cache_status_report::<SnsProposalsCacheCollection>(request)
}
