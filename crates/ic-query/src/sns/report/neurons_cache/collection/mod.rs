//! Drive complete SNS neuron refreshes through the shared paged snapshot runner.

use crate::{
    QueryProgress,
    report::SourceFuture,
    snapshot_cache::{
        PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh,
        run_paged_snapshot_refresh_with_progress,
    },
    sns::report::{
        SnsHostError, SnsNeuronRow,
        cache_attempt::write_running_sns_refresh_page,
        hex_bytes,
        neurons_cache::SnsRefreshContext,
        neurons_cache::model::CompleteSnsNeurons,
        source::{SnsNeuronId, SnsNeuronsSource, validate_mainnet_sns_neuron_page},
    },
};

/// Fetch every neuron page required for a complete SNS neuron snapshot.
pub(in crate::sns::report::neurons_cache) async fn fetch_complete_sns_neurons(
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
    .await
}

struct SnsNeuronsRefreshPages<'a> {
    context: SnsRefreshContext<'a>,
    source: &'a dyn SnsNeuronsSource,
    pages: PagedCollectionState<SnsNeuronRow, SnsNeuronId>,
}

impl PagedSnapshotRefresh for SnsNeuronsRefreshPages<'_> {
    type Complete = CompleteSnsNeurons;
    type Error = SnsHostError;

    fn progress(&self) -> crate::PagedQueryProgress {
        crate::PagedQueryProgress {
            collection: crate::QueryProgressCollection::SnsNeurons,
            target: Some(self.context.governance_canister_id.to_string()),
            pages_fetched: self.pages.page_count(),
            rows_fetched: self.pages.row_count(),
        }
    }

    fn max_pages_reached(&self) -> bool {
        self.context
            .request
            .max_pages
            .is_some_and(|maximum| self.pages.page_count() >= maximum)
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        SnsHostError::IncompleteRefresh {
            pages_fetched: self.pages.page_count(),
            rows_fetched: self.pages.row_count(),
            reason: reason.to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> SourceFuture<'_, Result<PagedCollectionPage, Self::Error>> {
        Box::pin(async move {
            let mut page = self
                .source
                .fetch_sns_neuron_page(
                    self.context.fetch_request,
                    self.context.governance_canister_id,
                    self.context.request.page_size,
                    self.pages.next_cursor(),
                    None,
                )
                .await?;
            validate_mainnet_sns_neuron_page(
                &page,
                self.context.request.page_size,
                self.pages.next_cursor(),
            )?;
            let raw_page_len = page.neurons.len();
            // Ordered pages can repeat only the requested boundary; retain its first observation.
            if let Some(cursor) = self.pages.next_cursor()
                && page
                    .neurons
                    .first()
                    .is_some_and(|neuron| neuron.neuron_id == hex_bytes(&cursor.id))
            {
                page.neurons.remove(0);
            }
            let exhausted = raw_page_len
                < usize::try_from(self.context.request.page_size).unwrap_or(usize::MAX)
                || page.last_cursor.is_none();
            let result = PagedCollectionPage::new(page.neurons.len(), exhausted);
            self.pages.ingest(page.neurons, page.last_cursor);
            Ok(result)
        })
    }

    fn write_running_attempt(&self) -> Result<(), Self::Error> {
        write_running_sns_refresh_page(
            self.context,
            self.pages.page_count(),
            self.pages.row_count(),
            self.pages.next_cursor().map(|cursor| hex_bytes(&cursor.id)),
        )
    }

    fn into_complete(self) -> Self::Complete {
        self.pages.into_complete(|cursor| hex_bytes(&cursor.id))
    }
}
