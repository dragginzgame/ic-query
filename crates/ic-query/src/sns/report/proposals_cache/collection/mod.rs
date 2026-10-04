//! Module: sns::report::proposals_cache::collection
//!
//! Responsibility: fetch complete SNS proposal collections page by page.
//! Does not own: cache publication, command parsing, or report rendering.
//! Boundary: drives proposal paging and refresh-attempt progress updates.

use crate::{
    QueryProgress,
    snapshot_cache::{
        PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh,
        run_paged_snapshot_refresh_with_progress,
    },
    sns::report::{
        SnsHostError, SnsProposalRow,
        cache_attempt::{SnsRefreshContext, write_running_sns_refresh_page},
        proposals_cache::model::CompleteSnsProposals,
        source::{SnsProposalsSource, validate_mainnet_sns_proposal_page},
    },
};

/// Fetch every proposal page required for a complete SNS proposal snapshot.
pub(in crate::sns::report::proposals_cache) fn fetch_complete_sns_proposals(
    context: SnsRefreshContext<'_>,
    source: &dyn SnsProposalsSource,
    progress: &mut dyn QueryProgress,
) -> Result<CompleteSnsProposals, SnsHostError> {
    run_paged_snapshot_refresh_with_progress(
        SnsProposalsRefreshPages {
            context,
            source,
            pages: PagedCollectionState::new(),
        },
        progress,
    )
}

///
/// SnsProposalsRefreshPages
///
/// Paged refresh runner state for complete SNS proposal collection.
///

struct SnsProposalsRefreshPages<'a> {
    context: SnsRefreshContext<'a>,
    source: &'a dyn SnsProposalsSource,
    pages: PagedCollectionState<SnsProposalRow, u64>,
}

impl PagedSnapshotRefresh for SnsProposalsRefreshPages<'_> {
    type Complete = CompleteSnsProposals;
    type Error = SnsHostError;

    fn progress_text(&self) -> String {
        self.context
            .progress_text("proposals", self.pages.page_count(), self.pages.row_count())
    }

    fn max_pages_reached(&self) -> bool {
        self.context.max_pages_reached(self.pages.page_count())
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        SnsRefreshContext::incomplete_refresh_error(
            self.pages.page_count(),
            self.pages.row_count(),
            reason,
        )
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let page = self.source.fetch_sns_proposal_page(
            self.context.fetch_request,
            self.context.sns,
            self.context.request.page_size(),
            self.pages.next_cursor().copied(),
        )?;
        validate_mainnet_sns_proposal_page(
            &page,
            self.context.request.page_size(),
            self.pages.next_cursor().copied(),
        )?;
        let last_cursor = page
            .proposals
            .iter()
            .map(|proposal| proposal.proposal_id)
            .min();
        let page_len = page.proposals.len();
        Ok(self
            .pages
            .ingest_page(page.proposals, page_len, last_cursor, ToString::to_string))
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_sns_refresh_page(
            self.context,
            self.pages.page_count(),
            self.pages.row_count(),
            page,
        )
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        self.context
            .page_exhausts_collection(page, self.pages.has_next_cursor())
    }

    fn into_complete(self) -> Self::Complete {
        self.pages.into_complete(ToString::to_string)
    }
}
