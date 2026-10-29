//! Module: snapshot_cache::refresh
//!
//! Responsibility: run paged snapshot refresh adapters.
//! Does not own: source APIs, cache publication, refresh-attempt schemas, or process output.
//! Boundary: emits structured progress while driving page fetches and attempt writes.

use super::PagedCollectionPage;
use crate::{QueryProgress, QueryProgressEvent, QueryProgressState};

///
/// PagedSnapshotRefresh
///
/// Command-specific adapter for running a complete paged snapshot refresh.
///

pub trait PagedSnapshotRefresh {
    type Complete;
    type Error;

    /// Family diagnostic retained when a page ceiling prevents API exhaustion.
    const PAGE_LIMIT_REASON: &'static str = "max pages reached before API exhaustion";

    fn progress_text(&self) -> String;
    /// Format family-owned progress without changing the shared lifecycle state.
    fn progress_text_for_state(&self, state: QueryProgressState) -> String {
        let text = self.progress_text();
        match state {
            QueryProgressState::Running => text,
            QueryProgressState::Stopped => format!("{text} stopped before completion"),
            QueryProgressState::Failed => format!("{text} failed"),
            QueryProgressState::Stalled => format!("{text} stalled"),
            QueryProgressState::Complete => format!("{text} complete"),
        }
    }

    fn max_pages_reached(&self) -> bool;
    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error;
    fn fetch_next_page(
        &mut self,
    ) -> crate::report::SourceFuture<'_, Result<PagedCollectionPage, Self::Error>>;
    /// Persist admitted progress from adapter state when this collection has a page sidecar.
    fn write_running_attempt(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn into_complete(self) -> Self::Complete;
}

/// Run a complete paged snapshot refresh and emit structured progress events.
pub async fn run_paged_snapshot_refresh_with_progress<Refresh>(
    mut refresh: Refresh,
    progress: &mut dyn QueryProgress,
) -> Result<Refresh::Complete, Refresh::Error>
where
    Refresh: PagedSnapshotRefresh,
{
    report_progress(
        progress,
        refresh.progress_text_for_state(QueryProgressState::Running),
        QueryProgressState::Running,
    );

    loop {
        if refresh.max_pages_reached() {
            report_progress(
                progress,
                refresh.progress_text_for_state(QueryProgressState::Stopped),
                QueryProgressState::Stopped,
            );
            return Err(refresh.incomplete_refresh_error(Refresh::PAGE_LIMIT_REASON));
        }

        let page = match refresh.fetch_next_page().await.and_then(|page| {
            refresh.write_running_attempt()?;
            Ok(page)
        }) {
            Ok(page) => page,
            Err(err) => {
                report_progress(
                    progress,
                    refresh.progress_text_for_state(QueryProgressState::Failed),
                    QueryProgressState::Failed,
                );
                return Err(err);
            }
        };
        report_progress(
            progress,
            refresh.progress_text_for_state(QueryProgressState::Running),
            QueryProgressState::Running,
        );

        if page.collection_exhausted {
            break;
        }
        if page.has_no_new_rows() {
            report_progress(
                progress,
                refresh.progress_text_for_state(QueryProgressState::Stalled),
                QueryProgressState::Stalled,
            );
            return Err(refresh.incomplete_refresh_error(
                "page returned no new rows while advertising another cursor",
            ));
        }
    }

    report_progress(
        progress,
        refresh.progress_text_for_state(QueryProgressState::Complete),
        QueryProgressState::Complete,
    );
    Ok(refresh.into_complete())
}

fn report_progress(progress: &mut dyn QueryProgress, text: String, state: QueryProgressState) {
    progress.report(QueryProgressEvent::PagedRefresh { text, state });
}
