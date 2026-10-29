//! Module: sns::report::proposals_cache::collection
//!
//! Responsibility: fetch complete SNS proposal collections page by page.
//! Does not own: cache publication, command parsing, or report rendering.
//! Boundary: drives proposal paging and refresh-attempt progress updates.

use crate::{
    QueryProgress,
    snapshot_cache::{PagedCollectionPage, PagedCollectionState},
    sns::report::{
        SnsHostError, SnsProposalRow,
        cache_attempt::SnsRefreshContext,
        cache_collection::{SnsCollectionPages, collect_sns_snapshot},
        proposals_cache::model::CompleteSnsProposals,
        source::{SnsProposalsSource, validate_mainnet_sns_proposal_page},
    },
};

/// Fetch every proposal page required for a complete SNS proposal snapshot.
pub(in crate::sns::report::proposals_cache) async fn fetch_complete_sns_proposals(
    context: SnsRefreshContext<'_>,
    source: &dyn SnsProposalsSource,
    progress: &mut dyn QueryProgress,
) -> Result<CompleteSnsProposals, SnsHostError> {
    collect_sns_snapshot(context, SnsProposalsRefreshPages { source }, progress).await
}

///
/// SnsProposalsRefreshPages
///
/// Paged refresh runner state for complete SNS proposal collection.
///

struct SnsProposalsRefreshPages<'a> {
    source: &'a dyn SnsProposalsSource,
}

impl SnsCollectionPages for SnsProposalsRefreshPages<'_> {
    type Row = SnsProposalRow;
    type Cursor = u64;
    const COLLECTION: &'static str = "proposals";

    fn fetch_next_page<'a>(
        &'a mut self,
        context: SnsRefreshContext<'a>,
        pages: &'a mut PagedCollectionState<Self::Row, Self::Cursor>,
    ) -> crate::report::SourceFuture<'a, Result<PagedCollectionPage, SnsHostError>> {
        Box::pin(async move {
            let page = self
                .source
                .fetch_sns_proposal_page(
                    context.fetch_request,
                    context.sns,
                    context.request.page_size,
                    pages.next_cursor().copied(),
                )
                .await?;
            validate_mainnet_sns_proposal_page(
                &page,
                context.request.page_size,
                pages.next_cursor().copied(),
            )?;
            let last_cursor = page.iter().map(|proposal| proposal.proposal_id).min();
            let page_len = page.len();
            Ok(context.ingest_page(pages, page, page_len, last_cursor))
        })
    }

    fn cursor_text(cursor: &Self::Cursor) -> String {
        cursor.to_string()
    }
}
