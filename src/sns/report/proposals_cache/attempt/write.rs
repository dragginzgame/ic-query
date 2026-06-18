//! Module: sns::report::proposals_cache::attempt::write
//!
//! Responsibility: write proposal refresh-attempt status metadata.
//! Does not own: live proposal paging, cache publication, or status report rendering.
//! Boundary: persists running and complete refresh-attempt states.

use super::model::{
    SnsProposalsAttemptContext, SnsProposalsAttemptParts, SnsProposalsAttemptProgress,
    attempt_from_parts,
};
use crate::{
    snapshot_cache::write_snapshot_refresh_attempt,
    sns::report::{
        SnsHostError,
        proposals_cache::{errors::sns_cache_file_error, model::SnsProposalsRefreshAttempt},
    },
};
use std::path::Path;

/// Write the initial running proposal refresh-attempt file.
pub(in crate::sns::report::proposals_cache) fn write_starting_attempt(
    context: SnsProposalsAttemptContext<'_>,
) -> Result<(), SnsHostError> {
    write_attempt_status(
        context,
        "running",
        SnsProposalsAttemptProgress::starting(),
        None,
    )
}

/// Write updated running proposal refresh progress.
pub(in crate::sns::report::proposals_cache) fn write_running_attempt(
    context: SnsProposalsAttemptContext<'_>,
    progress: SnsProposalsAttemptProgress,
) -> Result<(), SnsHostError> {
    write_attempt_status(context, "running", progress, None)
}

/// Write successful proposal refresh completion metadata.
pub(in crate::sns::report::proposals_cache) fn write_complete_attempt(
    context: SnsProposalsAttemptContext<'_>,
    progress: SnsProposalsAttemptProgress,
) -> Result<(), SnsHostError> {
    write_attempt_status(context, "complete", progress, None)
}

/// Best-effort write of failed proposal refresh-attempt metadata.
pub(in crate::sns::report::proposals_cache) fn write_failed_attempt(
    context: SnsProposalsAttemptContext<'_>,
    err: &SnsHostError,
) {
    let _ = write_attempt_status(
        context,
        "failed",
        SnsProposalsAttemptProgress::starting(),
        Some(err.to_string()),
    );
}

pub(in crate::sns::report::proposals_cache::attempt) fn write_attempt_status(
    context: SnsProposalsAttemptContext<'_>,
    status: &'static str,
    progress: SnsProposalsAttemptProgress,
    last_error: Option<String>,
) -> Result<(), SnsHostError> {
    write_proposals_attempt(
        context.path,
        &attempt_from_parts(SnsProposalsAttemptParts {
            context,
            status,
            progress,
            last_error,
        }),
    )
}

fn write_proposals_attempt(
    path: &Path,
    attempt: &SnsProposalsRefreshAttempt,
) -> Result<(), SnsHostError> {
    write_snapshot_refresh_attempt(
        path,
        attempt,
        |path, source| SnsHostError::SerializeCache { path, source },
        sns_cache_file_error,
    )
}
