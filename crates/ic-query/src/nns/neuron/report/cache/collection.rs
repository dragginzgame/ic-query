//! Module: nns::neuron::report::cache::collection
//!
//! Responsibility: fetch complete public NNS neuron collections page by page.
//! Does not own: refresh locking, cache publication, or command parsing.
//! Boundary: drives neuron paging and refresh-attempt progress updates.

use super::{NNS_NEURON_CACHE_COMPONENT, model::CompleteNeuronCollection};
use crate::{
    QueryProgress,
    nns::{
        NnsGovernanceRefreshRequest,
        governance::{NnsGovernanceRequest, write_running_governance_refresh_attempt},
        neuron::report::{
            NNS_NEURON_FETCHED_BY, NnsNeuronHostError,
            collection::{NnsNeuronCollectionState, advance_nns_neuron_collection_with_source},
            model::NnsNeuronRow,
            source::NnsNeuronSource,
        },
    },
    runtime::block_on_current_thread,
    snapshot_cache::{
        PagedCollectionPage, PagedSnapshotRefresh, SnapshotRefreshProgress,
        run_paged_snapshot_refresh_with_progress,
    },
    subnet_catalog::MAINNET_NETWORK,
};
use std::path::Path;

pub(super) fn fetch_complete_neuron_collection(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
    attempt_path: &Path,
    progress: &mut dyn QueryProgress,
) -> Result<CompleteNeuronCollection, NnsNeuronHostError> {
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
    type Complete = CompleteNeuronCollection;
    type Error = NnsNeuronHostError;

    fn progress_text(&self) -> String {
        format!(
            "refreshing NNS neurons: pages={} rows={}",
            self.collection_state.pages_fetched(),
            self.collection_state.neurons_fetched()
        )
    }

    fn max_pages_reached(&self) -> bool {
        self.request
            .max_pages
            .is_some_and(|max_pages| self.collection_state.pages_fetched() >= max_pages)
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        NnsNeuronHostError::IncompleteRefresh {
            pages_fetched: self.collection_state.pages_fetched(),
            rows_fetched: self.neurons.len(),
            reason: reason.to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let step = block_on_current_thread(advance_nns_neuron_collection_with_source(
            &self.fetch_request,
            &self.collection_state,
            self.source,
        ))??;
        let page_len = step.page.neurons.len();
        let cursor = step.state.next_start_neuron_id();
        self.neurons.extend(step.page.neurons);
        self.collection_state = step.state;
        Ok(PagedCollectionPage::new(
            page_len,
            page_len,
            cursor.map(|cursor| cursor.to_string()),
        ))
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_governance_refresh_attempt(
            self.attempt_path,
            self.request,
            NNS_NEURON_CACHE_COMPONENT,
            SnapshotRefreshProgress::new(
                self.collection_state.pages_fetched(),
                self.neurons.len(),
                page.last_cursor_text.clone(),
            ),
        )
        .map_err(NnsNeuronHostError::from)
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(
            self.request.page_size,
            self.collection_state.next_start_neuron_id().is_some(),
        )
    }

    fn into_complete(self) -> Self::Complete {
        CompleteNeuronCollection {
            neurons: self.neurons,
            page_count: self.collection_state.pages_fetched(),
            last_cursor: self
                .collection_state
                .next_start_neuron_id()
                .map(|cursor| cursor.to_string()),
        }
    }
}
