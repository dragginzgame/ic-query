//! Module: sns::report::assemble::provenance
//!
//! Responsibility: build common live/cache provenance fields for SNS reports.
//! Does not own: cache loading, report DTOs, or text rendering.
//! Boundary: keeps report-source metadata construction consistent.

use std::path::Path;

pub(in crate::sns::report) struct SnsReportProvenance {
    pub(in crate::sns::report) data_source: String,
    pub(in crate::sns::report) cache_path: Option<String>,
    pub(in crate::sns::report) cache_complete: Option<bool>,
}

impl SnsReportProvenance {
    pub(in crate::sns::report) fn live() -> Self {
        Self {
            data_source: "live".to_string(),
            cache_path: None,
            cache_complete: None,
        }
    }

    pub(in crate::sns::report) fn cache(cache_path: &Path, cache_complete: bool) -> Self {
        Self {
            data_source: "cache".to_string(),
            cache_path: Some(cache_path.display().to_string()),
            cache_complete: Some(cache_complete),
        }
    }
}
