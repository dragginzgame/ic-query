//! Governance snapshot locking, attempt sequencing and publication.

use super::{
    NnsGovernanceRefreshReport, NnsGovernanceRefreshRequest, mainnet_governance_cache_metadata,
    write_complete_governance_refresh_attempt, write_failed_governance_refresh_attempt,
    write_starting_governance_refresh_attempt,
};
use crate::{
    HostCacheError,
    cache::CacheCollectionCompleteness,
    cache_file::write_collection_snapshot,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SnapshotEnvelope, SnapshotJsonPaths, SnapshotRefreshProgress,
        publish_snapshot_with_attempt, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
    subnet_catalog::MAINNET_NETWORK,
};
use ic_host_text::timestamp::format_utc_timestamp_secs;
use serde::Serialize;
use std::fmt::Display;

// Both NNS collectors retain their own paging state and rows. Only their
// progress, bounds and attempt-sidecar projections are mechanically identical.
macro_rules! governance_refresh_bookkeeping {
    ($collection:ident, $component:ident, $rows:ident, $cursor:ident) => {
        fn progress(&self) -> $crate::PagedQueryProgress {
            $crate::PagedQueryProgress {
                collection: $crate::QueryProgressCollection::$collection,
                target: None,
                pages_fetched: self.collection_state.pages_fetched(),
                rows_fetched: self.$rows.len(),
            }
        }

        fn max_pages_reached(&self) -> bool {
            self.request
                .max_pages
                .is_some_and(|maximum| self.collection_state.pages_fetched() >= maximum)
        }

        fn incomplete_refresh_error(&self, reason: &'static str) -> Self::Error {
            Self::Error::IncompleteRefresh {
                pages_fetched: self.collection_state.pages_fetched(),
                rows_fetched: self.$rows.len(),
                reason: reason.to_string(),
            }
        }

        fn write_running_attempt(&self) -> Result<(), Self::Error> {
            $crate::nns::governance::write_running_governance_refresh_attempt(
                self.attempt_path,
                self.request,
                $component,
                $crate::snapshot_cache::SnapshotRefreshProgress::new(
                    self.collection_state.pages_fetched(),
                    self.$rows.len(),
                    self.collection_state
                        .$cursor()
                        .map(|cursor| cursor.to_string()),
                ),
            )
            .map_err(Self::Error::from)
        }
    };
}
pub(in crate::nns) use governance_refresh_bookkeeping;

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
        collection: crate::nns::governance::NnsGovernanceCollection,
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
                write_collection_snapshot(
                    self.component,
                    &self.request.cache_root,
                    &self.paths.snapshot_path,
                    &cache,
                )
                .map_err(Error::from)
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
            collection,
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
        },
        |source| Error::from(HostCacheError::operation(component, source)),
        async |replaced_existing_snapshot| {
            let context = GovernanceSnapshotRefreshContext {
                request,
                paths,
                component,
                replaced_existing_cache: replaced_existing_snapshot,
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
mod tests;
