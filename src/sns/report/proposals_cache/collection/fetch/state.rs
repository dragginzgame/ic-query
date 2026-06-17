//! Module: sns::report::proposals_cache::collection::fetch::state
//!
//! Responsibility: track paged SNS proposal collection state.
//! Does not own: live page fetching, attempt persistence, or cache publishing.
//! Boundary: accumulates de-duplicated proposal rows and pagination cursors.

use crate::snapshot_cache::{PagedCollectionPage, PagedCollectionState};
use crate::sns::report::{
    SnsProposalRow, proposals_cache::model::CompleteSnsProposals, source::MainnetSnsProposalPage,
};

pub(super) struct SnsProposalsCollectionState {
    pages: PagedCollectionState<SnsProposalRow, u64>,
}

impl SnsProposalsCollectionState {
    pub(super) fn new() -> Self {
        Self {
            pages: PagedCollectionState::new(),
        }
    }

    pub(super) const fn page_count(&self) -> u32 {
        self.pages.page_count()
    }

    pub(super) const fn row_count(&self) -> usize {
        self.pages.row_count()
    }

    pub(super) const fn before_proposal_id(&self) -> Option<u64> {
        match self.pages.next_cursor() {
            Some(cursor) => Some(*cursor),
            None => None,
        }
    }

    pub(super) const fn has_next_cursor(&self) -> bool {
        self.pages.has_next_cursor()
    }

    pub(super) fn ingest_page(&mut self, page: MainnetSnsProposalPage) -> PagedCollectionPage {
        self.pages.ingest_page(
            page.proposals,
            page.last_cursor,
            ToString::to_string,
            proposal_row_id,
        )
    }

    pub(super) fn into_complete(self) -> CompleteSnsProposals {
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
