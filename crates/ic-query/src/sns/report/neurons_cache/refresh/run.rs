//! Module: sns::report::neurons_cache::refresh::run
//!
//! Responsibility: run SNS neuron cache refresh operations.
//! Does not own: page collection internals, snapshot publishing details, text rendering, or CLI parsing.
//! Boundary: resolves the target SNS, acquires the refresh lock, and wraps attempt lifecycle hooks.

use super::{context::SnsNeuronsRefreshContext, publish::publish_complete_sns_neurons_cache};
use crate::{
    QueryProgress,
    progress::IgnoreQueryProgress,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
    sns::report::{
        SnsHostError, SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest,
        cache_attempt::{write_failed_sns_refresh_attempt, write_starting_sns_refresh_attempt},
        enforce_mainnet_network,
        live::LiveSnsSource,
        lookup::{lookup_request_from_parts, resolve_sns_lookup, validate_sns_refresh_page_size},
        neurons_cache::{collection::fetch_complete_sns_neurons, paths::SnsNeuronsCachePaths},
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
    validate_sns_refresh_page_size(request.page_size)?;
    enforce_mainnet_network(&request.network)?;
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let paths = SnsNeuronsCachePaths::for_root(
        &request.cache_root,
        &request.network,
        &lookup.sns.root_canister_id,
    );
    let context_paths = paths.clone();
    let fetch_request = lookup.fetch_request;
    let list = lookup.list;
    let id = lookup.id;
    let sns = lookup.sns;
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.cache_path,
            refresh_lock_path: &paths.lock_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS,
        },
        SnsHostError::Cache,
        |refresh_state| {
            refresh_sns_neurons_cache_locked(
                SnsNeuronsRefreshContext {
                    request,
                    fetch_request: &fetch_request,
                    list,
                    id,
                    sns,
                    paths: context_paths,
                    replaced_existing_cache: refresh_state.replaced_existing_snapshot,
                },
                source,
                progress,
            )
        },
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
                context.fetch_request,
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
