//! Module: sns::report::neurons_cache::refresh::run
//!
//! Responsibility: run SNS neuron cache refresh operations.
//! Does not own: page collection internals, snapshot publishing details, text rendering, or CLI parsing.
//! Boundary: resolves the target SNS, acquires the refresh lock, and wraps attempt lifecycle hooks.

use super::{SnsNeuronsRefreshContext, publish::publish_complete_sns_neurons_cache};
use crate::{
    QueryProgress,
    progress::IgnoreQueryProgress,
    snapshot_cache::run_snapshot_refresh_with_attempts,
    sns::report::{
        SnsHostError, SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest,
        cache_attempt::{write_failed_sns_refresh_attempt, write_starting_sns_refresh_attempt},
        cache_refresh::run_resolved_sns_snapshot_refresh,
        live::LiveSnsSource,
        neurons_cache::{collection::fetch_complete_sns_neurons, paths::SnsNeuronsCacheCollection},
        source::SnsNeuronsSource,
    },
};

pub const DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

pub fn refresh_sns_neurons_cache(
    request: &SnsNeuronsRefreshRequest,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source(request, &LiveSnsSource)
}

/// Refresh a complete SNS neuron snapshot and emit structured progress events.
pub fn refresh_sns_neurons_cache_with_progress(
    request: &SnsNeuronsRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source_and_progress(request, &LiveSnsSource, progress)
}

pub fn refresh_sns_neurons_cache_with_source(
    request: &SnsNeuronsRefreshRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_sns_neurons_cache_with_source_and_progress(request, source, &mut progress)
}

fn refresh_sns_neurons_cache_with_source_and_progress(
    request: &SnsNeuronsRefreshRequest,
    source: &dyn SnsNeuronsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    run_resolved_sns_snapshot_refresh::<_, SnsNeuronsCacheCollection, _>(
        request,
        source,
        DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS,
        |context| refresh_sns_neurons_cache_locked(context, source, progress),
    )
}

fn refresh_sns_neurons_cache_locked(
    context: SnsNeuronsRefreshContext<'_>,
    source: &dyn SnsNeuronsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    run_snapshot_refresh_with_attempts(
        || write_starting_sns_refresh_attempt(context.attempt_context()),
        || {
            let complete = fetch_complete_sns_neurons(
                context.request,
                &context.fetch_request,
                &context.sns,
                source,
                &context.paths.attempt_path,
                progress,
            )?;
            publish_complete_sns_neurons_cache(&context, complete)
        },
        |err| write_failed_sns_refresh_attempt(context.attempt_context(), err),
    )
}
