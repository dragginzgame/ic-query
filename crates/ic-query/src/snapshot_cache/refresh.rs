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

    fn progress_text(&self) -> String;
    fn max_pages_reached(&self) -> bool;
    fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error;
    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error>;
    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error>;
    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool;
    fn into_complete(self) -> Self::Complete;
}

/// Run a complete paged snapshot refresh and emit structured progress events.
pub fn run_paged_snapshot_refresh_with_progress<Refresh>(
    mut refresh: Refresh,
    progress: &mut dyn QueryProgress,
) -> Result<Refresh::Complete, Refresh::Error>
where
    Refresh: PagedSnapshotRefresh,
{
    report_progress(
        progress,
        refresh.progress_text(),
        QueryProgressState::Running,
    );

    loop {
        if refresh.max_pages_reached() {
            report_progress(
                progress,
                format!("{} stopped before completion", refresh.progress_text()),
                QueryProgressState::Stopped,
            );
            return Err(refresh.incomplete_refresh_error("max pages reached before API exhaustion"));
        }

        let page = match refresh.fetch_next_page() {
            Ok(page) => page,
            Err(err) => {
                report_progress(
                    progress,
                    format!("{} failed", refresh.progress_text()),
                    QueryProgressState::Failed,
                );
                return Err(err);
            }
        };
        if let Err(err) = refresh.write_running_attempt(&page) {
            report_progress(
                progress,
                format!("{} failed", refresh.progress_text()),
                QueryProgressState::Failed,
            );
            return Err(err);
        }
        report_progress(
            progress,
            refresh.progress_text(),
            QueryProgressState::Running,
        );

        if refresh.page_exhausts_collection(&page) {
            break;
        }
        if page.has_no_new_rows() {
            report_progress(
                progress,
                format!("{} stalled", refresh.progress_text()),
                QueryProgressState::Stalled,
            );
            return Err(refresh.incomplete_refresh_error(
                "page returned no new rows while advertising another cursor",
            ));
        }
    }

    report_progress(
        progress,
        format!("{} complete", refresh.progress_text()),
        QueryProgressState::Complete,
    );
    Ok(refresh.into_complete())
}

fn report_progress(progress: &mut dyn QueryProgress, text: String, state: QueryProgressState) {
    progress.report(QueryProgressEvent::PagedRefresh { text, state });
}
