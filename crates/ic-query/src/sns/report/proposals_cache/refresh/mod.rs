//! Module: sns::report::proposals_cache::refresh
//!
//! Responsibility: group complete SNS proposal snapshot refresh orchestration.
//! Does not own: command parsing, proposal text rendering, or cache status reports.
//! Boundary: re-exports refresh entry points while child modules own refresh phases.

mod publish;
mod run;

use crate::sns::report::{
    SnsProposalsRefreshRequest, cache_refresh::SnsSnapshotRefreshContext,
    proposals_cache::paths::SnsProposalsCacheCollection,
};

type SnsProposalsRefreshContext<'a> =
    SnsSnapshotRefreshContext<'a, SnsProposalsRefreshRequest, SnsProposalsCacheCollection>;

pub(in crate::sns::report) use run::refresh_sns_proposals_cache_with_source_and_progress;
pub use run::{
    DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS, refresh_sns_proposals_cache,
    refresh_sns_proposals_cache_with_progress, refresh_sns_proposals_cache_with_source,
};
