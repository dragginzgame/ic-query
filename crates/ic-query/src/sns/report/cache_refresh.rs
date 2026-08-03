//! Module: sns::report::cache_refresh
//!
//! Responsibility: shared SNS snapshot lookup, locking, attempt lifecycle, and publication.
//! Does not own: family paging, row validation, report DTOs, or text rendering.
//! Boundary: one lifecycle serves proposal and neuron complete-cache refreshes.

use crate::{
    cache::CacheCollectionCompleteness,
    snapshot_cache::{
        LockedSnapshotRefreshRequest, SnapshotEnvelope, SnapshotRefreshProgress,
        publish_snapshot_with_attempt, run_snapshot_refresh_with_attempts,
        with_locked_snapshot_refresh, write_snapshot_json,
    },
    sns::report::{
        SnsHostError,
        cache_attempt::{
            SnsRefreshContext, SnsRefreshRequestView, write_complete_sns_refresh_attempt,
            write_failed_sns_refresh_attempt, write_starting_sns_refresh_attempt,
        },
        cache_paths::{SnsCacheCollection, SnsSnapshotCachePaths},
        cache_storage::SnsCacheMetadata,
        lookup::{lookup_request_from_parts, resolve_sns_lookup, validate_sns_refresh_page_size},
        source::{JoinedMainnetSnsInventory, MainnetSns, SnsDiscoverySource, SnsSourceRequest},
    },
};
use serde::Serialize;

///
/// SnsSnapshotRefreshContext
///
/// Resolved, locked context shared by one SNS complete-cache refresh.
///

pub(in crate::sns::report) struct SnsSnapshotRefreshContext<'a, Request, Collection> {
    /// Family request that selected the SNS and cache root.
    pub(in crate::sns::report) request: &'a Request,
    /// Canonical source request shared by lookup and collection calls.
    pub(in crate::sns::report) fetch_request: SnsSourceRequest,
    /// Targeted joined discovery context that resolved the SNS.
    pub(in crate::sns::report) list: JoinedMainnetSnsInventory,
    /// Stable list position assigned to the resolved SNS.
    pub(in crate::sns::report) id: usize,
    /// Resolved SNS identity and canister principals.
    pub(in crate::sns::report) sns: MainnetSns,
    /// Complete-cache, lock, and attempt paths for the family collection.
    pub(in crate::sns::report) paths: SnsSnapshotCachePaths<Collection>,
    /// Whether publication replaces an existing complete cache.
    pub(in crate::sns::report) replaced_existing_cache: bool,
}

impl<Request, Collection> SnsSnapshotRefreshContext<'_, Request, Collection>
where
    Request: SnsRefreshRequestView,
{
    /// Borrow the shared inputs required by the attempt-sidecar writer.
    pub(in crate::sns::report) fn refresh_context(&self) -> SnsRefreshContext<'_> {
        SnsRefreshContext {
            path: &self.paths.attempt_path,
            request: self.request,
            fetch_request: &self.fetch_request,
            sns: &self.sns,
        }
    }
}

/// Resolve one SNS target, acquire its family lock, and run its attempt lifecycle.
pub(in crate::sns::report) fn run_resolved_sns_snapshot_refresh<Request, Collection, Report>(
    request: &Request,
    source: &dyn SnsDiscoverySource,
    lock_stale_after_seconds: u64,
    run_locked: impl FnOnce(
        &SnsSnapshotRefreshContext<'_, Request, Collection>,
    ) -> Result<Report, SnsHostError>,
) -> Result<Report, SnsHostError>
where
    Request: SnsRefreshRequestView,
    Collection: SnsCacheCollection + Clone,
{
    validate_sns_refresh_page_size(request.page_size())?;
    let lookup_request = lookup_request_from_parts(
        request.network(),
        request.source_endpoint(),
        request.now_unix_secs(),
        request.input(),
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let paths = SnsSnapshotCachePaths::<Collection>::for_root(
        request.cache_root(),
        request.network(),
        &lookup.sns.root_canister_id,
    );
    let context_paths = paths.clone();
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.cache_path,
            refresh_lock_path: &paths.lock_path,
            network: request.network(),
            now_unix_secs: request.now_unix_secs(),
            lock_stale_after_seconds,
        },
        SnsHostError::Cache,
        |refresh_state| {
            let context = SnsSnapshotRefreshContext {
                request,
                fetch_request: lookup.fetch_request,
                list: lookup.list,
                id: lookup.id,
                sns: lookup.sns,
                paths: context_paths,
                replaced_existing_cache: refresh_state.replaced_existing_snapshot,
            };
            run_snapshot_refresh_with_attempts(
                || write_starting_sns_refresh_attempt(context.refresh_context()),
                || run_locked(&context),
                |error| write_failed_sns_refresh_attempt(context.refresh_context(), error),
            )
        },
    )
}

/// Build and atomically publish one complete SNS family snapshot and attempt.
pub(in crate::sns::report) fn publish_complete_sns_snapshot<Request, Collection, Data>(
    context: &SnsSnapshotRefreshContext<'_, Request, Collection>,
    cache_schema_version: u32,
    page_count: u32,
    row_count: usize,
    last_cursor: Option<String>,
    data: Data,
) -> Result<Option<String>, SnsHostError>
where
    Request: SnsRefreshRequestView,
    Collection: SnsCacheCollection,
    Data: Serialize,
{
    let cache = SnapshotEnvelope {
        schema_version: cache_schema_version,
        network: context.list.network.clone(),
        fetched_at: context.list.fetched_at.clone(),
        source_endpoint: context.list.source_endpoint.clone(),
        fetched_by: context.list.fetched_by.clone(),
        domain: "sns".to_string(),
        entity: context.sns.root_canister_id.clone(),
        collection: Collection::COLLECTION.to_string(),
        scope: "full".to_string(),
        metadata: SnsCacheMetadata {
            sns_wasm_canister_id: context.list.sns_wasm_canister_id.clone(),
            id: context.id,
            name: context.sns.name.clone(),
            root_canister_id: context.sns.root_canister_id.clone(),
            governance_canister_id: context.sns.governance_canister_id.clone(),
        },
        completeness: CacheCollectionCompleteness::api_exhausted(
            context.request.page_size(),
            page_count,
            row_count,
            false,
        ),
        data,
    };
    publish_snapshot_with_attempt(
        || {
            write_snapshot_json(
                &context.paths.cache_path,
                &cache,
                |path, source| SnsHostError::SerializeCache { path, source },
                SnsHostError::Cache,
            )
        },
        || {
            write_complete_sns_refresh_attempt(
                context.refresh_context(),
                SnapshotRefreshProgress::new(page_count, row_count, last_cursor),
            )
        },
    )
}
