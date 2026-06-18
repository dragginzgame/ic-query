//! Module: sns::report::neurons_cache::refresh::publish
//!
//! Responsibility: publish complete SNS neuron snapshots and refresh reports.
//! Does not own: page fetching, lock acquisition, lookup, or text rendering.
//! Boundary: writes the complete cache JSON and marks the refresh attempt complete.

use super::context::SnsNeuronsRefreshContext;
use crate::{
    snapshot_cache::{SnapshotCompleteness, write_snapshot_json},
    sns::report::{
        SnsHostError, SnsNeuronRow, SnsNeuronsRefreshReport,
        cache_error::sns_cache_file_error,
        neurons_cache::{
            SNS_NEURONS_CACHE_SCHEMA_VERSION, SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION,
            attempt::{SnsNeuronsAttemptProgress, write_complete_sns_neurons_attempt},
            model::{
                CompleteSnsNeurons, SnsNeuronsCache, SnsNeuronsCacheMetadata, SnsNeuronsCacheRows,
            },
        },
        source::{MainnetSns, MainnetSnsList},
    },
};

pub(super) fn publish_complete_sns_neurons_cache(
    context: &SnsNeuronsRefreshContext<'_>,
    complete: CompleteSnsNeurons,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    let CompleteSnsNeurons {
        neurons,
        page_count,
        last_cursor,
    } = complete;
    let cache = sns_neurons_cache_from_parts(
        &context.list,
        context.id,
        &context.sns,
        context.request.page_size,
        page_count,
        neurons,
    );
    let neuron_count = cache.data.neurons.len();
    write_snapshot_json(
        &context.paths.cache_path,
        &cache,
        |path, source| SnsHostError::SerializeCache { path, source },
        sns_cache_file_error,
    )?;
    write_complete_sns_neurons_attempt(
        context.attempt_context(),
        SnsNeuronsAttemptProgress::new(page_count, neuron_count, last_cursor),
    )?;
    Ok(SnsNeuronsRefreshReport {
        schema_version: SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION,
        network: context.list.network.clone(),
        sns_wasm_canister_id: context.list.sns_wasm_canister_id.clone(),
        fetched_at: context.list.fetched_at.clone(),
        source_endpoint: context.list.source_endpoint.clone(),
        fetched_by: context.list.fetched_by.clone(),
        id: context.id,
        name: context.sns.name.clone(),
        root_canister_id: context.sns.root_canister_id.clone(),
        governance_canister_id: context.sns.governance_canister_id.clone(),
        cache_path: context.paths.cache_path.display().to_string(),
        refresh_lock_path: context.paths.lock_path.display().to_string(),
        refresh_attempt_path: context.paths.attempt_path.display().to_string(),
        page_size: context.request.page_size,
        page_count,
        neuron_count,
        complete: true,
        replaced_existing_cache: context.replaced_existing_cache,
        wrote_cache: true,
    })
}

fn sns_neurons_cache_from_parts(
    list: &MainnetSnsList,
    id: usize,
    sns: &MainnetSns,
    page_size: u32,
    page_count: u32,
    neurons: Vec<SnsNeuronRow>,
) -> SnsNeuronsCache {
    SnsNeuronsCache {
        schema_version: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        network: list.network.clone(),
        fetched_at: list.fetched_at.clone(),
        source_endpoint: list.source_endpoint.clone(),
        fetched_by: list.fetched_by.clone(),
        metadata: SnsNeuronsCacheMetadata {
            sns_wasm_canister_id: list.sns_wasm_canister_id.clone(),
            id,
            name: sns.name.clone(),
            root_canister_id: sns.root_canister_id.clone(),
            governance_canister_id: sns.governance_canister_id.clone(),
        },
        completeness: SnapshotCompleteness::api_exhausted(
            page_size,
            page_count,
            neurons.len(),
            false,
        ),
        data: SnsNeuronsCacheRows { neurons },
    }
}
