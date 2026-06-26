//! Module: sns::report::proposals_cache::refresh::run
//!
//! Responsibility: run complete SNS proposal snapshot refreshes.
//! Does not own: cache publication details, attempt models, or text rendering.
//! Boundary: resolves lookup, acquires refresh lock, fetches pages, and publishes.

use super::{context::SnsProposalsRefreshContext, publish::publish_complete_sns_proposals_cache};
use crate::{
    snapshot_cache::{
        LockedSnapshotRefreshRequest, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
    sns::report::{
        SnsHostError, SnsProposalsRefreshReport, SnsProposalsRefreshRequest,
        live::LiveSnsSource,
        lookup::{enforce_mainnet_network, lookup_request_from_parts, resolve_sns_lookup},
        proposals_cache::{
            attempt::{write_failed_attempt, write_starting_attempt},
            collection::fetch_complete_sns_proposals,
            paths::SnsProposalsCachePaths,
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

/// Refresh a complete SNS proposal snapshot using an injected source.
pub(in crate::sns::report) fn refresh_sns_proposals_cache_with_source(
    request: &SnsProposalsRefreshRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let paths = SnsProposalsCachePaths::for_root(
        &request.icp_root,
        &request.network,
        &lookup.sns.root_canister_id,
    );
    let context_paths = paths.clone();
    let fetch_request = lookup.fetch_request;
    let list = lookup.list;
    let id = lookup.id;
    let sns = lookup.sns;
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.cache_path,
            refresh_lock_path: &paths.lock_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS,
        },
        SnsHostError::Cache,
        |refresh_state| {
            refresh_sns_proposals_cache_locked(
                SnsProposalsRefreshContext {
                    request,
                    fetch_request: &fetch_request,
                    list,
                    id,
                    sns,
                    paths: context_paths,
                    replaced_existing_cache: refresh_state.replaced_existing_snapshot,
                },
                source,
            )
        },
    )
}

fn refresh_sns_proposals_cache_locked(
    context: SnsProposalsRefreshContext<'_>,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    run_snapshot_refresh_with_attempts(
        || write_starting_attempt(context.attempt_context()),
        || {
            let complete = fetch_complete_sns_proposals(
                context.request,
                context.fetch_request,
                &context.sns,
                source,
                &context.paths.attempt_path,
            )?;
            publish_complete_sns_proposals_cache(&context, complete)
        },
        |err| write_failed_attempt(context.attempt_context(), err),
    )
}
