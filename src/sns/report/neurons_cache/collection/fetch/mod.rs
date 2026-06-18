//! Module: sns::report::neurons_cache::collection::fetch
//!
//! Responsibility: drive paged SNS neuron collection refreshes.
//! Does not own: cache paths, snapshot publishing, report assembly, or CLI parsing.
//! Boundary: adapts SNS neuron page fetching to the shared paged snapshot runner.

mod attempt;
mod state;

use crate::{
    snapshot_cache::{PagedCollectionPage, PagedSnapshotRefresh, run_paged_snapshot_refresh},
    sns::report::{
        SnsHostError, SnsNeuronsRefreshRequest,
        neurons_cache::model::CompleteSnsNeurons,
        source::{MainnetSns, SnsFetchRequest, SnsNeuronsSource},
    },
};
use state::SnsNeuronsCollectionState;
use std::path::Path;

pub(in crate::sns::report::neurons_cache) fn fetch_complete_sns_neurons(
    request: &SnsNeuronsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    source: &dyn SnsNeuronsSource,
    attempt_path: &Path,
) -> Result<CompleteSnsNeurons, SnsHostError> {
    run_paged_snapshot_refresh(SnsNeuronsRefreshPages {
        request,
        fetch_request,
        sns,
        source,
        attempt_path,
        state: SnsNeuronsCollectionState::new(),
    })
}

struct SnsNeuronsRefreshPages<'a> {
    request: &'a SnsNeuronsRefreshRequest,
    fetch_request: &'a SnsFetchRequest,
    sns: &'a MainnetSns,
    source: &'a dyn SnsNeuronsSource,
    attempt_path: &'a Path,
    state: SnsNeuronsCollectionState,
}

impl PagedSnapshotRefresh for SnsNeuronsRefreshPages<'_> {
    type Complete = CompleteSnsNeurons;
    type Error = SnsHostError;

    fn progress_text(&self) -> String {
        sns_neurons_progress_text(self.sns, self.state.page_count(), self.state.row_count())
    }

    fn max_pages_reached(&self) -> bool {
        self.request
            .max_pages
            .is_some_and(|max_pages| self.state.page_count() >= max_pages)
    }

    fn incomplete_refresh_error(&self) -> Self::Error {
        SnsHostError::IncompleteRefresh {
            pages_fetched: self.state.page_count(),
            rows_fetched: self.state.row_count(),
            reason: "max pages reached before API exhaustion".to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let page = self.source.fetch_sns_neuron_page(
            self.fetch_request,
            self.sns,
            self.request.page_size,
            self.state.start_page_at(),
            None,
        )?;
        Ok(self.state.ingest_page(page))
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        attempt::write_running_attempt(
            self.attempt_path,
            self.request,
            self.fetch_request,
            self.sns,
            &self.state,
            page,
        )
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(self.request.page_size, self.state.has_next_cursor())
    }

    fn into_complete(self) -> Self::Complete {
        self.state.into_complete()
    }
}

fn sns_neurons_progress_text(sns: &MainnetSns, pages: u32, rows: usize) -> String {
    format!(
        "refreshing SNS neurons for {}: pages={} rows={}",
        sns.name, pages, rows
    )
}
