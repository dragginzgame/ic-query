//! Module: nns::proposals::report::cache::refresh
//!
//! Responsibility: run complete NNS proposal snapshot refreshes.
//! Does not own: command parsing, text rendering, or live transport internals.
//! Boundary: acquires the refresh lock, fetches pages, and publishes snapshots.

use super::super::{
    MAINNET_GOVERNANCE_CANISTER_ID, NnsProposalHostError, enforce_mainnet_network,
    model::NnsProposalRow,
    source::{
        LiveNnsProposalSource, NnsProposalFetchRequest, NnsProposalSource,
        nns_proposal_row_from_info,
    },
};
use super::{
    NNS_PROPOSAL_CACHE_SCHEMA_VERSION, NNS_PROPOSAL_REFRESH_REPORT_SCHEMA_VERSION,
    attempt::{
        NnsProposalAttemptProgress, write_complete_attempt, write_failed_attempt,
        write_running_attempt, write_starting_attempt,
    },
    cache_file_error,
    model::{
        CompleteNnsProposalCollection, NnsProposalCache, NnsProposalCacheMetadata,
        NnsProposalCacheRows, NnsProposalRefreshReport, NnsProposalRefreshRequest,
    },
    paths::nns_proposal_cache_paths,
};
use crate::{
    snapshot_cache::{
        LockedSnapshotRefreshRequest, PagedCollectionPage, PagedCollectionState,
        PagedSnapshotRefresh, SnapshotCompleteness, run_paged_snapshot_refresh,
        run_snapshot_refresh_with_attempts, with_locked_snapshot_refresh, write_snapshot_json,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use std::{cmp::Reverse, path::Path};

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

fn fetch_complete_nns_proposal_collection(
    request: &NnsProposalRefreshRequest,
    source: &dyn NnsProposalSource,
    attempt_path: &Path,
) -> Result<CompleteNnsProposalCollection, NnsProposalHostError> {
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    run_paged_snapshot_refresh(NnsProposalRefreshPages {
        request,
        fetch_request: NnsProposalFetchRequest {
            endpoint: request.source_endpoint.clone(),
            fetched_at,
            fetched_by: "ic-query".to_string(),
        },
        source,
        attempt_path,
        state: NnsProposalCollectionState::new(),
    })
}

///
/// NnsProposalRefreshPages
///
/// Paged refresh runner state for the complete NNS proposal collection.
///

struct NnsProposalRefreshPages<'a> {
    request: &'a NnsProposalRefreshRequest,
    fetch_request: NnsProposalFetchRequest,
    source: &'a dyn NnsProposalSource,
    attempt_path: &'a Path,
    state: NnsProposalCollectionState,
}

impl PagedSnapshotRefresh for NnsProposalRefreshPages<'_> {
    type Complete = CompleteNnsProposalCollection;
    type Error = NnsProposalHostError;

    fn progress_text(&self) -> String {
        format!(
            "refreshing NNS proposals: pages={} rows={}",
            self.state.page_count(),
            self.state.row_count()
        )
    }

    fn max_pages_reached(&self) -> bool {
        self.request
            .max_pages
            .is_some_and(|max_pages| self.state.page_count() >= max_pages)
    }

    fn incomplete_refresh_error(&self) -> Self::Error {
        NnsProposalHostError::IncompleteRefresh {
            pages_fetched: self.state.page_count(),
            rows_fetched: self.state.row_count(),
            reason: "max pages reached before API exhaustion".to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let proposal_infos = self.source.fetch_proposals(
            &self.fetch_request,
            self.request.page_size,
            self.state.before_proposal_id(),
            &[],
            &[],
        )?;
        self.state.ingest_page(
            proposal_infos
                .into_iter()
                .map(nns_proposal_row_from_info)
                .collect(),
        )
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_attempt(
            self.attempt_path,
            self.request,
            NnsProposalAttemptProgress::new(
                self.state.page_count(),
                self.state.row_count(),
                page.last_cursor_text.clone(),
            ),
        )
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(self.request.page_size, self.state.has_next_cursor())
    }

    fn into_complete(self) -> Self::Complete {
        self.state.into_complete()
    }
}

///
/// NnsProposalCollectionState
///
/// Page accumulator and cursor tracker for NNS proposal refreshes.
///

struct NnsProposalCollectionState {
    state: PagedCollectionState<NnsProposalRow, u64>,
}

impl NnsProposalCollectionState {
    fn new() -> Self {
        Self {
            state: PagedCollectionState::new(),
        }
    }

    const fn page_count(&self) -> u32 {
        self.state.page_count()
    }

    const fn row_count(&self) -> usize {
        self.state.row_count()
    }

    const fn before_proposal_id(&self) -> Option<u64> {
        self.state.next_cursor().copied()
    }

    const fn has_next_cursor(&self) -> bool {
        self.state.has_next_cursor()
    }

    fn ingest_page(
        &mut self,
        rows: Vec<NnsProposalRow>,
    ) -> Result<PagedCollectionPage, NnsProposalHostError> {
        if rows.iter().any(|row| row.proposal_id.is_none()) {
            return Err(NnsProposalHostError::MissingProposalIdInPage);
        }
        let next_cursor = rows
            .iter()
            .filter_map(|row| row.proposal_id)
            .min()
            .filter(|proposal_id| *proposal_id > 1);
        Ok(self
            .state
            .ingest_page(rows, next_cursor, ToString::to_string, row_id))
    }

    fn into_complete(self) -> CompleteNnsProposalCollection {
        let complete = self.state.into_complete(ToString::to_string);
        let mut proposals = complete.rows;
        proposals.sort_by_key(|proposal| Reverse(proposal.proposal_id));
        CompleteNnsProposalCollection {
            proposals,
            page_count: complete.page_count,
            last_cursor: complete.last_cursor,
        }
    }
}

fn row_id(row: &NnsProposalRow) -> String {
    row.proposal_id
        .map_or_else(|| "missing-proposal-id".to_string(), |id| id.to_string())
}

fn publish_complete_nns_proposal_cache(
    request: &NnsProposalRefreshRequest,
    paths: &crate::snapshot_cache::SnapshotJsonPaths,
    replaced_existing_cache: bool,
    complete: CompleteNnsProposalCollection,
) -> Result<NnsProposalRefreshReport, NnsProposalHostError> {
    let CompleteNnsProposalCollection {
        proposals,
        page_count,
        last_cursor,
    } = complete;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let cache = NnsProposalCache {
        schema_version: NNS_PROPOSAL_CACHE_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_at: fetched_at.clone(),
        fetched_by: "ic-query".to_string(),
        metadata: NnsProposalCacheMetadata {
            governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        },
        completeness: SnapshotCompleteness::api_exhausted(
            request.page_size,
            page_count,
            proposals.len(),
            false,
        ),
        data: NnsProposalCacheRows { proposals },
    };
    let proposal_count = cache.data.proposals.len();
    write_snapshot_json(
        &paths.snapshot_path,
        &cache,
        |path, source| NnsProposalHostError::SerializeCache { path, source },
        cache_file_error,
    )?;
    write_complete_attempt(
        &paths.refresh_attempt_path,
        request,
        NnsProposalAttemptProgress::new(page_count, proposal_count, last_cursor),
    )?;
    Ok(NnsProposalRefreshReport {
        schema_version: NNS_PROPOSAL_REFRESH_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        proposal_count,
        page_size: request.page_size,
        page_count,
        complete: true,
        replaced_existing_cache,
        wrote_cache: true,
        fetched_at,
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        refresh_lock_path: paths.refresh_lock_path.display().to_string(),
    })
}
