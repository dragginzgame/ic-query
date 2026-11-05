//! Module: sns::report::cache_refresh
//!
//! Responsibility: shared SNS snapshot lookup, locking, attempt lifecycle, and publication.
//! Does not own: family paging, row validation, report DTOs, or text rendering.
//! Boundary: one lifecycle serves neuron complete-cache refreshes.

use crate::cache_file::write_collection_snapshot;
use crate::report::CollectionSourceRequest;
use crate::{
    HostCacheError,
    cache::CacheCollectionCompleteness,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SnapshotEnvelope, SnapshotJsonPaths, SnapshotRefreshProgress,
        publish_snapshot_with_attempt, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
    sns::report::{
        SNS_CACHE_COMPONENT, SnsHostError, SnsReportContext, SnsSnapshotRefreshReport,
        SnsSnapshotRefreshRequest,
        cache_attempt::{
            SnsRefreshContext, write_complete_sns_refresh_attempt,
            write_failed_sns_refresh_attempt, write_starting_sns_refresh_attempt,
        },
        cache_paths::sns_snapshot_cache_paths,
        cache_storage::{SnsCacheMetadata, SnsCacheStorageFamily},
        target::{resolve_neuron_target, validate_sns_refresh_page_size},
    },
};
use std::marker::PhantomData;

///
/// SnsSnapshotRefreshContext
///
/// Resolved, locked context shared by one SNS complete-cache refresh.
///

pub(in crate::sns::report) struct SnsSnapshotRefreshContext<'a, Collection> {
    /// Family request that selected the SNS and cache root.
    pub(in crate::sns::report) request: &'a SnsSnapshotRefreshRequest,
    /// Canonical source request shared by lookup and collection calls.
    pub(in crate::sns::report) fetch_request: CollectionSourceRequest,
    /// Resolved SNS identity and canister principals.
    pub(in crate::sns::report) governance_canister_id: String,
    /// Complete-cache, lock, and attempt paths for the family collection.
    pub(in crate::sns::report) paths: &'a SnapshotJsonPaths,
    /// Whether publication replaces an existing complete cache.
    pub(in crate::sns::report) replaced_existing_cache: bool,
    collection: PhantomData<Collection>,
}

impl<Collection> SnsSnapshotRefreshContext<'_, Collection> {
    /// Borrow the shared inputs required by the attempt-sidecar writer.
    pub(in crate::sns::report) fn refresh_context(&self) -> SnsRefreshContext<'_> {
        SnsRefreshContext {
            path: &self.paths.refresh_attempt_path,
            request: self.request,
            fetch_request: &self.fetch_request,
            governance_canister_id: &self.governance_canister_id,
        }
    }
}

/// Resolve one SNS target, acquire its family lock, and run its attempt lifecycle.
#[expect(
    clippy::future_not_send,
    reason = "Borrowing AsyncFn callbacks select their future Send bounds at the concrete report caller; downstream probes admit those futures."
)]
pub(in crate::sns::report) async fn run_resolved_sns_snapshot_refresh<Collection, Report>(
    request: &SnsSnapshotRefreshRequest,

    run_locked: impl AsyncFnOnce(
        &SnsSnapshotRefreshContext<'_, Collection>,
    ) -> Result<Report, SnsHostError>,
) -> Result<Report, SnsHostError>
where
    Collection: SnsCacheStorageFamily,
{
    validate_sns_refresh_page_size(request.page_size)?;
    let lookup = resolve_neuron_target(&request.target)?;
    let paths = sns_snapshot_cache_paths::<Collection>(
        &request.cache_root,
        &request.target.network,
        &lookup.governance_canister_id,
    );
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            cache_root: &request.cache_root,
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
        },
        |error| SnsHostError::from(HostCacheError::operation(SNS_CACHE_COMPONENT, error)),
        async |replaced_existing_snapshot| {
            let context = SnsSnapshotRefreshContext {
                request,
                fetch_request: lookup.fetch_request,
                governance_canister_id: lookup.governance_canister_id,
                paths: &paths,
                replaced_existing_cache: replaced_existing_snapshot,
                collection: PhantomData,
            };
            run_snapshot_refresh_with_attempts(
                || write_starting_sns_refresh_attempt(context.refresh_context()),
                async || run_locked(&context).await,
                |error| write_failed_sns_refresh_attempt(context.refresh_context(), error),
            )
            .await
        },
    )
    .await
}

/// Atomically publish a complete SNS family snapshot, then finalize its attempt sidecar.
pub(in crate::sns::report) fn publish_complete_sns_snapshot<Collection>(
    context: &SnsSnapshotRefreshContext<'_, Collection>,
    page_count: u32,
    last_cursor: Option<String>,
    data: Collection::Data,
) -> Result<SnsSnapshotRefreshReport, SnsHostError>
where
    Collection: SnsCacheStorageFamily,
{
    let row_count = Collection::row_count(&data);
    let cache = SnapshotEnvelope {
        schema_version: Collection::CACHE_SCHEMA_VERSION,
        network: context.fetch_request.network.clone(),
        fetched_at: context.fetch_request.fetched_at.clone(),
        source_endpoint: context.fetch_request.endpoint.clone(),
        fetched_by: context.fetch_request.fetched_by.clone(),
        domain: "sns-neuron".to_string(),
        entity: context.governance_canister_id.clone(),
        collection: Collection::COLLECTION.to_string(),
        scope: "full".to_string(),
        metadata: SnsCacheMetadata {
            governance_canister_id: context.governance_canister_id.clone(),
        },
        completeness: CacheCollectionCompleteness::api_exhausted(
            context.request.page_size,
            page_count,
            row_count,
            false,
        ),
        data,
    };
    let attempt_finalization_error = publish_snapshot_with_attempt(
        || {
            write_collection_snapshot(
                SNS_CACHE_COMPONENT,
                &context.request.cache_root,
                &context.paths.snapshot_path,
                &cache,
            )
            .map_err(SnsHostError::from)
        },
        || {
            write_complete_sns_refresh_attempt(
                context.refresh_context(),
                SnapshotRefreshProgress::new(page_count, row_count, last_cursor),
            )
        },
    )?;
    let (report_context, _) = SnsReportContext::from_snapshot(cache);
    Ok(SnsSnapshotRefreshReport {
        schema_version: 1,
        context: report_context,
        collection: Collection::COLLECTION.to_string(),
        cache_path: context.paths.snapshot_path.display().to_string(),
        refresh_lock_path: context.paths.refresh_lock_path.display().to_string(),
        refresh_attempt_path: context.paths.refresh_attempt_path.display().to_string(),
        page_size: context.request.page_size,
        page_count,
        row_count,
        complete: true,
        replaced_existing_cache: context.replaced_existing_cache,
        wrote_cache: true,
        attempt_finalization_error,
    })
}
