//! Module: nns::proposals::report::cache::collection
//!
//! Responsibility: fetch complete NNS proposal collections page by page.
//! Does not own: refresh locking, cache publication, or command parsing.
//! Boundary: drives proposal paging and refresh-attempt progress updates.

use super::{
    attempt::{NnsProposalAttemptProgress, write_running_attempt},
    model::{CompleteNnsProposalCollection, NnsProposalRefreshRequest},
};
use crate::{
    nns::proposals::report::{
        NnsProposalHostError,
        model::NnsProposalRow,
        source::{NnsProposalFetchRequest, NnsProposalSource, nns_proposal_row_from_info},
    },
    snapshot_cache::{
        PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh, run_paged_snapshot_refresh,
    },
    subnet_catalog::format_utc_timestamp_secs,
};
use std::{cmp::Reverse, path::Path};

/// Fetch every proposal page required for a complete NNS proposal snapshot.
pub(super) fn fetch_complete_nns_proposal_collection(
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
