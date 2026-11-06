//! Module: nns::neuron::report::cache::collection
//!
//! Responsibility: fetch complete public NNS neuron collections page by page.
//! Does not own: refresh locking, cache publication, or command parsing.
//! Boundary: drives neuron paging and refresh-attempt progress updates.

use super::{NNS_NEURON_CACHE_COMPONENT, model::NnsNeuronCacheRows};
use crate::{
    QueryProgress,
    nns::{
        NnsGovernanceRefreshRequest,
        governance::{NnsGovernanceRequest, governance_refresh_bookkeeping},
        neuron::report::{
            NNS_NEURON_FETCHED_BY, NnsNeuronHostError,
            collection::{NnsNeuronCollectionState, advance_nns_neuron_collection_with_source},
            model::NnsNeuronRow,
            source::NnsNeuronSource,
        },
    },
    snapshot_cache::{
        PagedCollectionPage, PagedSnapshotRefresh, SnapshotRefreshProgress,
        run_paged_snapshot_refresh_with_progress,
    },
    subnet_catalog::MAINNET_NETWORK,
};
use std::path::Path;

pub(super) async fn fetch_complete_neuron_collection(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
    attempt_path: &Path,
    progress: &mut dyn QueryProgress,
) -> Result<(NnsNeuronCacheRows, SnapshotRefreshProgress), NnsNeuronHostError> {
    let fetch_request = NnsGovernanceRequest::replica_query_from_unix_secs(
        MAINNET_NETWORK,
        &request.source_endpoint,
        request.now_unix_secs,
        NNS_NEURON_FETCHED_BY,
    );
    let collection_state =
        NnsNeuronCollectionState::new(&fetch_request, request.page_size, u32::MAX)?;
    run_paged_snapshot_refresh_with_progress(
        NeuronRefreshPages {
            request,
            fetch_request,
            source,
            attempt_path,
            collection_state,
            neurons: Vec::new(),
        },
        progress,
    )
    .await
}

struct NeuronRefreshPages<'a> {
    request: &'a NnsGovernanceRefreshRequest,
    fetch_request: NnsGovernanceRequest,
    source: &'a dyn NnsNeuronSource,
    attempt_path: &'a Path,
    collection_state: NnsNeuronCollectionState,
    neurons: Vec<NnsNeuronRow>,
}

impl PagedSnapshotRefresh for NeuronRefreshPages<'_> {
    type Complete = (NnsNeuronCacheRows, SnapshotRefreshProgress);
    type Error = NnsNeuronHostError;

    governance_refresh_bookkeeping!(
        NnsNeurons,
        NNS_NEURON_CACHE_COMPONENT,
        neurons,
        next_start_neuron_id
    );

    fn fetch_next_page(
        &mut self,
    ) -> crate::report::SourceFuture<'_, Result<PagedCollectionPage, Self::Error>> {
        Box::pin(async move {
            let step = advance_nns_neuron_collection_with_source(
                &self.fetch_request,
                &self.collection_state,
                self.source,
            )
            .await?;
            let page_len = step.page.neurons.len();
            self.neurons.extend(step.page.neurons);
            self.collection_state = step.state;
            Ok(PagedCollectionPage::new(
                page_len,
                self.collection_state.is_complete(),
            ))
        })
    }

    fn into_complete(self) -> Self::Complete {
        let progress = SnapshotRefreshProgress::new(
            self.collection_state.pages_fetched(),
            self.neurons.len(),
            None,
        );
        (
            NnsNeuronCacheRows {
                neurons: self.neurons,
                collection_evidence: self.collection_state.collection_evidence(),
            },
            progress,
        )
    }
}
