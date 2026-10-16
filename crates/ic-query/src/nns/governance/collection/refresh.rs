//! Governance snapshot locking, attempt sequencing and publication.

use super::{
    NnsGovernanceRefreshReport, NnsGovernanceRefreshRequest, mainnet_governance_cache_metadata,
    write_complete_governance_refresh_attempt, write_failed_governance_refresh_attempt,
    write_starting_governance_refresh_attempt,
};
use crate::{
    HostCacheError,
    cache::CacheCollectionCompleteness,
    cache_file::write_managed_json_pretty_atomically,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SnapshotEnvelope, SnapshotJsonPaths, SnapshotRefreshProgress,
        publish_snapshot_with_attempt, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use serde::Serialize;
use std::fmt::Display;

///
/// GovernanceSnapshotRefreshContext
///
/// Locked request and paths shared by collection and publication of one Governance snapshot.
///

pub(in crate::nns) struct GovernanceSnapshotRefreshContext<'a> {
    request: &'a NnsGovernanceRefreshRequest,
    paths: &'a SnapshotJsonPaths,
    component: &'static str,
    replaced_existing_cache: bool,
}

impl GovernanceSnapshotRefreshContext<'_> {
    /// Publish an admitted complete collection, then retain any attempt-finalization failure.
    pub(in crate::nns) fn publish<Data, Error>(
        &self,
        collection: &'static str,
        fetched_by: &'static str,
        progress: SnapshotRefreshProgress,
        data: Data,
    ) -> Result<NnsGovernanceRefreshReport, Error>
    where
        Data: Serialize,
        Error: From<HostCacheError> + Display,
    {
        let cache = SnapshotEnvelope {
            schema_version: 1,
            network: MAINNET_NETWORK.to_string(),
            source_endpoint: self.request.source_endpoint.clone(),
            fetched_at: format_utc_timestamp_secs(self.request.now_unix_secs),
            fetched_by: fetched_by.to_string(),
            domain: "nns".to_string(),
            entity: "governance".to_string(),
            collection: collection.to_string(),
            scope: "full".to_string(),
            metadata: mainnet_governance_cache_metadata(),
            completeness: CacheCollectionCompleteness::api_exhausted(
                self.request.page_size,
                progress.pages_fetched,
                progress.rows_fetched,
                false,
            ),
            data,
        };
        let attempt_finalization_error = publish_snapshot_with_attempt(
            || {
                write_managed_json_pretty_atomically(
                    &self.request.cache_root,
                    &self.paths.snapshot_path,
                    &cache,
                    crate::cache_file::MAX_COLLECTION_SNAPSHOT_BYTES,
                    |path, source| {
                        Error::from(HostCacheError::serialize_cache(
                            self.component,
                            path,
                            source,
                        ))
                    },
                    |source| Error::from(HostCacheError::operation(self.component, source)),
                )
            },
            || {
                write_complete_governance_refresh_attempt(
                    &self.paths.refresh_attempt_path,
                    self.request,
                    self.component,
                    progress,
                )
                .map_err(Error::from)
            },
        )?;
        Ok(NnsGovernanceRefreshReport::from_snapshot(
            &cache,
            self.paths,
            self.replaced_existing_cache,
            attempt_finalization_error,
        ))
    }
}

/// Run one admitted family refresh under its lock and attempt lifecycle.
pub(in crate::nns) async fn run_governance_snapshot_refresh<Error>(
    request: &NnsGovernanceRefreshRequest,
    paths: &SnapshotJsonPaths,
    component: &'static str,
    lock_stale_after_seconds: u64,
    run_locked: impl AsyncFnOnce(
        &GovernanceSnapshotRefreshContext<'_>,
    ) -> Result<NnsGovernanceRefreshReport, Error>,
) -> Result<NnsGovernanceRefreshReport, Error>
where
    Error: From<HostCacheError> + Display,
{
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            cache_root: &request.cache_root,
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds,
        },
        |source| Error::from(HostCacheError::operation(component, source)),
        async |state| {
            let context = GovernanceSnapshotRefreshContext {
                request,
                paths,
                component,
                replaced_existing_cache: state.replaced_existing_snapshot,
            };
            run_snapshot_refresh_with_attempts(
                || {
                    write_starting_governance_refresh_attempt(
                        &paths.refresh_attempt_path,
                        request,
                        component,
                    )
                    .map_err(Error::from)
                },
                async || run_locked(&context).await,
                |error| {
                    let _ = write_failed_governance_refresh_attempt(
                        &paths.refresh_attempt_path,
                        request,
                        component,
                        error.to_string(),
                    );
                },
            )
            .await
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::temp_dir;
    use std::fs;

    #[test]
    fn publication_preserves_distinct_snapshot_and_attempt_failures() {
        for snapshot_fails in [false, true] {
            let root = temp_dir("ic-query-governance-publication");
            let request = NnsGovernanceRefreshRequest::new(
                &root,
                "ic",
                "https://replica.example",
                1_700_000_000,
                25,
            );
            let paths = SnapshotJsonPaths {
                snapshot_path: root.join("snapshot.json"),
                refresh_lock_path: root.join("refresh.lock"),
                refresh_attempt_path: root.join("attempt.json"),
            };
            if snapshot_fails {
                fs::create_dir(&paths.snapshot_path).unwrap();
                fs::write(&paths.refresh_attempt_path, b"retained attempt evidence").unwrap();
            } else {
                fs::create_dir(&paths.refresh_attempt_path).unwrap();
            }
            let context = GovernanceSnapshotRefreshContext {
                request: &request,
                paths: &paths,
                component: "NNS publication fixture",
                replaced_existing_cache: false,
            };
            let result = context.publish::<_, HostCacheError>(
                "proposals",
                "fixture",
                SnapshotRefreshProgress::new(2, 3, None),
                serde_json::json!({"proposals": [1, 2, 3]}),
            );
            if snapshot_fails {
                assert!(matches!(result, Err(HostCacheError::Operation { .. })));
                assert_eq!(
                    fs::read(&paths.refresh_attempt_path).unwrap(),
                    b"retained attempt evidence"
                );
            } else {
                let report = result.unwrap();
                assert!(report.wrote_cache);
                assert!(report.attempt_finalization_error.is_some());
                let snapshot: serde_json::Value =
                    serde_json::from_slice(&fs::read(&paths.snapshot_path).unwrap()).unwrap();
                assert_eq!(snapshot["schema_version"], 1);
                assert_eq!(snapshot["collection"], "proposals");
                assert_eq!(snapshot["completeness"]["row_count"], report.row_count);
                assert_eq!(snapshot["source_endpoint"], "https://replica.example");
                assert_eq!(snapshot["fetched_by"], "fixture");
                assert!(paths.refresh_attempt_path.is_dir());
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
}
