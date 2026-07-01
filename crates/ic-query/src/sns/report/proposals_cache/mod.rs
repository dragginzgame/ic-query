//! Module: sns::report::proposals_cache
//!
//! Responsibility: complete SNS proposal snapshot refresh and inspection.
//! Does not own: command parsing, live proposal conversion, or text rendering.
//! Boundary: stores complete proposal snapshots and refresh-attempt metadata.

mod attempt;
mod collection;
mod errors;
mod model;
mod paths;
mod refresh;
mod reports;
mod storage;

pub use paths::{
    sns_proposals_cache_path, sns_proposals_refresh_attempt_path, sns_proposals_refresh_lock_path,
};
pub use refresh::refresh_sns_proposals_cache_with_source;
pub use refresh::{DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS, refresh_sns_proposals_cache};
pub(in crate::sns::report) use reports::{
    build_sns_proposal_report_from_cache, build_sns_proposals_report_from_cache_or_refresh,
};
pub use reports::{build_sns_proposals_cache_list_report, build_sns_proposals_cache_status_report};

pub(super) const SNS_PROPOSALS_CACHE_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;

const SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE: u32 = 100;
