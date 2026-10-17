//! Module: sns::report::neurons_cache::refresh::run
//!
//! Responsibility: expose complete SNS neuron snapshot refresh entry points.
//! Does not own: shared locking and attempt mechanics, cache publication, or text rendering.
//! Boundary: adapts neuron collection and publication to the shared refresh lifecycle.

use crate::{
    QueryProgress,
    progress::IgnoreQueryProgress,
    sns::report::{
        SnsHostError, SnsSnapshotRefreshReport, SnsSnapshotRefreshRequest,
        cache_refresh::{publish_complete_sns_snapshot, run_resolved_sns_snapshot_refresh},
        live::LiveSnsSource,
        neurons_cache::{collection::fetch_complete_sns_neurons, paths::SnsNeuronsCacheCollection},
        source::SnsNeuronsSource,
    },
};

pub async fn refresh_sns_neurons_cache(
    request: &SnsSnapshotRefreshRequest,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source(request, &LiveSnsSource::default()).await
}

/// Refresh a complete SNS neuron snapshot and emit structured progress events.
pub async fn refresh_sns_neurons_cache_with_progress(
    request: &SnsSnapshotRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source_and_progress(request, &LiveSnsSource::default(), progress)
        .await
}

pub async fn refresh_sns_neurons_cache_with_source(
    request: &SnsSnapshotRefreshRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_sns_neurons_cache_with_source_and_progress(request, source, &mut progress).await
}

async fn refresh_sns_neurons_cache_with_source_and_progress(
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
                crate::sns::report::neurons_cache::SNS_NEURONS_CACHE_SCHEMA_VERSION,
                complete.page_count,
                complete.rows.len(),
                complete.last_cursor,
                crate::sns::report::neurons_cache::model::SnsNeuronsCacheRows {
                    neurons: complete.rows,
                },
            )
        },
    )
    .await
}
