//! Module: sns::report::neurons_cache::collection::fetch
//!
//! Responsibility: drive paged SNS neuron collection refreshes.
//! Does not own: cache paths, snapshot publishing, report assembly, or CLI parsing.
//! Boundary: adapts SNS neuron page fetching to the shared paged snapshot runner.

mod state;

use crate::{
    QueryProgress,
    snapshot_cache::{
        PagedCollectionPage, PagedSnapshotRefresh, run_paged_snapshot_refresh_with_progress,
    },
    sns::report::{
        SnsHostError,
        cache_attempt::{SnsRefreshContext, write_running_sns_refresh_page},
        neurons_cache::model::CompleteSnsNeurons,
        source::SnsNeuronsSource,
    },
};
use state::SnsNeuronsCollectionState;

/// Fetch every neuron page required for a complete SNS neuron snapshot.
pub(in crate::sns::report::neurons_cache) fn fetch_complete_sns_neurons(
    context: SnsRefreshContext<'_>,
    source: &dyn SnsNeuronsSource,
    progress: &mut dyn QueryProgress,
) -> Result<CompleteSnsNeurons, SnsHostError> {
    run_paged_snapshot_refresh_with_progress(
        SnsNeuronsRefreshPages {
            context,
            source,
            state: SnsNeuronsCollectionState::new(),
        },
        progress,
    )
}

///
/// SnsNeuronsRefreshPages
///
/// Paged refresh runner state for complete SNS neuron collection.
///

struct SnsNeuronsRefreshPages<'a> {
    context: SnsRefreshContext<'a>,
    source: &'a dyn SnsNeuronsSource,
    state: SnsNeuronsCollectionState,
}

impl PagedSnapshotRefresh for SnsNeuronsRefreshPages<'_> {
    type Complete = CompleteSnsNeurons;
    type Error = SnsHostError;

    fn progress_text(&self) -> String {
        self.context
            .progress_text("neurons", self.state.page_count(), self.state.row_count())
    }

    fn max_pages_reached(&self) -> bool {
        self.context.max_pages_reached(self.state.page_count())
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        SnsRefreshContext::incomplete_refresh_error(
            self.state.page_count(),
            self.state.row_count(),
            reason,
        )
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let page = self.source.fetch_sns_neuron_page(
            self.context.fetch_request,
            self.context.sns,
            self.context.request.page_size(),
            self.state.start_page_at(),
            None,
        )?;
        self.state
            .ingest_page(page, self.context.request.page_size())
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_sns_refresh_page(
            self.context,
            self.state.page_count(),
            self.state.row_count(),
            page,
        )
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        self.context
            .page_exhausts_collection(page, self.state.has_next_cursor())
    }

    fn into_complete(self) -> Self::Complete {
        self.state.into_complete()
    }
}
