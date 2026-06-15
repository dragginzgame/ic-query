use crate::sns::report::{
    SnsNeuronRow, hex_bytes,
    neurons_cache::model::CompleteSnsNeurons,
    source::{MainnetSnsNeuronPage, SnsNeuronId},
};
use std::collections::HashSet;

pub(super) struct SnsNeuronsCollectionState {
    neurons: Vec<SnsNeuronRow>,
    seen: HashSet<String>,
    page_count: u32,
    start_page_at: Option<SnsNeuronId>,
}

pub(super) struct SnsNeuronsCollectionPage {
    page_len: usize,
    new_rows: usize,
    pub(super) last_cursor_text: Option<String>,
}

impl SnsNeuronsCollectionState {
    pub(super) fn new() -> Self {
        Self {
            neurons: Vec::new(),
            seen: HashSet::new(),
            page_count: 0,
            start_page_at: None,
        }
    }

    pub(super) const fn page_count(&self) -> u32 {
        self.page_count
    }

    pub(super) const fn row_count(&self) -> usize {
        self.neurons.len()
    }

    pub(super) const fn start_page_at(&self) -> Option<&SnsNeuronId> {
        self.start_page_at.as_ref()
    }

    pub(super) const fn has_next_cursor(&self) -> bool {
        self.start_page_at.is_some()
    }

    pub(super) fn ingest_page(&mut self, page: MainnetSnsNeuronPage) -> SnsNeuronsCollectionPage {
        self.page_count = self.page_count.saturating_add(1);
        let page_len = page.neurons.len();
        let next_cursor = page.last_cursor;
        let last_cursor_text = next_cursor.as_ref().map(|cursor| hex_bytes(&cursor.id));
        let mut new_rows = 0_usize;
        for neuron in page.neurons {
            if self.seen.insert(neuron.neuron_id.clone()) {
                new_rows = new_rows.saturating_add(1);
                self.neurons.push(neuron);
            }
        }
        self.start_page_at = next_cursor;

        SnsNeuronsCollectionPage {
            page_len,
            new_rows,
            last_cursor_text,
        }
    }

    pub(super) fn into_complete(self) -> CompleteSnsNeurons {
        CompleteSnsNeurons {
            neurons: self.neurons,
            page_count: self.page_count,
            last_cursor: self
                .start_page_at
                .as_ref()
                .map(|cursor| hex_bytes(&cursor.id)),
        }
    }
}

impl SnsNeuronsCollectionPage {
    pub(super) fn exhausts_collection(&self, page_size: u32, has_next_cursor: bool) -> bool {
        self.page_len < usize::try_from(page_size).unwrap_or(usize::MAX)
            || !has_next_cursor
            || self.new_rows == 0
    }
}
