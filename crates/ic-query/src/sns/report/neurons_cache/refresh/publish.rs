//! Module: sns::report::neurons_cache::refresh::publish
//!
//! Responsibility: publish complete SNS neuron snapshots and refresh reports.
//! Does not own: page fetching, lock acquisition, lookup, or text rendering.
//! Boundary: writes the complete cache JSON and marks the refresh attempt complete.

use super::SnsNeuronsRefreshContext;
use crate::sns::report::{
    SnsHostError, SnsNeuronsRefreshReport,
    cache_refresh::publish_complete_sns_snapshot,
    neurons_cache::{
        SNS_NEURONS_CACHE_SCHEMA_VERSION, SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION,
        model::{CompleteSnsNeurons, SnsNeuronsCacheRows},
    },
};

pub(super) fn publish_complete_sns_neurons_cache(
    context: &SnsNeuronsRefreshContext<'_>,
    complete: CompleteSnsNeurons,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    let page_count = complete.page_count;
    let neuron_count = complete.rows.len();
    let attempt_finalization_error = publish_complete_sns_snapshot(
        context,
        SNS_NEURONS_CACHE_SCHEMA_VERSION,
        page_count,
        neuron_count,
        complete.last_cursor,
        SnsNeuronsCacheRows {
            neurons: complete.rows,
        },
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
        attempt_finalization_error,
    })
}
