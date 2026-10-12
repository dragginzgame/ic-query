//! Module: sns::report::proposals_cache::refresh::run
//!
//! Responsibility: expose complete SNS proposal snapshot refresh entry points.
//! Does not own: shared locking and attempt mechanics, cache publication, or text rendering.
//! Boundary: adapts proposal collection and publication to the shared refresh lifecycle.

use super::publish::publish_complete_sns_proposals_cache;
use crate::{
    QueryProgress,
    progress::IgnoreQueryProgress,
    sns::report::{
        SnsHostError, SnsProposalsRefreshReport, SnsProposalsRefreshRequest,
        cache_refresh::run_resolved_sns_snapshot_refresh,
        live::LiveSnsSource,
        proposals_cache::{
            collection::fetch_complete_sns_proposals, paths::SnsProposalsCacheCollection,
        },
        source::SnsProposalsSource,
    },
};

pub const DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

/// Refresh a complete SNS proposal snapshot using the live SNS source.
pub async fn refresh_sns_proposals_cache(
    request: &SnsProposalsRefreshRequest,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    refresh_sns_proposals_cache_with_source(request, &LiveSnsSource::default()).await
}

/// Refresh a complete SNS proposal snapshot and emit structured progress events.
pub async fn refresh_sns_proposals_cache_with_progress(
    request: &SnsProposalsRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    refresh_sns_proposals_cache_with_source_and_progress(
        request,
        &LiveSnsSource::default(),
        progress,
    )
    .await
}

/// Refresh a complete SNS proposal snapshot using an injected source.
pub async fn refresh_sns_proposals_cache_with_source(
    request: &SnsProposalsRefreshRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_sns_proposals_cache_with_source_and_progress(request, source, &mut progress).await
}

pub(in crate::sns::report) async fn refresh_sns_proposals_cache_with_source_and_progress(
    request: &SnsProposalsRefreshRequest,
    source: &dyn SnsProposalsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    run_resolved_sns_snapshot_refresh::<_, SnsProposalsCacheCollection, _>(
        request,
        source,
        DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS,
        async |context| {
            let complete =
                fetch_complete_sns_proposals(context.refresh_context(), source, progress).await?;
            publish_complete_sns_proposals_cache(context, complete)
        },
    )
    .await
}
