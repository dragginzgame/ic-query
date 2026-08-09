//! Module: nns::proposals::report::cache::refresh
//!
//! Responsibility: run complete NNS proposal snapshot refreshes.
//! Does not own: command parsing, text rendering, or live transport internals.
//! Boundary: acquires the refresh lock, fetches pages, and publishes snapshots.

use super::{
    NNS_PROPOSAL_CACHE_COMPONENT, collection::fetch_complete_nns_proposal_collection,
    model::NnsProposalRefreshReport, paths::nns_proposal_cache_paths,
    publish::publish_complete_nns_proposal_cache,
};
use crate::{
    HostCacheError, QueryProgress,
    nns::{
        LiveNnsSource, NnsGovernanceRefreshRequest,
        governance::{
            write_failed_governance_refresh_attempt, write_starting_governance_refresh_attempt,
        },
        proposals::report::{
            NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE, NnsProposalHostError, enforce_mainnet_network,
            source::NnsProposalSource,
        },
    },
    progress::IgnoreQueryProgress,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
};

pub const DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

/// Refresh a complete NNS proposal snapshot using the live NNS proposal source.
pub fn refresh_nns_proposal_cache(
    request: &NnsGovernanceRefreshRequest,
) -> Result<NnsProposalRefreshReport, NnsProposalHostError> {
    refresh_nns_proposal_cache_with_source(request, &LiveNnsSource)
}

/// Refresh a complete NNS proposal snapshot and emit structured progress events.
pub fn refresh_nns_proposal_cache_with_progress(
    request: &NnsGovernanceRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsProposalRefreshReport, NnsProposalHostError> {
    refresh_nns_proposal_cache_with_source_and_progress(request, &LiveNnsSource, progress)
}

pub fn refresh_nns_proposal_cache_with_source(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalRefreshReport, NnsProposalHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_nns_proposal_cache_with_source_and_progress(request, source, &mut progress)
}

pub(super) fn refresh_nns_proposal_cache_with_source_and_progress(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsProposalSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsProposalRefreshReport, NnsProposalHostError> {
    if !(1..=NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE).contains(&request.page_size) {
        return Err(NnsProposalHostError::InvalidRefreshPageSize {
            page_size: request.page_size,
            max_page_size: NNS_PROPOSAL_REFRESH_MAX_PAGE_SIZE,
        });
    }
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.cache_root, &request.network);
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            cache_root: &request.cache_root,
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS,
        },
        |error| {
            NnsProposalHostError::Cache(HostCacheError::operation(
                NNS_PROPOSAL_CACHE_COMPONENT,
                error,
            ))
        },
        |refresh_state| {
            run_snapshot_refresh_with_attempts(
                || {
                    write_starting_governance_refresh_attempt(
                        &paths.refresh_attempt_path,
                        request,
                        NNS_PROPOSAL_CACHE_COMPONENT,
                    )
                    .map_err(NnsProposalHostError::from)
                },
                || {
                    let complete = fetch_complete_nns_proposal_collection(
                        request,
                        source,
                        &paths.refresh_attempt_path,
                        progress,
                    )?;
                    publish_complete_nns_proposal_cache(
                        request,
                        &paths,
                        refresh_state.replaced_existing_snapshot,
                        complete,
                    )
                },
                |error| {
                    let _ = write_failed_governance_refresh_attempt(
                        &paths.refresh_attempt_path,
                        request,
                        NNS_PROPOSAL_CACHE_COMPONENT,
                        error.to_string(),
                    );
                },
            )
        },
    )
}
