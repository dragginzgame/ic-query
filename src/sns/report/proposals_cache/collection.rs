//! Module: sns::report::proposals_cache::collection
//!
//! Responsibility: fetch complete paged SNS proposal collections.
//! Does not own: cache publishing, command parsing, or report rendering.
//! Boundary: drives proposal paging and writes running refresh-attempt progress.

use super::{
    attempt::{SnsProposalsAttemptContext, SnsProposalsAttemptProgress, write_running_attempt},
    model::CompleteSnsProposals,
};
use crate::{
    snapshot_cache::{
        PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh, run_paged_snapshot_refresh,
    },
    sns::report::{
        SnsHostError, SnsProposalRow, SnsProposalsRefreshRequest,
        source::{MainnetSns, MainnetSnsProposalPage, SnsFetchRequest, SnsProposalsSource},
    },
};
use std::path::Path;

/// Fetch every proposal page required for a complete SNS proposal snapshot.
pub(super) fn fetch_complete_sns_proposals(
    request: &SnsProposalsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    source: &dyn SnsProposalsSource,
    attempt_path: &Path,
) -> Result<CompleteSnsProposals, SnsHostError> {
    run_paged_snapshot_refresh(SnsProposalsRefreshPages {
        request,
        fetch_request,
        sns,
        source,
        attempt_path,
        state: SnsProposalsCollectionState::new(),
    })
}

///
/// SnsProposalsRefreshPages
///
/// Paged refresh runner state for complete SNS proposal collection.
///

struct SnsProposalsRefreshPages<'a> {
    request: &'a SnsProposalsRefreshRequest,
    fetch_request: &'a SnsFetchRequest,
    sns: &'a MainnetSns,
    source: &'a dyn SnsProposalsSource,
    attempt_path: &'a Path,
    state: SnsProposalsCollectionState,
}

impl PagedSnapshotRefresh for SnsProposalsRefreshPages<'_> {
    type Complete = CompleteSnsProposals;
    type Error = SnsHostError;

    fn progress_text(&self) -> String {
        format!(
            "refreshing SNS proposals for {}: pages={} rows={}",
            self.sns.name,
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
        SnsHostError::IncompleteRefresh {
            pages_fetched: self.state.page_count(),
            rows_fetched: self.state.row_count(),
            reason: "max pages reached before API exhaustion".to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let page = self.source.fetch_sns_proposal_page(
            self.fetch_request,
            self.sns,
            self.request.page_size,
            self.state.before_proposal_id(),
        )?;
        Ok(self.state.ingest_page(page))
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_attempt(
            SnsProposalsAttemptContext {
                path: self.attempt_path,
                request: self.request,
                fetch_request: self.fetch_request,
                sns: self.sns,
            },
            SnsProposalsAttemptProgress::new(
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
/// SnsProposalsCollectionState
///
/// Accumulates proposal pages, cursors, and de-duplicated proposal rows.
///

struct SnsProposalsCollectionState {
    pages: PagedCollectionState<SnsProposalRow, u64>,
}

impl SnsProposalsCollectionState {
    fn new() -> Self {
        Self {
            pages: PagedCollectionState::new(),
        }
    }

    const fn page_count(&self) -> u32 {
        self.pages.page_count()
    }

    const fn row_count(&self) -> usize {
        self.pages.row_count()
    }

    const fn before_proposal_id(&self) -> Option<u64> {
        match self.pages.next_cursor() {
            Some(cursor) => Some(*cursor),
            None => None,
        }
    }

    const fn has_next_cursor(&self) -> bool {
        self.pages.has_next_cursor()
    }

    fn ingest_page(&mut self, page: MainnetSnsProposalPage) -> PagedCollectionPage {
        self.pages.ingest_page(
            page.proposals,
            page.last_cursor,
            ToString::to_string,
            proposal_row_id,
        )
    }

    fn into_complete(self) -> CompleteSnsProposals {
        let complete = self.pages.into_complete(ToString::to_string);
        CompleteSnsProposals {
            proposals: complete.rows,
            page_count: complete.page_count,
            last_cursor: complete.last_cursor,
        }
    }
}

fn proposal_row_id(proposal: &SnsProposalRow) -> String {
    proposal.proposal_id.map_or_else(
        || {
            format!(
                "missing:{}:{}",
                proposal.proposal_creation_timestamp_seconds, proposal.title
            )
        },
        |proposal_id| proposal_id.to_string(),
    )
}
