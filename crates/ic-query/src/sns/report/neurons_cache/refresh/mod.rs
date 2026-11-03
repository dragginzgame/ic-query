//! Module: sns::report::neurons_cache::refresh
//!
//! Responsibility: expose complete SNS neuron snapshot refresh entry points.
//! Does not own: shared locking and attempt mechanics, cache publication, or text rendering.
//! Boundary: adapts neuron collection and publication to the shared refresh lifecycle.

use crate::{
    QueryProgress,
    sns::report::{
        SnsHostError, SnsSnapshotRefreshReport, SnsSnapshotRefreshRequest,
        cache_refresh::{publish_complete_sns_snapshot, run_resolved_sns_snapshot_refresh},
        live::LiveSnsSource,
        neurons_cache::{collection::fetch_complete_sns_neurons, paths::SnsNeuronsCacheCollection},
        source::SnsNeuronsSource,
    },
};

/// Refresh a complete SNS neuron snapshot using the live SNS source.
pub async fn refresh_sns_neurons_cache(
    request: &SnsSnapshotRefreshRequest,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source_and_progress(
        request,
        &LiveSnsSource::default(),
        &mut |_| {},
    )
    .await
}

/// Refresh with a caller-owned source and progress sink.
pub async fn refresh_sns_neurons_cache_with_source_and_progress(
    request: &SnsSnapshotRefreshRequest,
    source: &dyn SnsNeuronsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    run_resolved_sns_snapshot_refresh::<SnsNeuronsCacheCollection, _>(
        request,
        source,
        async |context| {
            let complete =
                fetch_complete_sns_neurons(context.refresh_context(), source, progress).await?;
            publish_complete_sns_snapshot(
                context,
                complete.page_count,
                complete.last_cursor,
                crate::sns::report::neurons_cache::model::SnsNeuronsCacheRows {
                    neurons: complete.rows,
                },
            )
        },
    )
    .await
}
