//! Module: sns::report::proposals_cache::refresh::run
//!
//! Responsibility: run complete SNS proposal snapshot refreshes.
//! Does not own: cache publication details, attempt models, or text rendering.
//! Boundary: resolves lookup, acquires refresh lock, fetches pages, and publishes.

use super::{SnsProposalsRefreshContext, publish::publish_complete_sns_proposals_cache};
use crate::{
    QueryProgress,
    progress::IgnoreQueryProgress,
    snapshot_cache::run_snapshot_refresh_with_attempts,
    sns::report::{
        SnsHostError, SnsProposalsRefreshReport, SnsProposalsRefreshRequest,
        cache_attempt::{write_failed_sns_refresh_attempt, write_starting_sns_refresh_attempt},
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
pub fn refresh_sns_proposals_cache(
    request: &SnsProposalsRefreshRequest,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    refresh_sns_proposals_cache_with_source(request, &LiveSnsSource)
}

/// Refresh a complete SNS proposal snapshot and emit structured progress events.
pub fn refresh_sns_proposals_cache_with_progress(
    request: &SnsProposalsRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    refresh_sns_proposals_cache_with_source_and_progress(request, &LiveSnsSource, progress)
}

/// Refresh a complete SNS proposal snapshot using an injected source.
pub fn refresh_sns_proposals_cache_with_source(
    request: &SnsProposalsRefreshRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_sns_proposals_cache_with_source_and_progress(request, source, &mut progress)
}

pub(in crate::sns::report) fn refresh_sns_proposals_cache_with_source_and_progress(
    request: &SnsProposalsRefreshRequest,
    source: &dyn SnsProposalsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    run_resolved_sns_snapshot_refresh::<_, SnsProposalsCacheCollection, _>(
        request,
        source,
        DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS,
        |context| refresh_sns_proposals_cache_locked(context, source, progress),
    )
}

fn refresh_sns_proposals_cache_locked(
    context: SnsProposalsRefreshContext<'_>,
    source: &dyn SnsProposalsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    run_snapshot_refresh_with_attempts(
        || write_starting_sns_refresh_attempt(context.attempt_context()),
        || {
            let complete = fetch_complete_sns_proposals(
                context.request,
                &context.fetch_request,
                &context.sns,
                source,
                &context.paths.attempt_path,
                progress,
            )?;
            publish_complete_sns_proposals_cache(&context, complete)
        },
        |err| write_failed_sns_refresh_attempt(context.attempt_context(), err),
    )
}
