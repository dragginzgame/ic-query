//! Module: sns::report::proposals_cache::attempt::read
//!
//! Responsibility: read proposal refresh-attempt status metadata.
//! Does not own: cache snapshot loading, refresh orchestration, or text rendering.
//! Boundary: maps stored refresh attempts into public status report DTOs.

use super::super::model::SnsProposalsRefreshAttempt;
use crate::{
    snapshot_cache::read_snapshot_refresh_attempt, sns::report::SnsProposalsRefreshAttemptStatus,
};
use std::path::Path;

/// Read the latest proposal refresh-attempt status when present.
pub(in crate::sns::report::proposals_cache) fn read_sns_proposals_attempt_status(
    path: &Path,
) -> Option<SnsProposalsRefreshAttemptStatus> {
    let attempt = read_snapshot_refresh_attempt::<SnsProposalsRefreshAttempt>(path)?;
    Some(SnsProposalsRefreshAttemptStatus {
        status: attempt.status,
        started_at: attempt.started_at,
        updated_at: attempt.updated_at,
        page_size: attempt.page_size,
        pages_fetched: attempt.pages_fetched,
        rows_fetched: attempt.rows_fetched,
        last_cursor: attempt.last_cursor,
        last_error: attempt.last_error,
    })
}
