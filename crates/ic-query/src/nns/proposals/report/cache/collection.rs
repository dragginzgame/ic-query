//! Module: nns::proposals::report::cache::collection
//!
//! Responsibility: fetch complete NNS proposal collections page by page.
//! Does not own: refresh locking, cache publication, or command parsing.
//! Boundary: drives proposal paging and refresh-attempt progress updates.

use super::{NNS_PROPOSAL_CACHE_COMPONENT, model::CompleteNnsProposalCollection};
use crate::subnet_catalog::MAINNET_NETWORK;
use crate::{
    QueryProgress,
    nns::{
        NnsGovernanceRefreshRequest,
        governance::{NnsGovernanceRequest, write_running_governance_refresh_attempt},
        proposals::report::{
            NNS_PROPOSAL_FETCHED_BY, NnsProposalHostError,
            collection::{NnsProposalCollectionState, advance_nns_proposal_collection_with_source},
            model::NnsProposalRow,
            source::NnsProposalSource,
        },
    },
    runtime::block_on_current_thread,
    snapshot_cache::{
        PagedCollectionPage, PagedSnapshotRefresh, SnapshotRefreshProgress,
        run_paged_snapshot_refresh_with_progress,
    },
};
use std::{cmp::Reverse, path::Path};

/// Fetch every proposal page required for a complete NNS proposal snapshot.
pub(super) fn fetch_complete_nns_proposal_collection(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsProposalSource,
    attempt_path: &Path,
    progress: &mut dyn QueryProgress,
) -> Result<CompleteNnsProposalCollection, NnsProposalHostError> {
    let fetch_request = NnsGovernanceRequest::replica_query_from_unix_secs(
        MAINNET_NETWORK,
        &request.source_endpoint,
        request.now_unix_secs,
        NNS_PROPOSAL_FETCHED_BY,
    );
    let collection_state =
        NnsProposalCollectionState::new(&fetch_request, request.page_size, u32::MAX)?;
    run_paged_snapshot_refresh_with_progress(
        NnsProposalRefreshPages {
            request,
            fetch_request,
            source,
            attempt_path,
            collection_state,
            proposals: Vec::new(),
        },
        progress,
    )
}

///
/// NnsProposalRefreshPages
///
/// Paged refresh runner state for the complete NNS proposal collection.
///

struct NnsProposalRefreshPages<'a> {
    request: &'a NnsGovernanceRefreshRequest,
    fetch_request: NnsGovernanceRequest,
    source: &'a dyn NnsProposalSource,
    attempt_path: &'a Path,
    collection_state: NnsProposalCollectionState,
    proposals: Vec<NnsProposalRow>,
}

impl PagedSnapshotRefresh for NnsProposalRefreshPages<'_> {
    type Complete = CompleteNnsProposalCollection;
    type Error = NnsProposalHostError;

    fn progress_text(&self) -> String {
        format!(
            "refreshing NNS proposals: pages={} rows={}",
            self.collection_state.pages_fetched(),
            self.collection_state.proposals_fetched()
        )
    }

    fn max_pages_reached(&self) -> bool {
        self.request
            .max_pages
            .is_some_and(|max_pages| self.collection_state.pages_fetched() >= max_pages)
    }

    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
        NnsProposalHostError::IncompleteRefresh {
            pages_fetched: self.collection_state.pages_fetched(),
            rows_fetched: self.proposals.len(),
            reason: reason.to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let step = block_on_current_thread(advance_nns_proposal_collection_with_source(
            &self.fetch_request,
            &self.collection_state,
            self.source,
        ))??;
        let page_len = step.page.proposals.len();
        let next_cursor = step.state.next_before_proposal_id();
        self.proposals.extend(step.page.proposals);
        self.collection_state = step.state;
        Ok(PagedCollectionPage::new(
            page_len,
            page_len,
            next_cursor.map(|cursor| cursor.to_string()),
        ))
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_governance_refresh_attempt(
            self.attempt_path,
            self.request,
            NNS_PROPOSAL_CACHE_COMPONENT,
            SnapshotRefreshProgress::new(
                self.collection_state.pages_fetched(),
                self.proposals.len(),
                page.last_cursor_text.clone(),
            ),
        )
        .map_err(NnsProposalHostError::from)
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(
            self.request.page_size,
            self.collection_state.next_before_proposal_id().is_some(),
        )
    }

    fn into_complete(self) -> CompleteNnsProposalCollection {
        let mut proposals = self.proposals;
        proposals.sort_by_key(|proposal| Reverse(proposal.proposal_id));
        CompleteNnsProposalCollection {
            proposals,
            page_count: self.collection_state.pages_fetched(),
            last_cursor: self
                .collection_state
                .next_before_proposal_id()
                .map(|cursor| cursor.to_string()),
        }
    }
}
