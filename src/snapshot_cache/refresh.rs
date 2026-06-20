use super::PagedCollectionPage;
use crate::progress::ProgressLine;

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
    fn incomplete_refresh_error(&self) -> Self::Error;
    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error>;
    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error>;
    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool;
    fn into_complete(self) -> Self::Complete;
}

pub fn run_paged_snapshot_refresh<Refresh>(
    mut refresh: Refresh,
) -> Result<Refresh::Complete, Refresh::Error>
where
    Refresh: PagedSnapshotRefresh,
{
    let mut progress = ProgressLine::stderr();
    progress.update(&refresh.progress_text());

    loop {
        if refresh.max_pages_reached() {
            progress.finish(&format!(
                "{} stopped before completion",
                refresh.progress_text()
            ));
            return Err(refresh.incomplete_refresh_error());
        }

        let page = match refresh.fetch_next_page() {
            Ok(page) => page,
            Err(err) => {
                progress.finish(&format!("{} failed", refresh.progress_text()));
                return Err(err);
            }
        };
        if let Err(err) = refresh.write_running_attempt(&page) {
            progress.finish(&format!("{} failed", refresh.progress_text()));
            return Err(err);
        }
        progress.update(&refresh.progress_text());

        if refresh.page_exhausts_collection(&page) {
            break;
        }
    }

    progress.finish(&format!("{} complete", refresh.progress_text()));
    Ok(refresh.into_complete())
}
