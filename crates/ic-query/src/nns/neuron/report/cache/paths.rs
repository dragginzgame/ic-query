//! Module: nns::neuron::report::cache::paths
//!
//! Responsibility: construct NNS neuron snapshot cache paths.
//! Does not own: refresh locking, JSON IO, or cache report rendering.
//! Boundary: maps the fixed NNS Governance neuron collection onto snapshot paths.

use crate::snapshot_cache::{SnapshotJsonPaths, SnapshotKey};
use std::path::{Path, PathBuf};

pub(super) const NNS_NEURON_CACHE_DOMAIN: &str = "nns";
pub(super) const NNS_NEURON_CACHE_ENTITY: &str = "governance";
pub(super) const NNS_NEURON_CACHE_COLLECTION: &str = "neurons";

pub(super) fn nns_neuron_cache_paths(cache_root: &Path, network: &str) -> SnapshotJsonPaths {
    SnapshotJsonPaths::for_key(
        cache_root,
        &SnapshotKey::full(
            NNS_NEURON_CACHE_DOMAIN,
            network,
            NNS_NEURON_CACHE_ENTITY,
            NNS_NEURON_CACHE_COLLECTION,
        ),
    )
}

/// Return the complete NNS neuron snapshot path.
#[must_use]
pub fn nns_neuron_cache_path(cache_root: &Path, network: &str) -> PathBuf {
    nns_neuron_cache_paths(cache_root, network).snapshot_path
}

/// Return the NNS neuron refresh-lock path.
#[must_use]
pub fn nns_neuron_refresh_lock_path(cache_root: &Path, network: &str) -> PathBuf {
    nns_neuron_cache_paths(cache_root, network).refresh_lock_path
}

/// Return the NNS neuron refresh-attempt path.
#[must_use]
pub fn nns_neuron_refresh_attempt_path(cache_root: &Path, network: &str) -> PathBuf {
    nns_neuron_cache_paths(cache_root, network).refresh_attempt_path
}
