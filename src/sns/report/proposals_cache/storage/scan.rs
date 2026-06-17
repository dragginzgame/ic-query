//! Module: sns::report::proposals_cache::storage::scan
//!
//! Responsibility: scan and header-read local SNS proposal cache snapshots.
//! Does not own: complete cache loading, report summaries, or cache status rendering.
//! Boundary: centralizes deterministic proposal snapshot path discovery.

use super::super::{
    SNS_PROPOSALS_CACHE_SCHEMA_VERSION, model::SnsProposalsCacheHeader,
    paths::sns_network_cache_dir,
};
use crate::{
    cache_file::LoadJsonCacheRequest,
    snapshot_cache::{collect_full_collection_snapshot_paths, load_snapshot_header},
    sns::report::{SnsHostError, proposals_cache::errors::SnsProposalsCacheErrors},
};
use std::path::{Path, PathBuf};

pub(super) fn collect_sns_proposals_cache_paths(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError> {
    let root = sns_network_cache_dir(icp_root, network);
    collect_full_collection_snapshot_paths(&root, "proposals")
        .map_err(|source| SnsHostError::ReadCache { path: root, source })
}

pub(super) fn read_sns_proposals_cache_header(
    path: &Path,
    network: &str,
) -> Result<SnsProposalsCacheHeader, SnsHostError> {
    load_snapshot_header(
        LoadJsonCacheRequest {
            path: path.to_path_buf(),
            network,
            expected_schema_version: SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        },
        SnsProposalsCacheErrors,
    )
}
