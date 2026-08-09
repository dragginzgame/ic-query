//! Module: nns::neuron::report::cache::collection
//!
//! Responsibility: fetch complete public NNS neuron collections page by page.
//! Does not own: refresh locking, cache publication, or command parsing.
//! Boundary: drives neuron paging and refresh-attempt progress updates.

use super::{NNS_NEURON_CACHE_COMPONENT, model::CompleteNeuronCollection};
use crate::{
    QueryProgress,
    nns::{
        NnsGovernanceRefreshRequest, NnsSourceRequest,
        governance::write_running_governance_refresh_attempt,
        neuron::report::{
            NNS_NEURON_FETCHED_BY, NnsNeuronHostError,
            model::NnsNeuronRow,
            source::{NnsNeuronSource, validate_neuron_page},
        },
    },
    snapshot_cache::{
        PagedCollectionPage, PagedSnapshotRefresh, SnapshotRefreshProgress,
        run_paged_snapshot_refresh_with_progress,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use std::path::Path;

pub(super) fn fetch_complete_neuron_collection(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsNeuronSource,
    attempt_path: &Path,
    progress: &mut dyn QueryProgress,
) -> Result<CompleteNeuronCollection, NnsNeuronHostError> {
    run_paged_snapshot_refresh_with_progress(
        NeuronRefreshPages {
            request,
            fetch_request: NnsSourceRequest::new(
                MAINNET_NETWORK,
                &request.source_endpoint,
                format_utc_timestamp_secs(request.now_unix_secs),
                NNS_NEURON_FETCHED_BY,
            ),
            source,
            attempt_path,
            neurons: Vec::new(),
            page_count: 0,
            next_cursor: None,
        },
        progress,
    )
}

struct NeuronRefreshPages<'a> {
    request: &'a NnsGovernanceRefreshRequest,
    fetch_request: NnsSourceRequest,
    source: &'a dyn NnsNeuronSource,
    attempt_path: &'a Path,
    neurons: Vec<NnsNeuronRow>,
    page_count: u32,
    next_cursor: Option<u64>,
}

impl PagedSnapshotRefresh for NeuronRefreshPages<'_> {
    type Complete = CompleteNeuronCollection;
    type Error = NnsNeuronHostError;

    fn progress_text(&self) -> String {
        format!(
            "refreshing NNS neurons: pages={} rows={}",
            self.page_count,
            self.neurons.len()
        )
    }

    fn max_pages_reached(&self) -> bool {
        self.request
            .max_pages
            .is_some_and(|max_pages| self.page_count >= max_pages)
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        NnsNeuronHostError::IncompleteRefresh {
            pages_fetched: self.page_count,
            rows_fetched: self.neurons.len(),
            reason: reason.to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let page = self.source.fetch_neuron_page(
            &self.fetch_request,
            self.next_cursor,
            self.request.page_size,
        )?;
        validate_neuron_page(&page, self.next_cursor, self.request.page_size)?;
        let page_len = page.neurons.len();
        let cursor = page.next_start_neuron_id;
        self.neurons.extend(page.neurons);
        self.page_count = self.page_count.saturating_add(1);
        self.next_cursor = cursor;
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
                self.page_count,
                self.neurons.len(),
                page.last_cursor_text.clone(),
            ),
        )
        .map_err(NnsNeuronHostError::from)
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(self.request.page_size, self.next_cursor.is_some())
    }

    fn into_complete(self) -> Self::Complete {
        CompleteNeuronCollection {
            neurons: self.neurons,
            page_count: self.page_count,
            last_cursor: self.next_cursor.map(|cursor| cursor.to_string()),
        }
    }
}
