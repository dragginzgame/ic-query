//! Module: sns::report::proposals_cache::storage::lookup
//!
//! Responsibility: resolve proposal cache snapshots by SNS id or root.
//! Does not own: cache summary construction, report assembly, or refresh policy.
//! Boundary: provides typed proposal cache lookup helpers for report builders.

use super::{load::load_sns_proposals_cache_at, scan::read_sns_proposals_cache_header};
use crate::sns::report::{
    SnsHostError, cache_paths::sns_snapshot_network_cache_dir,
    cache_storage::find_unique_sns_cache_path_by_id, parse_sns_root_canister_input,
    proposals_cache::model::SnsProposalsCache, proposals_cache::paths::SnsProposalsCachePaths,
};
use std::path::{Path, PathBuf};

use super::scan::collect_sns_proposals_cache_paths;

/// Load a complete SNS proposal cache and return its concrete cache path.
pub(in crate::sns::report::proposals_cache) fn load_sns_proposals_cache_for_input_with_path(
    cache_root: &Path,
    network: &str,
    input: &str,
) -> Result<(PathBuf, SnsProposalsCache), SnsHostError> {
    if let Ok(id) = input.parse::<usize>() {
        return find_sns_proposals_cache_by_id(cache_root, network, id)?.ok_or_else(|| {
            SnsHostError::MissingProposalsCache {
                path: sns_snapshot_network_cache_dir(cache_root, network),
            }
        });
    }

    let root_canister_id = parse_sns_root_canister_input(input)?;
    let cache_path =
        SnsProposalsCachePaths::for_root(cache_root, network, &root_canister_id).cache_path;
    let cache = load_sns_proposals_cache_at(cache_path.clone(), network)?;
    Ok((cache_path, cache))
}

/// Find a complete SNS proposal cache by stable SNS list id.
pub(in crate::sns::report::proposals_cache) fn find_sns_proposals_cache_by_id(
    cache_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<(PathBuf, SnsProposalsCache)>, SnsHostError> {
    let path = find_unique_sns_cache_path_by_id(
        collect_sns_proposals_cache_paths(cache_root, network)?,
        id,
        |path| read_sns_proposals_cache_header(path, network).map(|header| header.metadata.id),
    )?;
    path.map(|path| load_sns_proposals_cache_at(path.clone(), network).map(|cache| (path, cache)))
        .transpose()
}
