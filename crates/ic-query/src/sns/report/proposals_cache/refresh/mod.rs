//! Module: sns::report::proposals_cache::refresh
//!
//! Responsibility: expose complete SNS proposal snapshot refresh entry points.
//! Does not own: shared locking and attempt mechanics, cache publication, or text rendering.
//! Boundary: adapts proposal collection and publication to the shared refresh lifecycle.

use crate::{
    QueryProgress,
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
    refresh_sns_proposals_cache_with_source_and_progress(
        request,
        &LiveSnsSource::default(),
        &mut |_| {},
    )
    .await
}

/// Refresh with a caller-owned source and progress sink.
pub async fn refresh_sns_proposals_cache_with_source_and_progress(
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
