//! Module: nns::proposals::report::cache::paths
//!
//! Responsibility: construct NNS proposal snapshot cache paths.
//! Does not own: refresh locking, JSON IO, or cache report rendering.
//! Boundary: maps the fixed NNS governance entity onto shared snapshot paths.

use crate::snapshot_cache::{SnapshotJsonPaths, SnapshotKey, snapshot_network_dir};
use std::path::{Path, PathBuf};

const NNS_PROPOSAL_CACHE_DOMAIN: &str = "nns";
const NNS_PROPOSAL_CACHE_ENTITY: &str = "governance";
const NNS_PROPOSAL_CACHE_COLLECTION: &str = "proposals";

pub(super) fn nns_proposal_cache_paths(cache_root: &Path, network: &str) -> SnapshotJsonPaths {
    SnapshotJsonPaths::for_key(cache_root, &nns_proposal_cache_key(network))
}

#[must_use]
pub fn nns_proposal_cache_path(cache_root: &Path, network: &str) -> PathBuf {
    nns_proposal_cache_paths(cache_root, network).snapshot_path
}

#[must_use]
pub fn nns_proposal_refresh_lock_path(cache_root: &Path, network: &str) -> PathBuf {
    nns_proposal_cache_paths(cache_root, network).refresh_lock_path
}

#[must_use]
pub fn nns_proposal_refresh_attempt_path(cache_root: &Path, network: &str) -> PathBuf {
    nns_proposal_cache_paths(cache_root, network).refresh_attempt_path
}

#[must_use]
pub fn nns_proposal_cache_root(cache_root: &Path, network: &str) -> PathBuf {
    snapshot_network_dir(cache_root, NNS_PROPOSAL_CACHE_DOMAIN, network)
        .join(NNS_PROPOSAL_CACHE_ENTITY)
        .join(NNS_PROPOSAL_CACHE_COLLECTION)
}

fn nns_proposal_cache_key(network: &str) -> SnapshotKey {
    SnapshotKey::full(
        NNS_PROPOSAL_CACHE_DOMAIN,
        network,
        NNS_PROPOSAL_CACHE_ENTITY,
        NNS_PROPOSAL_CACHE_COLLECTION,
    )
}
