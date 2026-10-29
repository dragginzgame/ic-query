//! Module: nns::proposals::report::cache::collection
//!
//! Responsibility: fetch complete NNS proposal collections page by page.
//! Does not own: refresh locking, cache publication, or command parsing.
//! Boundary: drives proposal paging and refresh-attempt progress updates.

use super::{NNS_PROPOSAL_CACHE_COMPONENT, model::NnsProposalCacheRows};
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
    snapshot_cache::{
        PagedCollectionPage, PagedSnapshotRefresh, SnapshotRefreshProgress,
        run_paged_snapshot_refresh_with_progress,
    },
};
use std::{cmp::Reverse, path::Path};

/// Fetch every proposal page required for a complete NNS proposal snapshot.
pub(super) async fn fetch_complete_nns_proposal_collection(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsProposalSource,
    attempt_path: &Path,
    progress: &mut dyn QueryProgress,
) -> Result<NnsProposalCacheRows, NnsProposalHostError> {
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
    .await
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
    type Complete = NnsProposalCacheRows;
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

    fn fetch_next_page(
        &mut self,
    ) -> crate::report::SourceFuture<'_, Result<PagedCollectionPage, Self::Error>> {
        Box::pin(async move {
            let step = advance_nns_proposal_collection_with_source(
                &self.fetch_request,
                &self.collection_state,
                self.source,
            )
            .await?;
            let page_len = step.page.proposals.len();
            self.proposals.extend(step.page.proposals);
            self.collection_state = step.state;
            Ok(PagedCollectionPage::new(
                page_len,
                self.collection_state.is_complete(),
            ))
        })
    }

    fn write_running_attempt(&self) -> Result<(), Self::Error> {
        write_running_governance_refresh_attempt(
            self.attempt_path,
            self.request,
            NNS_PROPOSAL_CACHE_COMPONENT,
            SnapshotRefreshProgress::new(
                self.collection_state.pages_fetched(),
                self.proposals.len(),
                self.collection_state
                    .next_before_proposal_id()
                    .map(|cursor| cursor.to_string()),
            ),
        )
        .map_err(NnsProposalHostError::from)
    }

    fn into_complete(self) -> NnsProposalCacheRows {
        let mut proposals = self.proposals;
        proposals.sort_by_key(|proposal| Reverse(proposal.proposal_id));
        NnsProposalCacheRows {
            proposals,
            collection_state: self.collection_state,
        }
    }
}
