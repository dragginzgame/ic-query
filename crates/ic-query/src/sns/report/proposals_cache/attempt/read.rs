//! Module: sns::report::proposals_cache::attempt::read
//!
//! Responsibility: read proposal refresh-attempt status metadata.
//! Does not own: cache snapshot loading, refresh orchestration, or text rendering.
//! Boundary: maps stored refresh attempts into public status report DTOs.

use crate::{
    snapshot_cache::read_snapshot_refresh_attempt,
    sns::report::{
        SnsProposalsRefreshAttemptStatus, proposals_cache::model::SnsProposalsRefreshAttempt,
    },
};
use std::path::Path;

/// Read the latest proposal refresh-attempt status when present.
pub(in crate::sns::report::proposals_cache) fn read_sns_proposals_attempt_status(
    path: &Path,
) -> Option<SnsProposalsRefreshAttemptStatus> {
    let attempt = read_snapshot_refresh_attempt::<SnsProposalsRefreshAttempt>(path)?;
    Some(SnsProposalsRefreshAttemptStatus::from(attempt))
}
