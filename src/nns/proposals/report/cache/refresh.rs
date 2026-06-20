//! Module: nns::proposals::report::cache::refresh
//!
//! Responsibility: run complete NNS proposal snapshot refreshes.
//! Does not own: command parsing, text rendering, or live transport internals.
//! Boundary: acquires the refresh lock, fetches pages, and publishes snapshots.

use super::{
    attempt::{write_failed_attempt, write_starting_attempt},
    cache_file_error,
    collection::fetch_complete_nns_proposal_collection,
    model::{NnsProposalRefreshReport, NnsProposalRefreshRequest},
    paths::nns_proposal_cache_paths,
    publish::publish_complete_nns_proposal_cache,
};
use crate::{
    nns::proposals::report::{
        NnsProposalHostError, enforce_mainnet_network,
        source::{LiveNnsProposalSource, NnsProposalSource},
    },
    snapshot_cache::{
        LockedSnapshotRefreshRequest, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
};

const NNS_PROPOSAL_REFRESH_LOCK_STALE_AFTER_SECONDS: u64 = 30 * 60;

/// Refresh a complete NNS proposal snapshot using the live NNS proposal source.
pub(in crate::nns::proposals) fn refresh_nns_proposal_cache(
    request: &NnsProposalRefreshRequest,
) -> Result<NnsProposalRefreshReport, NnsProposalHostError> {
    refresh_nns_proposal_cache_with_source(request, &LiveNnsProposalSource)
}

pub(in crate::nns::proposals::report) fn refresh_nns_proposal_cache_with_source(
    request: &NnsProposalRefreshRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsProposalRefreshReport, NnsProposalHostError> {
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.icp_root, &request.network);
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: NNS_PROPOSAL_REFRESH_LOCK_STALE_AFTER_SECONDS,
        },
        cache_file_error,
        |refresh_state| {
            run_snapshot_refresh_with_attempts(
                || write_starting_attempt(&paths.refresh_attempt_path, request),
                || {
                    let complete = fetch_complete_nns_proposal_collection(
                        request,
                        source,
                        &paths.refresh_attempt_path,
                    )?;
                    publish_complete_nns_proposal_cache(
                        request,
                        &paths,
                        refresh_state.replaced_existing_snapshot,
                        complete,
                    )
                },
                |err| write_failed_attempt(&paths.refresh_attempt_path, request, err),
            )
        },
    )
}
