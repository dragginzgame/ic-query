//! Module: sns::report::proposals_cache::storage::lookup
//!
//! Responsibility: resolve proposal cache snapshots by SNS id or root.
//! Does not own: cache summary construction, report assembly, or refresh policy.
//! Boundary: provides typed proposal cache lookup helpers for report builders.

use super::{load::load_sns_proposals_cache_at, scan::read_sns_proposals_cache_header};
use crate::sns::report::{
    SnsHostError,
    proposals_cache::paths::{SnsProposalsCachePaths, sns_network_cache_dir},
};
use candid::Principal;
use std::path::{Path, PathBuf};

use super::super::model::SnsProposalsCache;
use super::scan::collect_sns_proposals_cache_paths;

/// Load a complete SNS proposal cache by stable SNS list id or root principal.
pub(in crate::sns::report::proposals_cache) fn load_sns_proposals_cache_for_input(
    icp_root: &Path,
    network: &str,
    input: &str,
) -> Result<SnsProposalsCache, SnsHostError> {
    if let Ok(id) = input.parse::<usize>() {
        return find_sns_proposals_cache_by_id(icp_root, network, id)?.map_or_else(
            || {
                Err(SnsHostError::MissingProposalsCache {
                    path: sns_network_cache_dir(icp_root, network),
                })
            },
            |(_path, cache)| Ok(cache),
        );
    }

    let root_canister_id = Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })?
        .to_text();
    let cache_path = expected_cache_path_for_root(icp_root, network, &root_canister_id);
    load_sns_proposals_cache_at(cache_path, network)
}

/// Find a complete SNS proposal cache by stable SNS list id.
pub(in crate::sns::report::proposals_cache) fn find_sns_proposals_cache_by_id(
    icp_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<(PathBuf, SnsProposalsCache)>, SnsHostError> {
    for path in collect_sns_proposals_cache_paths(icp_root, network)? {
        let header = read_sns_proposals_cache_header(&path, network)?;
        if header.metadata.id == id {
            let cache = load_sns_proposals_cache_at(path.clone(), network)?;
            return Ok(Some((path, cache)));
        }
    }
    Ok(None)
}

/// Build the expected complete proposal cache path for one SNS root.
pub(in crate::sns::report::proposals_cache) fn expected_cache_path_for_root(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    SnsProposalsCachePaths::for_root(icp_root, network, root_canister_id).cache_path
}
