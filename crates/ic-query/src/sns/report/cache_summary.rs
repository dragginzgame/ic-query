//! Module: sns::report::cache_summary
//!
//! Responsibility: share cache-summary projection and lookup helpers across SNS cache reports.
//! Does not own: cache storage, refresh-attempt persistence, or text rendering.
//! Boundary: keeps common cache-summary fields and ordering consistent.

use crate::sns::report::{SnsHostError, enforce_mainnet_network};
use candid::Principal;
use std::path::{Path, PathBuf};

///
/// SnsCacheListLookup
///
/// Shared lookup result used to assemble SNS cache-list reports.
///

pub(in crate::sns::report) struct SnsCacheListLookup<Summary> {
    pub(in crate::sns::report) cache_root: String,
    pub(in crate::sns::report) caches: Vec<Summary>,
}

///
/// SnsCacheSummarySortKey
///
/// Stable ordering key implemented by SNS cache summary report rows.
///

pub(in crate::sns::report) trait SnsCacheSummarySortKey {
    fn id(&self) -> usize;
    fn root_canister_id(&self) -> &str;
    fn cache_path(&self) -> &str;
    fn cache_error(&self) -> Option<&str>;
}

macro_rules! project_sns_cache_summary {
    (valid $summary:ident, $cache_path:expr, $attempt_path:expr, $cache:expr) => {{
        let cache = $cache;
        let latest_attempt = $crate::sns::report::cache_attempt::read_sns_refresh_attempt_status(
            $attempt_path,
            &cache.network,
        );
        $summary {
            id: cache.metadata.id,
            name: cache.metadata.name,
            root_canister_id: cache.metadata.root_canister_id,
            governance_canister_id: cache.metadata.governance_canister_id,
            cache_status: $crate::snapshot_cache::SNAPSHOT_CACHE_STATUS_OK.to_string(),
            cache_error: None,
            complete: cache.completeness.is_api_exhausted(),
            row_count: cache.completeness.row_count,
            page_count: cache.completeness.page_count,
            page_size: cache.completeness.page_size,
            fetched_at: cache.fetched_at,
            source_endpoint: cache.source_endpoint,
            cache_path: ($cache_path).display().to_string(),
            refresh_attempt_path: ($attempt_path).display().to_string(),
            latest_attempt,
        }
    }};
    (invalid $summary:ident, $cache_path:expr, $attempt_path:expr, $network:expr, $error:expr) => {
        $summary {
            id: 0,
            name: "-".to_string(),
            root_canister_id: $crate::sns::report::cache_summary::root_from_cache_path($cache_path),
            governance_canister_id: "-".to_string(),
            cache_status: $crate::snapshot_cache::SNAPSHOT_CACHE_STATUS_INVALID.to_string(),
            cache_error: Some(($error).to_string()),
            complete: false,
            row_count: 0,
            page_count: 0,
            page_size: 0,
            fetched_at: "-".to_string(),
            source_endpoint: "-".to_string(),
            cache_path: ($cache_path).display().to_string(),
            refresh_attempt_path: ($attempt_path).display().to_string(),
            latest_attempt: $crate::sns::report::cache_attempt::read_sns_refresh_attempt_status(
                $attempt_path,
                $network,
            ),
        }
    };
}

pub(in crate::sns::report) use project_sns_cache_summary;

///
/// SnsCacheListFamily
///
/// Family-specific hooks required by the shared SNS cache-list report flow.
///

pub(in crate::sns::report) trait SnsCacheListFamily {
    type Summary: SnsCacheSummarySortKey;

    fn network_cache_dir(cache_root: &Path, network: &str) -> PathBuf;
    fn list_cache_summaries(
        cache_root: &Path,
        network: &str,
    ) -> Result<Vec<Self::Summary>, SnsHostError>;
}

/// Build a deterministic cache-list lookup for one SNS cache family.
pub(in crate::sns::report) fn build_sns_cache_list_lookup<Family>(
    network: &str,
    cache_root: &Path,
) -> Result<SnsCacheListLookup<Family::Summary>, SnsHostError>
where
    Family: SnsCacheListFamily,
{
    enforce_mainnet_network(network)?;
    let network_cache_root = Family::network_cache_dir(cache_root, network)
        .display()
        .to_string();
    let mut caches = Family::list_cache_summaries(cache_root, network)?;
    sort_sns_cache_summaries(&mut caches);
    Ok(SnsCacheListLookup {
        cache_root: network_cache_root,
        caches,
    })
}

/// Parse and normalize an SNS root canister principal input.
pub(in crate::sns::report) fn parse_sns_root_canister_input(
    input: &str,
) -> Result<String, SnsHostError> {
    Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })
        .map(|principal| principal.to_text())
}

/// Sort SNS cache summaries by stable list id and root principal.
pub(in crate::sns::report) fn sort_sns_cache_summaries<T>(caches: &mut [T])
where
    T: SnsCacheSummarySortKey,
{
    caches.sort_by(|left, right| {
        left.id()
            .cmp(&right.id())
            .then_with(|| left.root_canister_id().cmp(right.root_canister_id()))
    });
}

/// Find a valid SNS cache summary by id without loading unrelated snapshots.
pub(in crate::sns::report) fn find_sns_cache_summary_by_id<T>(
    paths: impl IntoIterator<Item = PathBuf>,
    id: usize,
    mut read_id: impl FnMut(&Path) -> Result<usize, SnsHostError>,
    mut load_summary: impl FnMut(PathBuf) -> T,
) -> Result<Option<T>, SnsHostError>
where
    T: SnsCacheSummarySortKey,
{
    let mut matching = None;
    for path in paths {
        let Ok(candidate_id) = read_id(&path) else {
            continue;
        };
        if candidate_id != id {
            continue;
        }
        let summary = load_summary(path);
        if summary.id() != id || summary.cache_error().is_some() {
            continue;
        }
        if matching.replace(summary).is_some() {
            return Err(SnsHostError::AmbiguousCacheId { id });
        }
    }
    Ok(matching)
}

/// Recover an SNS root identity from a complete snapshot cache path.
pub(in crate::sns::report) fn root_from_cache_path(cache_path: &Path) -> String {
    cache_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map_or_else(
            || "-".to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
}

#[cfg(test)]
mod tests {
    use super::{SnsCacheSummarySortKey, find_sns_cache_summary_by_id};
    use crate::sns::report::SnsHostError;
    use std::{cell::Cell, path::PathBuf};

    struct Summary {
        id: usize,
        path: String,
        error: Option<String>,
    }

    impl SnsCacheSummarySortKey for Summary {
        fn id(&self) -> usize {
            self.id
        }

        fn root_canister_id(&self) -> &str {
            &self.path
        }

        fn cache_path(&self) -> &str {
            &self.path
        }

        fn cache_error(&self) -> Option<&str> {
            self.error.as_deref()
        }
    }

    #[test]
    fn id_lookup_loads_only_the_matching_snapshot() {
        let paths = (1..=100)
            .map(|id| PathBuf::from(id.to_string()))
            .collect::<Vec<_>>();
        let header_reads = Cell::new(0);
        let snapshot_loads = Cell::new(0);

        let summary = find_sns_cache_summary_by_id(
            paths,
            73,
            |path| {
                header_reads.set(header_reads.get() + 1);
                path.to_string_lossy()
                    .parse::<usize>()
                    .map_err(|_| SnsHostError::InvalidLookup {
                        input: path.display().to_string(),
                    })
            },
            |path| {
                snapshot_loads.set(snapshot_loads.get() + 1);
                Summary {
                    id: path
                        .to_string_lossy()
                        .parse()
                        .expect("numeric fixture path"),
                    path: path.display().to_string(),
                    error: None,
                }
            },
        )
        .expect("lookup succeeds")
        .expect("matching summary");

        assert_eq!(summary.id, 73);
        assert_eq!(header_reads.get(), 100);
        assert_eq!(snapshot_loads.get(), 1);
    }

    #[test]
    fn id_lookup_rejects_multiple_valid_matching_snapshots() {
        let result = find_sns_cache_summary_by_id(
            [PathBuf::from("a"), PathBuf::from("b")],
            7,
            |_| Ok(7),
            |path| Summary {
                id: 7,
                path: path.display().to_string(),
                error: None,
            },
        );

        assert!(matches!(
            result,
            Err(SnsHostError::AmbiguousCacheId { id: 7 })
        ));
    }
}
