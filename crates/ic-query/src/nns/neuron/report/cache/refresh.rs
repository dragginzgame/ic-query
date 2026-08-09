//! Module: nns::neuron::report::cache::refresh
//!
//! Responsibility: run complete public NNS neuron snapshot refreshes.
//! Does not own: command parsing, text rendering, or live transport internals.
//! Boundary: acquires the refresh lock, fetches pages, and publishes snapshots.

use super::{
    NNS_NEURON_CACHE_COMPONENT, cache_operation, collection::fetch_complete_neuron_collection,
    model::NnsNeuronRefreshReport, paths::nns_neuron_cache_paths,
    publish::publish_complete_neuron_cache,
};
use crate::{
    QueryProgress,
    nns::{
        LiveNnsSource, NnsGovernanceRefreshRequest,
        governance::{
            write_failed_governance_refresh_attempt, write_starting_governance_refresh_attempt,
        },
        neuron::report::{
            NnsNeuronHostError, enforce_mainnet_network,
            source::{NnsNeuronSource, validate_page_size},
        },
    },
    progress::IgnoreQueryProgress,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
};

/// Default age after which an NNS neuron refresh lock is reported as stale.
pub const DEFAULT_NNS_NEURON_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

/// Refresh the complete public neuron index using the built-in live source.
pub fn refresh_nns_neuron_cache(
    request: &NnsGovernanceRefreshRequest,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    refresh_nns_neuron_cache_with_source(request, &LiveNnsSource)
}

/// Refresh the complete neuron index while emitting structured progress.
pub fn refresh_nns_neuron_cache_with_progress(
    request: &NnsGovernanceRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    refresh_with_source_and_progress(request, &LiveNnsSource, progress)
}

/// Refresh the complete neuron index through a custom source.
pub fn refresh_nns_neuron_cache_with_source(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_with_source_and_progress(request, source, &mut progress)
}

fn refresh_with_source_and_progress(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    validate_page_size(request.page_size)?;
    enforce_mainnet_network(&request.network)?;
    let paths = nns_neuron_cache_paths(&request.cache_root, &request.network);
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            cache_root: &request.cache_root,
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: DEFAULT_NNS_NEURON_REFRESH_LOCK_STALE_SECONDS,
        },
        cache_operation,
        |state| {
            run_snapshot_refresh_with_attempts(
                || {
                    write_starting_governance_refresh_attempt(
                        &paths.refresh_attempt_path,
                        request,
                        NNS_NEURON_CACHE_COMPONENT,
                    )
                    .map_err(NnsNeuronHostError::from)
                },
                || {
                    let complete = fetch_complete_neuron_collection(
                        request,
                        source,
                        &paths.refresh_attempt_path,
                        progress,
                    )?;
                    publish_complete_neuron_cache(
                        request,
                        &paths,
                        state.replaced_existing_snapshot,
                        complete,
                    )
                },
                |error| {
                    let _ = write_failed_governance_refresh_attempt(
                        &paths.refresh_attempt_path,
                        request,
                        NNS_NEURON_CACHE_COMPONENT,
                        error.to_string(),
                    );
                },
            )
        },
    )
}
