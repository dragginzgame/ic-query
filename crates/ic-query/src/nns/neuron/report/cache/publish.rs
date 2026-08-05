//! Module: nns::neuron::report::cache::publish
//!
//! Responsibility: publish complete public NNS neuron snapshots.
//! Does not own: refresh locking, live neuron paging, or command parsing.
//! Boundary: validates and writes complete cache JSON and attempt metadata.

use super::{
    NNS_NEURON_CACHE_COMPONENT, NNS_NEURON_CACHE_SCHEMA_VERSION,
    NNS_NEURON_REFRESH_REPORT_SCHEMA_VERSION,
    attempt::write_complete_attempt,
    cache_operation,
    model::{CompleteNeuronCollection, NnsNeuronCache, NnsNeuronCacheRows, NnsNeuronRefreshReport},
    paths::{NNS_NEURON_CACHE_COLLECTION, NNS_NEURON_CACHE_DOMAIN, NNS_NEURON_CACHE_ENTITY},
};
use crate::{
    HostCacheError,
    cache::CacheCollectionCompleteness,
    ic_registry::MAINNET_GOVERNANCE_CANISTER_ID,
    nns::{
        NnsGovernanceRefreshRequest,
        governance::mainnet_governance_cache_metadata,
        neuron::report::{NNS_NEURON_FETCHED_BY, NnsNeuronHostError, source::validate_neuron_rows},
    },
    snapshot_cache::{
        SnapshotJsonPaths, SnapshotRefreshProgress, publish_snapshot_with_attempt,
        write_snapshot_json,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};

pub(super) fn publish_complete_neuron_cache(
    request: &NnsGovernanceRefreshRequest,
    paths: &SnapshotJsonPaths,
    replaced_existing_cache: bool,
    complete: CompleteNeuronCollection,
) -> Result<NnsNeuronRefreshReport, NnsNeuronHostError> {
    let CompleteNeuronCollection {
        neurons,
        page_count,
        last_cursor,
    } = complete;
    validate_neuron_rows(&neurons)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let neuron_count = neurons.len();
    let cache = NnsNeuronCache {
        schema_version: NNS_NEURON_CACHE_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_at: fetched_at.clone(),
        fetched_by: NNS_NEURON_FETCHED_BY.to_string(),
        domain: NNS_NEURON_CACHE_DOMAIN.to_string(),
        entity: NNS_NEURON_CACHE_ENTITY.to_string(),
        collection: NNS_NEURON_CACHE_COLLECTION.to_string(),
        scope: "full".to_string(),
        metadata: mainnet_governance_cache_metadata(),
        completeness: CacheCollectionCompleteness::api_exhausted(
            request.page_size,
            page_count,
            neuron_count,
            false,
        ),
        data: NnsNeuronCacheRows { neurons },
    };
    let progress = SnapshotRefreshProgress::new(page_count, neuron_count, last_cursor);
    let attempt_finalization_error = publish_snapshot_with_attempt(
        || {
            write_snapshot_json(
                &request.cache_root,
                &paths.snapshot_path,
                &cache,
                |path, source| {
                    NnsNeuronHostError::Cache(HostCacheError::serialize_cache(
                        NNS_NEURON_CACHE_COMPONENT,
                        path,
                        source,
                    ))
                },
                cache_operation,
            )
        },
        || write_complete_attempt(&paths.refresh_attempt_path, request, progress),
    )?;
    Ok(NnsNeuronRefreshReport {
        schema_version: NNS_NEURON_REFRESH_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        neuron_count,
        page_size: request.page_size,
        page_count,
        complete: true,
        point_in_time_guaranteed: false,
        replaced_existing_cache,
        attempt_finalization_error,
        fetched_at,
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: NNS_NEURON_FETCHED_BY.to_string(),
        cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        refresh_lock_path: paths.refresh_lock_path.display().to_string(),
    })
}
