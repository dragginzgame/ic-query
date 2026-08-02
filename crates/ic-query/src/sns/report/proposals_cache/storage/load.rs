//! Module: sns::report::proposals_cache::storage::load
//!
//! Responsibility: load complete SNS proposal cache snapshots.
//! Does not own: cache path scanning, status summaries, or refresh orchestration.
//! Boundary: maps snapshot JSON loading errors into SNS host errors.

use crate::snapshot_cache::validate_snapshot_completeness;
use crate::sns::report::{
    SnsHostError,
    cache_paths::sns_snapshot_key_for_cache_path,
    cache_storage::{load_sns_complete_cache, validate_sns_cache_metadata},
    proposals_cache::{model::SnsProposalsCache, paths::SnsProposalsCacheCollection},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// Load one complete SNS proposal snapshot from a concrete cache path.
pub(in crate::sns::report::proposals_cache) fn load_sns_proposals_cache_at(
    cache_path: PathBuf,
    network: &str,
) -> Result<SnsProposalsCache, SnsHostError> {
    let key = sns_snapshot_key_for_cache_path::<SnsProposalsCacheCollection>(network, &cache_path);
    let cache = load_sns_complete_cache::<SnsProposalsCache, SnsProposalsCacheCollection>(
        cache_path.clone(),
        network,
        &key,
    )?;
    validate_sns_proposals_cache(&cache_path, &cache)?;
    Ok(cache)
}

fn validate_sns_proposals_cache(
    path: &Path,
    cache: &SnsProposalsCache,
) -> Result<(), SnsHostError> {
    let invalid = |reason| SnsHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    };
    validate_snapshot_completeness(&cache.completeness, cache.data.proposals.len())
        .map_err(invalid)?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid(
            "SNS Governance proposal pagination cannot claim a point-in-time guarantee".to_string(),
        ));
    }
    validate_sns_cache_metadata(path, &cache.metadata, &cache.entity)?;
    let mut proposal_ids = HashSet::new();
    if let Some(duplicate) = cache
        .data
        .proposals
        .iter()
        .map(|proposal| proposal.proposal_id)
        .find(|proposal_id| !proposal_ids.insert(*proposal_id))
    {
        return Err(invalid(format!("duplicate proposal id {duplicate}")));
    }
    Ok(())
}
