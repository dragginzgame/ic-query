//! SNS-specific bookkeeping for the shared paged refresh lifecycle.

use super::{
    SnsHostError,
    cache_attempt::{SnsRefreshContext, write_running_sns_refresh_page},
};
use crate::{
    QueryProgress,
    report::SourceFuture,
    snapshot_cache::{
        CompletePagedCollection, PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh,
        run_paged_snapshot_refresh_with_progress,
    },
};

///
/// SnsCollectionPages
///
/// Family-owned page fetching, admission, and cursor text for one SNS snapshot.
///

pub(super) trait SnsCollectionPages {
    type Row;
    type Cursor;
    const COLLECTION: &'static str;

    fn fetch_next_page<'a>(
        &'a mut self,
        context: SnsRefreshContext<'a>,
        pages: &'a mut PagedCollectionState<Self::Row, Self::Cursor>,
    ) -> SourceFuture<'a, Result<PagedCollectionPage, SnsHostError>>;

    fn cursor_text(cursor: &Self::Cursor) -> String;
}

pub(super) async fn collect_sns_snapshot<Pages: SnsCollectionPages>(
    context: SnsRefreshContext<'_>,
    fetch: Pages,
    progress: &mut dyn QueryProgress,
) -> Result<CompletePagedCollection<Pages::Row>, SnsHostError> {
    run_paged_snapshot_refresh_with_progress(
        SnsCollectionRefresh {
            context,
            fetch,
            pages: PagedCollectionState::new(),
        },
        progress,
    )
    .await
}

struct SnsCollectionRefresh<'a, Pages: SnsCollectionPages> {
    context: SnsRefreshContext<'a>,
    fetch: Pages,
    pages: PagedCollectionState<Pages::Row, Pages::Cursor>,
}

impl<Pages: SnsCollectionPages> PagedSnapshotRefresh for SnsCollectionRefresh<'_, Pages> {
    type Complete = CompletePagedCollection<Pages::Row>;
    type Error = SnsHostError;

    fn progress_text(&self) -> String {
        self.context.progress_text(
            Pages::COLLECTION,
            self.pages.page_count(),
            self.pages.row_count(),
        )
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

    fn fetch_next_page(&mut self) -> SourceFuture<'_, Result<PagedCollectionPage, Self::Error>> {
        self.fetch.fetch_next_page(self.context, &mut self.pages)
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_sns_refresh_page(
            self.context,
            self.pages.page_count(),
            self.pages.row_count(),
            page,
        )
    }

    fn into_complete(self) -> Self::Complete {
        self.pages.into_complete(Pages::cursor_text)
    }
}
