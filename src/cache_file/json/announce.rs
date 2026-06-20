//! Module: cache_file::json::announce
//!
//! Responsibility: render missing-cache refresh announcements.
//! Does not own: cache refresh policy, loaders, or command text reports.
//! Boundary: writes explicit live-call notices before automatic cache creation.

use std::path::Path;

pub fn announce_cache_refresh(component: &str, path: &Path, source_endpoint: &str) {
    eprintln!(
        "{component} cache missing at {}; calling {source_endpoint} to refresh/create cache",
        path.display()
    );
}
