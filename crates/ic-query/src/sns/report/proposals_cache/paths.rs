//! Module: sns::report::proposals_cache::paths
//!
//! Responsibility: own SNS proposal snapshot identity, storage admission, and cache paths.
//! Does not own: cache loading, refresh locking, or status rendering.
//! Boundary: maps SNS root principals to proposal snapshot file paths.

use crate::sns::report::{
    SnsHostError,
    cache_paths::sns_snapshot_cache_paths,
    cache_storage::SnsCacheStorageFamily,
    proposals_cache::{SNS_PROPOSALS_CACHE_SCHEMA_VERSION, model::SnsProposalsCacheRows},
    source::validate_sns_proposal_rows,
};
use std::path::{Path, PathBuf};

///
/// SnsProposalsCacheCollection
///
/// Collection marker used to derive complete SNS proposal snapshot paths.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) enum SnsProposalsCacheCollection {}

#[must_use]
pub fn sns_proposals_cache_path(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths::<SnsProposalsCacheCollection>(cache_root, network, root_canister_id)
        .snapshot_path
}

#[must_use]
pub fn sns_proposals_refresh_lock_path(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths::<SnsProposalsCacheCollection>(cache_root, network, root_canister_id)
        .refresh_lock_path
}

#[must_use]
pub fn sns_proposals_refresh_attempt_path(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    sns_snapshot_cache_paths::<SnsProposalsCacheCollection>(cache_root, network, root_canister_id)
        .refresh_attempt_path
}

impl SnsCacheStorageFamily for SnsProposalsCacheCollection {
    const COLLECTION: &'static str = "proposals";

    type Data = SnsProposalsCacheRows;

    const CACHE_SCHEMA_VERSION: u32 = SNS_PROPOSALS_CACHE_SCHEMA_VERSION;

    const CACHE_ITEM_NAME: &'static str = "proposal";

    fn missing_cache_error(path: PathBuf) -> SnsHostError {
        SnsHostError::MissingProposalsCache { path }
    }

    fn row_count(data: &Self::Data) -> usize {
        data.proposals.len()
    }

    fn validate_rows(data: &Self::Data) -> Result<(), String> {
        validate_sns_proposal_rows(&data.proposals)
    }
}
