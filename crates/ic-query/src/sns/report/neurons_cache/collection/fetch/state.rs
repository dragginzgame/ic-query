//! Module: sns::report::neurons_cache::collection::fetch::state
//!
//! Responsibility: maintain SNS neuron collection paging state.
//! Does not own: live fetching, cache writing, report construction, or CLI parsing.
//! Boundary: deduplicates rows, tracks cursors, and produces a complete neuron collection.

use crate::snapshot_cache::{PagedCollectionPage, PagedCollectionState};
use crate::sns::report::{
    SnsHostError, SnsNeuronRow, hex_bytes,
    neurons_cache::model::CompleteSnsNeurons,
    source::{MainnetSnsNeuronPage, SnsNeuronId, validate_mainnet_sns_neuron_page},
};

///
/// SnsNeuronsCollectionState
///
/// Accumulated page state for a complete SNS neuron snapshot refresh.
///

pub(super) struct SnsNeuronsCollectionState {
    pages: PagedCollectionState<SnsNeuronRow, SnsNeuronId>,
}

impl SnsNeuronsCollectionState {
    pub(super) fn new() -> Self {
        Self {
            pages: PagedCollectionState::new(),
        }
    }

    pub(super) const fn page_count(&self) -> u32 {
        self.pages.page_count()
    }

    pub(super) const fn row_count(&self) -> usize {
        self.pages.row_count()
    }

    pub(super) const fn start_page_at(&self) -> Option<&SnsNeuronId> {
        self.pages.next_cursor()
    }

    pub(super) const fn has_next_cursor(&self) -> bool {
        self.pages.has_next_cursor()
    }

    pub(super) fn ingest_page(
        &mut self,
        page: MainnetSnsNeuronPage,
        requested_limit: u32,
    ) -> Result<PagedCollectionPage, SnsHostError> {
        validate_mainnet_sns_neuron_page(&page, requested_limit)?;
        Ok(self.pages.ingest_page(
            page.neurons,
            page.last_cursor,
            |cursor| hex_bytes(&cursor.id),
            |neuron| neuron.neuron_id.clone(),
        ))
    }

    pub(super) fn into_complete(self) -> CompleteSnsNeurons {
        self.pages.into_complete(|cursor| hex_bytes(&cursor.id))
    }
}
