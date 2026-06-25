//! Module: sns::report::neurons_cache::attempt::write
//!
//! Responsibility: write SNS neuron refresh-attempt sidecars.
//! Does not own: attempt status projection, refresh fetching, cache publishing, or rendering.
//! Boundary: serializes refresh lifecycle states through the shared snapshot attempt writer.

use super::{
    model::{
        SnsNeuronsAttemptContext, SnsNeuronsAttemptParts, SnsNeuronsAttemptProgress,
        SnsNeuronsRefreshAttempt, attempt_from_parts,
    },
    read::read_sns_neurons_attempt,
};
use crate::{snapshot_cache::write_snapshot_refresh_attempt, sns::report::SnsHostError};
use std::path::Path;

pub(in crate::sns::report::neurons_cache) fn write_starting_sns_neurons_attempt(
    context: SnsNeuronsAttemptContext<'_>,
) -> Result<(), SnsHostError> {
    write_sns_neurons_attempt_status(
        context,
        "running",
        SnsNeuronsAttemptProgress::starting(),
        None,
    )
}

pub(in crate::sns::report::neurons_cache) fn write_running_sns_neurons_attempt(
    context: SnsNeuronsAttemptContext<'_>,
    progress: SnsNeuronsAttemptProgress,
) -> Result<(), SnsHostError> {
    write_sns_neurons_attempt_status(context, "running", progress, None)
}

pub(in crate::sns::report::neurons_cache) fn write_complete_sns_neurons_attempt(
    context: SnsNeuronsAttemptContext<'_>,
    progress: SnsNeuronsAttemptProgress,
) -> Result<(), SnsHostError> {
    write_sns_neurons_attempt_status(context, "complete", progress, None)
}

pub(in crate::sns::report::neurons_cache) fn write_failed_sns_neurons_attempt(
    context: SnsNeuronsAttemptContext<'_>,
    err: &SnsHostError,
) -> Result<(), SnsHostError> {
    let latest = read_sns_neurons_attempt(context.path);
    let progress = SnsNeuronsAttemptProgress::new(
        latest.as_ref().map_or(0, |attempt| attempt.pages_fetched),
        latest.as_ref().map_or(0, |attempt| attempt.rows_fetched),
        latest.and_then(|attempt| attempt.last_cursor),
    );
    write_sns_neurons_attempt_status(context, "failed", progress, Some(err.to_string()))
}

pub(in crate::sns::report::neurons_cache::attempt) fn write_sns_neurons_attempt_status(
    context: SnsNeuronsAttemptContext<'_>,
    status: &'static str,
    progress: SnsNeuronsAttemptProgress,
    last_error: Option<String>,
) -> Result<(), SnsHostError> {
    write_sns_neurons_attempt(
        context.path,
        &attempt_from_parts(SnsNeuronsAttemptParts {
            context,
            status,
            progress,
            last_error,
        }),
    )
}

fn write_sns_neurons_attempt(
    path: &Path,
    attempt: &SnsNeuronsRefreshAttempt,
) -> Result<(), SnsHostError> {
    write_snapshot_refresh_attempt(
        path,
        attempt,
        |path, source| SnsHostError::SerializeCache { path, source },
        SnsHostError::Cache,
    )
}
