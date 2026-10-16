//! Module: sns::report::model::requests::refresh
//!
//! Responsibility: define common complete SNS snapshot refresh settings.
//! Does not own: collection choice, cursor rules, source calls, or publication.
//! Boundary: neuron and proposal entry points select the collection explicitly.

use super::lookup::SnsLookupRequest;
use std::path::PathBuf;

///
/// SnsSnapshotRefreshRequest
///
/// Request accepted by the complete SNS snapshot refresh builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsSnapshotRefreshRequest {
    /// SNS target, explicit endpoint, network and caller clock.
    pub lookup: SnsLookupRequest,
    /// Caller-selected root for confined managed cache files.
    pub cache_root: PathBuf,
    /// Maximum rows requested per Governance page.
    pub page_size: u32,
    /// Optional page ceiling; reaching it before exhaustion prevents complete publication.
    pub max_pages: Option<u32>,
}

impl SnsSnapshotRefreshRequest {
    /// Construct snapshot refresh settings using the caller clock.
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, lookup: SnsLookupRequest, page_size: u32) -> Self {
        Self {
            lookup,
            cache_root: cache_root.into(),
            page_size,
            max_pages: None,
        }
    }

    /// Bound refresh pagination; truncated collections are not published as complete.
    #[must_use]
    pub const fn with_max_pages(mut self, max_pages: Option<u32>) -> Self {
        self.max_pages = max_pages;
        self
    }
}
