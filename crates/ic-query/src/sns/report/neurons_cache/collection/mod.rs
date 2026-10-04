//! Module: sns::report::neurons_cache::collection
//!
//! Responsibility: drive paged SNS neuron collection refreshes.
//! Does not own: cache paths, snapshot publishing, report assembly, or CLI parsing.
//! Boundary: adapts SNS neuron page fetching to the shared paged snapshot runner.

use crate::{
    QueryProgress,
    snapshot_cache::{
        PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh,
        run_paged_snapshot_refresh_with_progress,
    },
    sns::report::{
        SnsHostError, SnsNeuronRow,
        cache_attempt::{SnsRefreshContext, write_running_sns_refresh_page},
        hex_bytes,
        neurons_cache::model::CompleteSnsNeurons,
        source::{SnsNeuronId, SnsNeuronsSource, validate_mainnet_sns_neuron_page},
    },
};

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
            pages: PagedCollectionState::new(),
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
    pages: PagedCollectionState<SnsNeuronRow, SnsNeuronId>,
}

impl PagedSnapshotRefresh for SnsNeuronsRefreshPages<'_> {
    type Complete = CompleteSnsNeurons;
    type Error = SnsHostError;

    fn progress_text(&self) -> String {
        self.context
            .progress_text("neurons", self.pages.page_count(), self.pages.row_count())
    }

    fn max_pages_reached(&self) -> bool {
        self.context.max_pages_reached(self.pages.page_count())
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        SnsRefreshContext::incomplete_refresh_error(
            self.pages.page_count(),
            self.pages.row_count(),
            reason,
        )
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let mut page = self.source.fetch_sns_neuron_page(
            self.context.fetch_request,
            self.context.sns,
            self.context.request.page_size(),
            self.pages.next_cursor(),
            None,
        )?;
        validate_mainnet_sns_neuron_page(
            &page,
            self.context.request.page_size(),
            self.pages.next_cursor(),
        )?;
        let page_len = page.neurons.len();
        // Ordered pages can repeat only the requested boundary; retain its first observation.
        if let Some(cursor) = self.pages.next_cursor()
            && page
                .neurons
                .first()
                .is_some_and(|neuron| neuron.neuron_id == hex_bytes(&cursor.id))
        {
            page.neurons.remove(0);
        }
        Ok(self
            .pages
            .ingest_page(page.neurons, page_len, page.last_cursor, |cursor| {
                hex_bytes(&cursor.id)
            }))
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_sns_refresh_page(
            self.context,
            self.pages.page_count(),
            self.pages.row_count(),
            page,
        )
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        self.context
            .page_exhausts_collection(page, self.pages.has_next_cursor())
    }

    fn into_complete(self) -> Self::Complete {
        self.pages.into_complete(|cursor| hex_bytes(&cursor.id))
    }
}
