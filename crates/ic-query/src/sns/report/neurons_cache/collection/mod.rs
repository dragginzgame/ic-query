//! Module: sns::report::neurons_cache::collection
//!
//! Responsibility: drive paged SNS neuron collection refreshes.
//! Does not own: cache paths, snapshot publishing, report assembly, or CLI parsing.
//! Boundary: adapts SNS neuron page fetching to the shared paged snapshot runner.

use crate::{
    QueryProgress,
    snapshot_cache::{PagedCollectionPage, PagedCollectionState},
    sns::report::{
        SnsHostError, SnsNeuronRow,
        cache_attempt::SnsRefreshContext,
        cache_collection::{SnsCollectionPages, collect_sns_snapshot},
        hex_bytes,
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
    collect_sns_snapshot(context, SnsNeuronsRefreshPages { source }, progress).await
}

///
/// SnsNeuronsRefreshPages
///
/// Paged refresh runner state for complete SNS neuron collection.
///

struct SnsNeuronsRefreshPages<'a> {
    source: &'a dyn SnsNeuronsSource,
}

impl SnsCollectionPages for SnsNeuronsRefreshPages<'_> {
    type Row = SnsNeuronRow;
    type Cursor = SnsNeuronId;
    const COLLECTION: &'static str = "neurons";

    fn fetch_next_page<'a>(
        &'a mut self,
        context: SnsRefreshContext<'a>,
        pages: &'a mut PagedCollectionState<Self::Row, Self::Cursor>,
    ) -> crate::report::SourceFuture<'a, Result<PagedCollectionPage, SnsHostError>> {
        Box::pin(async move {
            let mut page = self
                .source
                .fetch_sns_neuron_page(
                    context.fetch_request,
                    context.sns,
                    context.request.page_size,
                    pages.next_cursor(),
                    None,
                )
                .await?;
            validate_mainnet_sns_neuron_page(
                &page,
                context.request.page_size,
                pages.next_cursor(),
            )?;
            let page_len = page.neurons.len();
            // Ordered pages can repeat only the requested boundary; retain its first observation.
            if let Some(cursor) = pages.next_cursor()
                && page
                    .neurons
                    .first()
                    .is_some_and(|neuron| neuron.neuron_id == hex_bytes(&cursor.id))
            {
                page.neurons.remove(0);
            }
            Ok(
                pages.ingest_page(page.neurons, page_len, page.last_cursor, |cursor| {
                    hex_bytes(&cursor.id)
                }),
            )
        })
    }

    fn cursor_text(cursor: &Self::Cursor) -> String {
        hex_bytes(&cursor.id)
    }
}
