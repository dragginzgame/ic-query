use std::path::Path;

pub fn announce_cache_refresh(component: &str, path: &Path, source_endpoint: &str) {
    eprintln!(
        "{component} cache missing at {}; calling {source_endpoint} to refresh/create cache",
        path.display()
    );
}
