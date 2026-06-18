//! Module: sns::report::cache_status
//!
//! Responsibility: share SNS cache status lookup flow across snapshot families.
//! Does not own: cache storage, refresh attempts, report DTOs, or rendering.
//! Boundary: resolves id/root status inputs into family-owned summary values.

use crate::sns::report::{SnsHostError, enforce_mainnet_network, parse_sns_root_canister_input};
use std::path::{Path, PathBuf};

pub(in crate::sns::report) struct SnsCacheStatusPaths {
    pub(in crate::sns::report) cache_path: PathBuf,
    pub(in crate::sns::report) attempt_path: PathBuf,
}

pub(in crate::sns::report) struct SnsCacheStatusLookup<Summary, Attempt> {
    pub(in crate::sns::report) cache_root: String,
    pub(in crate::sns::report) cache: Option<Summary>,
    pub(in crate::sns::report) expected_cache_path: Option<String>,
    pub(in crate::sns::report) refresh_attempt_path: Option<String>,
    pub(in crate::sns::report) latest_attempt: Option<Attempt>,
}

pub(in crate::sns::report) trait SnsCacheStatusSummaryView {
    type Attempt: Clone;

    fn refresh_attempt_path(&self) -> &str;
    fn latest_attempt(&self) -> Option<Self::Attempt>;
}

pub(in crate::sns::report) trait SnsCacheStatusFamily {
    type Summary: SnsCacheStatusSummaryView<Attempt = Self::Attempt>;
    type Attempt: Clone;

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
    fn read_attempt_status(attempt_path: &Path) -> Option<Self::Attempt>;
}

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
    let refresh_attempt_path = cache
        .as_ref()
        .map(|cache| cache.refresh_attempt_path().to_string());
    let latest_attempt = cache
        .as_ref()
        .and_then(SnsCacheStatusSummaryView::latest_attempt);
    Ok(SnsCacheStatusLookup {
        cache_root,
        cache,
        expected_cache_path: None,
        refresh_attempt_path,
        latest_attempt,
    })
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
    let latest_attempt = cache.as_ref().map_or_else(
        || Family::read_attempt_status(&paths.attempt_path),
        SnsCacheStatusSummaryView::latest_attempt,
    );
    Ok(SnsCacheStatusLookup {
        cache_root,
        cache,
        expected_cache_path: Some(paths.cache_path.display().to_string()),
        refresh_attempt_path: Some(paths.attempt_path.display().to_string()),
        latest_attempt,
    })
}
