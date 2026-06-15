use crate::snapshot_cache::{PagedCollectionPage, PagedCollectionState};
use crate::sns::report::{
    SnsNeuronRow, hex_bytes,
    neurons_cache::model::CompleteSnsNeurons,
    source::{MainnetSnsNeuronPage, SnsNeuronId},
};

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

    pub(super) fn ingest_page(&mut self, page: MainnetSnsNeuronPage) -> PagedCollectionPage {
        self.pages.ingest_page(
            page.neurons,
            page.last_cursor,
            |cursor| hex_bytes(&cursor.id),
            |neuron| neuron.neuron_id.clone(),
        )
    }

    pub(super) fn into_complete(self) -> CompleteSnsNeurons {
        let complete = self.pages.into_complete(|cursor| hex_bytes(&cursor.id));
        CompleteSnsNeurons {
            neurons: complete.rows,
            page_count: complete.page_count,
            last_cursor: complete.last_cursor,
        }
    }
}
