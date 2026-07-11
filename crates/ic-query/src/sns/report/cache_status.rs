//! Module: sns::report::cache_status
//!
//! Responsibility: share SNS cache status lookup flow across snapshot families.
//! Does not own: cache storage, refresh attempts, report DTOs, or rendering.
//! Boundary: resolves id/root status inputs into family-owned summary values.

use crate::{
    snapshot_cache::collect_full_collection_attempt_paths,
    sns::report::{
        SnsHostError, SnsRefreshAttemptStatus, enforce_mainnet_network,
        parse_sns_root_canister_input,
    },
};
use std::path::{Path, PathBuf};

///
/// SnsCacheStatusPaths
///
/// Filesystem paths needed to report status for one SNS snapshot cache.
///

pub(in crate::sns::report) struct SnsCacheStatusPaths {
    pub(in crate::sns::report) cache_path: PathBuf,
    pub(in crate::sns::report) attempt_path: PathBuf,
}

///
/// SnsCacheStatusLookup
///
/// Shared lookup result used to assemble SNS cache-status reports.
///

pub(in crate::sns::report) struct SnsCacheStatusLookup<Summary, Attempt> {
    pub(in crate::sns::report) cache_root: String,
    pub(in crate::sns::report) cache: Option<Summary>,
    pub(in crate::sns::report) expected_cache_path: Option<String>,
    pub(in crate::sns::report) refresh_attempt_path: Option<String>,
    pub(in crate::sns::report) latest_attempt: Option<Attempt>,
}

///
/// SnsCacheStatusSummaryView
///
/// Summary fields required by the shared SNS cache-status report flow.
///

pub(in crate::sns::report) trait SnsCacheStatusSummaryView {
    fn refresh_attempt_path(&self) -> &str;
}

///
/// SnsCacheStatusAttemptView
///
/// Attempt identity required for numeric cache-status lookups.
///

pub(in crate::sns::report) trait SnsCacheStatusAttemptView {
    fn id(&self) -> usize;
}

///
/// SnsCacheStatusFamily
///
/// Family-specific hooks required by the shared SNS cache-status report flow.
///

pub(in crate::sns::report) trait SnsCacheStatusFamily {
    type Summary: SnsCacheStatusSummaryView;
    type Attempt: Clone + SnsCacheStatusAttemptView;

    const COLLECTION: &'static str;

    fn network_cache_dir(icp_root: &Path, network: &str) -> PathBuf;
    fn find_cache_by_id(
        icp_root: &Path,
        network: &str,
        id: usize,
    ) -> Result<Option<Self::Summary>, SnsHostError>;
    fn root_cache_paths(
        icp_root: &Path,
        network: &str,
        root_canister_id: &str,
    ) -> SnsCacheStatusPaths;
    fn load_root_cache_summary(
        cache_path: PathBuf,
        network: &str,
    ) -> Result<Self::Summary, SnsHostError>;
    fn read_attempt_status(
        attempt_path: &Path,
        network: &str,
    ) -> Result<Option<Self::Attempt>, SnsHostError>;
}

/// Build a cache-status lookup for an SNS cache family by list id or root principal.
pub(in crate::sns::report) fn build_sns_cache_status_lookup<Family>(
    network: &str,
    icp_root: &Path,
    input: &str,
) -> Result<SnsCacheStatusLookup<Family::Summary, Family::Attempt>, SnsHostError>
where
    Family: SnsCacheStatusFamily,
{
    enforce_mainnet_network(network)?;
    let cache_root = Family::network_cache_dir(icp_root, network)
        .display()
        .to_string();
    if let Ok(id) = input.parse::<usize>() {
        return build_id_cache_status_lookup::<Family>(network, icp_root, cache_root, id);
    }
    build_root_cache_status_lookup::<Family>(network, icp_root, input, cache_root)
}

fn build_id_cache_status_lookup<Family>(
    network: &str,
    icp_root: &Path,
    cache_root: String,
    id: usize,
) -> Result<SnsCacheStatusLookup<Family::Summary, Family::Attempt>, SnsHostError>
where
    Family: SnsCacheStatusFamily,
{
    let cache = Family::find_cache_by_id(icp_root, network, id)?;
    let (refresh_attempt_path, latest_attempt) = match cache.as_ref() {
        Some(cache) => {
            let path = cache.refresh_attempt_path().to_string();
            let attempt = Family::read_attempt_status(Path::new(&path), network)?;
            (Some(path), attempt)
        }
        None => match find_attempt_by_id::<Family>(network, icp_root, id)? {
            Some((path, attempt)) => (Some(path.display().to_string()), Some(attempt)),
            None => (None, None),
        },
    };
    let expected_cache_path = cache
        .is_none()
        .then(|| {
            refresh_attempt_path
                .as_deref()
                .map(Path::new)
                .map(|path| path.with_file_name("full.json").display().to_string())
        })
        .flatten();
    Ok(SnsCacheStatusLookup {
        cache_root,
        cache,
        expected_cache_path,
        refresh_attempt_path,
        latest_attempt,
    })
}

fn find_attempt_by_id<Family>(
    network: &str,
    icp_root: &Path,
    id: usize,
) -> Result<Option<(PathBuf, Family::Attempt)>, SnsHostError>
where
    Family: SnsCacheStatusFamily,
{
    let network_dir = Family::network_cache_dir(icp_root, network);
    let attempt_paths = collect_full_collection_attempt_paths(&network_dir, Family::COLLECTION)
        .map_err(|source| SnsHostError::ReadCache {
            path: network_dir,
            source,
        })?;
    let mut matching = Vec::new();
    for path in attempt_paths {
        if let Some(attempt) = Family::read_attempt_status(&path, network)?
            && attempt.id() == id
        {
            matching.push((path, attempt));
        }
    }
    match matching.len() {
        0 => Ok(None),
        1 => Ok(matching.pop()),
        _ => Err(SnsHostError::AmbiguousRefreshAttemptId { id }),
    }
}

fn build_root_cache_status_lookup<Family>(
    network: &str,
    icp_root: &Path,
    input: &str,
    cache_root: String,
) -> Result<SnsCacheStatusLookup<Family::Summary, Family::Attempt>, SnsHostError>
where
    Family: SnsCacheStatusFamily,
{
    let root_canister_id = parse_sns_root_canister_input(input)?;
    let paths = Family::root_cache_paths(icp_root, network, &root_canister_id);
    let cache = if paths.cache_path.is_file() {
        Some(Family::load_root_cache_summary(
            paths.cache_path.clone(),
            network,
        )?)
    } else {
        None
    };
    let latest_attempt = Family::read_attempt_status(&paths.attempt_path, network)?;
    Ok(SnsCacheStatusLookup {
        cache_root,
        cache,
        expected_cache_path: Some(paths.cache_path.display().to_string()),
        refresh_attempt_path: Some(paths.attempt_path.display().to_string()),
        latest_attempt,
    })
}

impl SnsCacheStatusAttemptView for SnsRefreshAttemptStatus {
    fn id(&self) -> usize {
        self.id
    }
}
