//! Module: sns::report::proposals_cache::collection::fetch::state
//!
//! Responsibility: track paged SNS proposal collection state.
//! Does not own: live page fetching, attempt persistence, or cache publishing.
//! Boundary: accumulates de-duplicated proposal rows and pagination cursors.

use crate::snapshot_cache::{PagedCollectionPage, PagedCollectionState};
use crate::sns::report::{
    SnsHostError, SnsProposalRow,
    proposals_cache::model::CompleteSnsProposals,
    source::{MainnetSnsProposalPage, validate_mainnet_sns_proposal_page},
};

///
/// SnsProposalsCollectionState
///
/// Accumulated page state for a complete SNS proposal snapshot refresh.
///

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

    pub(super) fn ingest_page(
        &mut self,
        page: MainnetSnsProposalPage,
        requested_limit: u32,
    ) -> Result<PagedCollectionPage, SnsHostError> {
        validate_mainnet_sns_proposal_page(&page, requested_limit)?;
        let last_cursor = page.proposals.last().map(|proposal| proposal.proposal_id);
        Ok(self.pages.ingest_page(
            page.proposals,
            last_cursor,
            ToString::to_string,
            |proposal| proposal.proposal_id.to_string(),
        ))
    }

    pub(super) fn into_complete(self) -> CompleteSnsProposals {
        self.pages.into_complete(ToString::to_string)
    }
}
