//! Module: sns::report::cache_refresh
//!
//! Responsibility: shared SNS snapshot lookup, locking, attempt lifecycle, and publication.
//! Does not own: family paging, row validation, report DTOs, or text rendering.
//! Boundary: one lifecycle serves proposal and neuron complete-cache refreshes.

use crate::cache_file::write_managed_json_pretty_atomically;
use crate::{
    HostCacheError,
    cache::CacheCollectionCompleteness,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SnapshotEnvelope, SnapshotJsonPaths, SnapshotRefreshProgress,
        publish_snapshot_with_attempt, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh,
    },
    sns::report::{
        MAINNET_SNS_WASM_CANISTER_ID, SNS_CACHE_COMPONENT, SnsHostError, SnsSnapshotRefreshReport,
        SnsSnapshotRefreshRequest,
        cache_attempt::{
            SnsRefreshContext, write_complete_sns_refresh_attempt,
            write_failed_sns_refresh_attempt, write_starting_sns_refresh_attempt,
        },
        cache_paths::{SnsCacheCollection, sns_snapshot_cache_paths},
        cache_storage::SnsCacheMetadata,
        lookup::{resolve_sns_lookup, validate_sns_refresh_page_size},
        source::{MainnetSns, SnsDiscoverySource, SnsSourceRequest},
    },
};
use serde::Serialize;
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
    pub(in crate::sns::report) fetch_request: SnsSourceRequest,
    /// Resolved SNS identity and canister principals.
    pub(in crate::sns::report) sns: MainnetSns,
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
            sns: &self.sns,
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
    source: &dyn SnsDiscoverySource,

    run_locked: impl AsyncFnOnce(
        &SnsSnapshotRefreshContext<'_, Collection>,
    ) -> Result<Report, SnsHostError>,
) -> Result<Report, SnsHostError>
where
    Collection: SnsCacheCollection,
{
    validate_sns_refresh_page_size(request.page_size)?;
    let lookup = resolve_sns_lookup(&request.lookup, source).await?;
    let paths = sns_snapshot_cache_paths::<Collection>(
        &request.cache_root,
        &request.lookup.network,
        &lookup.sns.canisters.root_canister_id,
    );
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            cache_root: &request.cache_root,
            snapshot_path: &paths.snapshot_path,
            refresh_lock_path: &paths.refresh_lock_path,
        },
        |error| SnsHostError::from(HostCacheError::operation(SNS_CACHE_COMPONENT, error)),
        async |refresh_state| {
            let context = SnsSnapshotRefreshContext {
                request,
                fetch_request: lookup.fetch_request,
                sns: lookup.sns,
                paths: &paths,
                replaced_existing_cache: refresh_state.replaced_existing_snapshot,
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
pub(in crate::sns::report) fn publish_complete_sns_snapshot<Collection, Data>(
    context: &SnsSnapshotRefreshContext<'_, Collection>,
    cache_schema_version: u32,
    page_count: u32,
    row_count: usize,
    last_cursor: Option<String>,
    data: Data,
) -> Result<SnsSnapshotRefreshReport, SnsHostError>
where
    Collection: SnsCacheCollection,
    Data: Serialize,
{
    let cache = SnapshotEnvelope {
        schema_version: cache_schema_version,
        network: context.fetch_request.network.clone(),
        fetched_at: context.fetch_request.fetched_at.clone(),
        source_endpoint: context.fetch_request.endpoint.clone(),
        fetched_by: context.fetch_request.fetched_by.clone(),
        domain: "sns".to_string(),
        entity: context.sns.canisters.root_canister_id.clone(),
        collection: Collection::COLLECTION.to_string(),
        scope: "full".to_string(),
        metadata: SnsCacheMetadata {
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            id: context.sns.id,
            name: context.sns.name.clone(),
            root_canister_id: context.sns.canisters.root_canister_id.clone(),
            governance_canister_id: context.sns.canisters.governance_canister_id.clone(),
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
            write_managed_json_pretty_atomically(
                &context.request.cache_root,
                &context.paths.snapshot_path,
                &cache,
                crate::cache_file::MAX_COLLECTION_SNAPSHOT_BYTES,
                |path, source| {
                    SnsHostError::from(HostCacheError::serialize_cache(
                        SNS_CACHE_COMPONENT,
                        path,
                        source,
                    ))
                },
                |error| SnsHostError::from(HostCacheError::operation(SNS_CACHE_COMPONENT, error)),
            )
        },
        || {
            write_complete_sns_refresh_attempt(
                context.refresh_context(),
                SnapshotRefreshProgress::new(page_count, row_count, last_cursor),
            )
        },
    )?;
    Ok(SnsSnapshotRefreshReport {
        schema_version: 1,
        network: context.fetch_request.network.clone(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: context.fetch_request.fetched_at.clone(),
        source_endpoint: context.fetch_request.endpoint.clone(),
        fetched_by: context.fetch_request.fetched_by.clone(),
        id: context.sns.id,
        name: context.sns.name.clone(),
        root_canister_id: context.sns.canisters.root_canister_id.clone(),
        governance_canister_id: context.sns.canisters.governance_canister_id.clone(),
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
