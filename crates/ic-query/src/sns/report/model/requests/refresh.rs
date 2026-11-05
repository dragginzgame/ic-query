//! Module: sns::report::model::requests::refresh
//!
//! Responsibility: define common complete SNS snapshot refresh settings.
//! Does not own: collection choice, cursor rules, source calls, or publication.
//! Boundary: neuron entry points select the collection explicitly.

use super::target::SnsNeuronTargetRequest;
use std::path::PathBuf;

///
/// SnsSnapshotRefreshRequest
///
/// Request accepted by the complete SNS snapshot refresh builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsSnapshotRefreshRequest {
    /// SNS target, explicit endpoint, network and caller clock.
    pub target: SnsNeuronTargetRequest,
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
    pub fn new(
        cache_root: impl Into<PathBuf>,
        target: SnsNeuronTargetRequest,
        page_size: u32,
    ) -> Self {
        Self {
            target,
            cache_root: cache_root.into(),
            page_size,
            max_pages: None,
        }
    }
}
