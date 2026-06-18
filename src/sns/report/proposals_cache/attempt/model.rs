//! Module: sns::report::proposals_cache::attempt::model
//!
//! Responsibility: define proposal refresh-attempt writer inputs.
//! Does not own: attempt file IO, cache paths, or status report rendering.
//! Boundary: builds typed proposal refresh-attempt snapshots from refresh context.

use crate::{
    snapshot_cache::SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
    sns::report::{
        SnsProposalsRefreshRequest,
        proposals_cache::model::{SnsProposalsRefreshAttempt, SnsProposalsRefreshAttemptMetadata},
        source::{MainnetSns, SnsFetchRequest},
    },
};
use std::path::Path;

///
/// SnsProposalsAttemptContext
///
/// Shared context required to write one proposal refresh-attempt file.
///

#[derive(Clone, Copy)]
pub(in crate::sns::report::proposals_cache) struct SnsProposalsAttemptContext<'a> {
    pub(in crate::sns::report::proposals_cache) path: &'a Path,
    pub(in crate::sns::report::proposals_cache) request: &'a SnsProposalsRefreshRequest,
    pub(in crate::sns::report::proposals_cache) fetch_request: &'a SnsFetchRequest,
    pub(in crate::sns::report::proposals_cache) sns: &'a MainnetSns,
}

///
/// SnsProposalsAttemptProgress
///
/// Proposal refresh progress persisted into refresh-attempt metadata.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report::proposals_cache) struct SnsProposalsAttemptProgress {
    pub(in crate::sns::report::proposals_cache::attempt) pages_fetched: u32,
    pub(in crate::sns::report::proposals_cache::attempt) rows_fetched: usize,
    pub(in crate::sns::report::proposals_cache::attempt) last_cursor: Option<String>,
}

impl SnsProposalsAttemptProgress {
    /// Build refresh-attempt progress from current paging counters.
    pub(in crate::sns::report::proposals_cache) const fn new(
        pages_fetched: u32,
        rows_fetched: usize,
        last_cursor: Option<String>,
    ) -> Self {
        Self {
            pages_fetched,
            rows_fetched,
            last_cursor,
        }
    }

    pub(in crate::sns::report::proposals_cache::attempt) const fn starting() -> Self {
        Self {
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
        }
    }
}

pub(in crate::sns::report::proposals_cache::attempt) struct SnsProposalsAttemptParts<'a> {
    pub(in crate::sns::report::proposals_cache::attempt) context: SnsProposalsAttemptContext<'a>,
    pub(in crate::sns::report::proposals_cache::attempt) status: &'static str,
    pub(in crate::sns::report::proposals_cache::attempt) progress: SnsProposalsAttemptProgress,
    pub(in crate::sns::report::proposals_cache::attempt) last_error: Option<String>,
}

pub(in crate::sns::report::proposals_cache::attempt) fn attempt_from_parts(
    parts: SnsProposalsAttemptParts<'_>,
) -> SnsProposalsRefreshAttempt {
    SnsProposalsRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: parts.context.request.network.clone(),
        source_endpoint: parts.context.request.source_endpoint.clone(),
        started_at: parts.context.fetch_request.fetched_at.clone(),
        updated_at: parts.context.fetch_request.fetched_at.clone(),
        metadata: SnsProposalsRefreshAttemptMetadata {
            root_canister_id: parts.context.sns.root_canister_id.clone(),
            governance_canister_id: parts.context.sns.governance_canister_id.clone(),
        },
        status: parts.status.to_string(),
        page_size: parts.context.request.page_size,
        pages_fetched: parts.progress.pages_fetched,
        rows_fetched: parts.progress.rows_fetched,
        last_cursor: parts.progress.last_cursor,
        last_error: parts.last_error,
    }
}
