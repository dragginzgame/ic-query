//! Module: sns::report::proposals_cache::storage::scan
//!
//! Responsibility: scan and header-read local SNS proposal cache snapshots.
//! Does not own: complete cache loading, report summaries, or cache status rendering.
//! Boundary: centralizes deterministic proposal snapshot path discovery.

use crate::sns::report::{
    SnsHostError,
    cache_storage::{collect_sns_cache_paths, read_sns_cache_header},
    proposals_cache::{
        SNS_PROPOSALS_CACHE_SCHEMA_VERSION, errors::SnsProposalsCacheErrors,
        model::SnsProposalsCacheHeader, paths::SnsProposalsCacheCollection,
    },
};
use std::path::{Path, PathBuf};

pub(super) fn collect_sns_proposals_cache_paths(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError> {
    collect_sns_cache_paths::<SnsProposalsCacheCollection>(icp_root, network)
}

pub(super) fn read_sns_proposals_cache_header(
    path: &Path,
    network: &str,
) -> Result<SnsProposalsCacheHeader, SnsHostError> {
    read_sns_cache_header(
        path,
        network,
        SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        SnsProposalsCacheErrors,
    )
}
