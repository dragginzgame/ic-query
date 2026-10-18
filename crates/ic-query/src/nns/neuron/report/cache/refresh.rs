//! Collect complete Governance snapshots through family-specific paging.

use super::{
    NNS_NEURON_CACHE_COMPONENT,
    collection::fetch_complete_neuron_collection,
    model::{CompleteNeuronCollection, NnsNeuronCacheRows},
    paths::nns_neuron_cache_paths,
};
use crate::{
    QueryProgress,
    nns::{
        LiveNnsSource, NnsGovernanceRefreshRequest,
        governance::{NnsGovernanceRefreshReport, run_governance_snapshot_refresh},
        neuron::report::{
            NNS_NEURON_FETCHED_BY, NnsNeuronHostError, enforce_mainnet_network,
            source::{NnsNeuronSource, validate_page_size},
        },
    },
    progress::IgnoreQueryProgress,
    snapshot_cache::SnapshotRefreshProgress,
};

/// Refresh the complete public neuron index using the built-in live source.
pub async fn refresh_nns_neuron_cache(
    request: &NnsGovernanceRefreshRequest,
) -> Result<NnsGovernanceRefreshReport, NnsNeuronHostError> {
    refresh_nns_neuron_cache_with_source(request, &LiveNnsSource::default()).await
}

/// Refresh the complete neuron index while emitting structured progress.
pub async fn refresh_nns_neuron_cache_with_progress(
    request: &NnsGovernanceRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsGovernanceRefreshReport, NnsNeuronHostError> {
    refresh_with_source_and_progress(request, &LiveNnsSource::default(), progress).await
}

/// Refresh the complete neuron index through a custom source.
pub async fn refresh_nns_neuron_cache_with_source(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
) -> Result<NnsGovernanceRefreshReport, NnsNeuronHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_with_source_and_progress(request, source, &mut progress).await
}

async fn refresh_with_source_and_progress(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsGovernanceRefreshReport, NnsNeuronHostError> {
    validate_page_size(request.page_size)?;
    enforce_mainnet_network(&request.network)?;
    let paths = nns_neuron_cache_paths(&request.cache_root, &request.network);
    run_governance_snapshot_refresh(
        request,
        &paths,
        NNS_NEURON_CACHE_COMPONENT,
        async |context| {
            let CompleteNeuronCollection { neurons, state } = fetch_complete_neuron_collection(
                request,
                source,
                &paths.refresh_attempt_path,
                progress,
            )
            .await?;
            let publication_progress = SnapshotRefreshProgress::new(
                state.pages_fetched(),
                neurons.len(),
                state
                    .next_start_neuron_id()
                    .map(|cursor| cursor.to_string()),
            );
            context.publish(
                "neurons",
                NNS_NEURON_FETCHED_BY,
                publication_progress,
                NnsNeuronCacheRows {
                    collection_state: state,
                    neurons,
                },
            )
        },
    )
    .await
}
