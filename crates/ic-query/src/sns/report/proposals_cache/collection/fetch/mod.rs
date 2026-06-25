//! Module: sns::report::proposals_cache::collection::fetch
//!
//! Responsibility: fetch complete SNS proposal collections page by page.
//! Does not own: cache publication, command parsing, or report rendering.
//! Boundary: drives proposal paging and refresh-attempt progress updates.

mod attempt;
mod state;

use crate::{
    snapshot_cache::{PagedCollectionPage, PagedSnapshotRefresh, run_paged_snapshot_refresh},
    sns::report::{
        SnsHostError, SnsProposalsRefreshRequest,
        proposals_cache::model::CompleteSnsProposals,
        source::{MainnetSns, SnsFetchRequest, SnsProposalsSource},
    },
};
use state::SnsProposalsCollectionState;
use std::path::Path;

/// Fetch every proposal page required for a complete SNS proposal snapshot.
pub(in crate::sns::report::proposals_cache) fn fetch_complete_sns_proposals(
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
        sns_proposals_progress_text(self.sns, self.state.page_count(), self.state.row_count())
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
        attempt::write_running_attempt(
            self.attempt_path,
            self.request,
            self.fetch_request,
            self.sns,
            &self.state,
            page,
        )
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(self.request.page_size, self.state.has_next_cursor())
    }

    fn into_complete(self) -> Self::Complete {
        self.state.into_complete()
    }
}

fn sns_proposals_progress_text(sns: &MainnetSns, pages: u32, rows: usize) -> String {
    format!(
        "refreshing SNS proposals for {}: pages={} rows={}",
        sns.name, pages, rows
    )
}
