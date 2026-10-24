//! Module: sns::report::proposals_cache::refresh::run
//!
//! Responsibility: expose complete SNS proposal snapshot refresh entry points.
//! Does not own: shared locking and attempt mechanics, cache publication, or text rendering.
//! Boundary: adapts proposal collection and publication to the shared refresh lifecycle.

use crate::{
    QueryProgress,
    progress::IgnoreQueryProgress,
    sns::report::{
        SnsHostError, SnsSnapshotRefreshReport, SnsSnapshotRefreshRequest,
        cache_refresh::{publish_complete_sns_snapshot, run_resolved_sns_snapshot_refresh},
        live::LiveSnsSource,
        proposals_cache::{
            collection::fetch_complete_sns_proposals, paths::SnsProposalsCacheCollection,
        },
        source::SnsProposalsSource,
    },
};

/// Refresh a complete SNS proposal snapshot using the live SNS source.
pub async fn refresh_sns_proposals_cache(
    request: &SnsSnapshotRefreshRequest,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    refresh_sns_proposals_cache_with_source(request, &LiveSnsSource::default()).await
}

/// Refresh a complete SNS proposal snapshot and emit structured progress events.
pub async fn refresh_sns_proposals_cache_with_progress(
    request: &SnsSnapshotRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    refresh_sns_proposals_cache_with_source_and_progress(
        request,
        &LiveSnsSource::default(),
        progress,
    )
    .await
}

/// Refresh a complete SNS proposal snapshot using an injected source.
pub async fn refresh_sns_proposals_cache_with_source(
    request: &SnsSnapshotRefreshRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_sns_proposals_cache_with_source_and_progress(request, source, &mut progress).await
}

pub(in crate::sns::report) async fn refresh_sns_proposals_cache_with_source_and_progress(
    request: &SnsSnapshotRefreshRequest,
    source: &dyn SnsProposalsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    run_resolved_sns_snapshot_refresh::<SnsProposalsCacheCollection, _>(
        request,
        source,
        async |context| {
            let complete =
                fetch_complete_sns_proposals(context.refresh_context(), source, progress).await?;
            publish_complete_sns_snapshot(
                context,
                complete.page_count,
                complete.last_cursor,
                crate::sns::report::proposals_cache::model::SnsProposalsCacheRows {
                    proposals: complete.rows,
                },
            )
        },
    )
    .await
}
