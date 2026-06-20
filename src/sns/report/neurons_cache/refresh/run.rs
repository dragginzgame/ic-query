//! Module: sns::report::neurons_cache::refresh::run
//!
//! Responsibility: run SNS neuron cache refresh operations.
//! Does not own: page collection internals, snapshot publishing details, text rendering, or CLI parsing.
//! Boundary: resolves the target SNS, acquires the refresh lock, and wraps attempt lifecycle hooks.

use super::{context::SnsNeuronsRefreshContext, publish::publish_complete_sns_neurons_cache};
use crate::{
    snapshot_cache::{
        LockedSnapshotRefreshRequest, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
    sns::report::{
        SnsHostError, SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest,
        live::LiveSnsSource,
        lookup::{enforce_mainnet_network, lookup_request_from_parts, resolve_sns_lookup},
        neurons_cache::{
            attempt::{write_failed_sns_neurons_attempt, write_starting_sns_neurons_attempt},
            collection::fetch_complete_sns_neurons,
            paths::SnsNeuronsCachePaths,
        },
        source::SnsNeuronsSource,
    },
};

const SNS_NEURONS_REFRESH_LOCK_STALE_AFTER_SECONDS: u64 = 30 * 60;

pub fn refresh_sns_neurons_cache(
    request: &SnsNeuronsRefreshRequest,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source(request, &LiveSnsSource)
}

pub(in crate::sns::report) fn refresh_sns_neurons_cache_with_source(
    request: &SnsNeuronsRefreshRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let paths = SnsNeuronsCachePaths::for_root(
        &request.icp_root,
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
            lock_stale_after_seconds: SNS_NEURONS_REFRESH_LOCK_STALE_AFTER_SECONDS,
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
            )
        },
    )
}

fn refresh_sns_neurons_cache_locked(
    context: SnsNeuronsRefreshContext<'_>,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    run_snapshot_refresh_with_attempts(
        || write_starting_attempt(&context),
        || {
            let complete = fetch_complete_sns_neurons(
                context.request,
                context.fetch_request,
                &context.sns,
                source,
                &context.paths.attempt_path,
            )?;
            publish_complete_sns_neurons_cache(&context, complete)
        },
        |err| write_failed_attempt(&context, err),
    )
}

fn write_starting_attempt(context: &SnsNeuronsRefreshContext<'_>) -> Result<(), SnsHostError> {
    write_starting_sns_neurons_attempt(context.attempt_context())
}

fn write_failed_attempt(context: &SnsNeuronsRefreshContext<'_>, err: &SnsHostError) {
    let _ = write_failed_sns_neurons_attempt(context.attempt_context(), err);
}
