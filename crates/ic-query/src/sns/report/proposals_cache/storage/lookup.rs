//! Module: sns::report::proposals_cache::storage::lookup
//!
//! Responsibility: resolve proposal cache snapshots by SNS id or root.
//! Does not own: cache summary construction, report assembly, or refresh policy.
//! Boundary: provides typed proposal cache lookup helpers for report builders.

use crate::sns::report::{
    SnsHostError,
    cache_paths::sns_snapshot_network_cache_dir,
    cache_storage::{load_sns_cache_by_id, load_sns_cache_for_root},
    parse_sns_root_canister_input,
    proposals_cache::{model::SnsProposalsCache, paths::SnsProposalsCacheCollection},
};
use std::path::{Path, PathBuf};

/// Load a complete SNS proposal cache and return its concrete cache path.
pub(in crate::sns::report::proposals_cache) fn load_sns_proposals_cache_for_input_with_path(
    cache_root: &Path,
    network: &str,
    input: &str,
) -> Result<(PathBuf, SnsProposalsCache), SnsHostError> {
    if let Ok(id) = input.parse::<usize>() {
        return load_sns_cache_by_id::<SnsProposalsCacheCollection>(cache_root, network, id)?
            .ok_or_else(|| SnsHostError::MissingProposalsCache {
                path: sns_snapshot_network_cache_dir(cache_root, network),
            });
    }

    let root_canister_id = parse_sns_root_canister_input(input)?;
    load_sns_cache_for_root::<SnsProposalsCacheCollection>(cache_root, network, &root_canister_id)
}
