use super::super::{
    SNS_NEURONS_CACHE_SCHEMA_VERSION, SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION,
    attempt::{SnsNeuronsAttemptParts, attempt_from_parts, write_sns_neurons_attempt},
    errors::sns_cache_file_error,
    model::{CompleteSnsNeurons, SnsNeuronsCache, SnsNeuronsCacheMetadata, SnsNeuronsCacheRows},
};
use super::context::SnsNeuronsRefreshContext;
use crate::{
    cache_file::write_text_atomically,
    snapshot_cache::SnapshotCompleteness,
    sns::report::{
        SnsHostError, SnsNeuronRow, SnsNeuronsRefreshReport,
        source::{MainnetSns, MainnetSnsList},
    },
};

pub(super) fn publish_complete_sns_neurons_cache(
    context: SnsNeuronsRefreshContext<'_>,
    complete: CompleteSnsNeurons,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    let cache = sns_neurons_cache_from_parts(
        &context.list,
        context.id,
        &context.sns,
        context.request.page_size,
        complete.page_count,
        complete.neurons,
    );
    let neuron_count = cache.data.neurons.len();
    let cache_json =
        serde_json::to_string_pretty(&cache).map_err(|source| SnsHostError::SerializeCache {
            path: context.paths.cache_path.clone(),
            source,
        })?;
    write_text_atomically(&context.paths.cache_path, &cache_json).map_err(sns_cache_file_error)?;
    write_sns_neurons_attempt(
        &context.paths.attempt_path,
        &attempt_from_parts(SnsNeuronsAttemptParts {
            request: context.request,
            fetch_request: context.fetch_request,
            sns: &context.sns,
            status: "complete",
            pages_fetched: complete.page_count,
            rows_fetched: neuron_count,
            last_cursor: complete.last_cursor,
            last_error: None,
        }),
    )?;
    Ok(SnsNeuronsRefreshReport {
        schema_version: SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION,
        network: context.list.network,
        sns_wasm_canister_id: context.list.sns_wasm_canister_id,
        fetched_at: context.list.fetched_at,
        source_endpoint: context.list.source_endpoint,
        fetched_by: context.list.fetched_by,
        id: context.id,
        name: context.sns.name,
        root_canister_id: context.sns.root_canister_id,
        governance_canister_id: context.sns.governance_canister_id,
        cache_path: context.paths.cache_path.display().to_string(),
        refresh_lock_path: context.paths.lock_path.display().to_string(),
        refresh_attempt_path: context.paths.attempt_path.display().to_string(),
        page_size: context.request.page_size,
        page_count: complete.page_count,
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
