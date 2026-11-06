//! Module: sns::report::neurons_cache::refresh
//!
//! Responsibility: refresh complete SNS neuron snapshots under one lock and attempt lifecycle.
//! Does not own: paging admission, row validation, report DTOs, or text rendering.
//! Boundary: collect and publish only complete neuron snapshots with retained failure evidence.

use crate::cache_file::write_collection_snapshot;
use crate::report::CollectionSourceRequest;
use crate::{
    HostCacheError, QueryProgress,
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
            write_complete_sns_refresh_attempt, write_failed_sns_refresh_attempt,
            write_starting_sns_refresh_attempt,
        },
        cache_paths::sns_snapshot_cache_paths,
        cache_storage::SnsCacheMetadata,
        live::LiveSnsSource,
        neurons_cache::{
            collection::fetch_complete_sns_neurons,
            model::{CompleteSnsNeurons, SnsNeuronsCacheRows},
        },
        source::SnsNeuronsSource,
        target::{resolve_neuron_target, validate_sns_refresh_page_size},
    },
};

///
/// SnsRefreshContext
///
/// Resolved, locked context shared by one SNS complete-cache refresh.
///

#[derive(Clone, Copy)]
pub(in crate::sns::report) struct SnsRefreshContext<'a> {
    /// Request that selected the Governance target and cache root.
    pub(in crate::sns::report) request: &'a SnsSnapshotRefreshRequest,
    /// Canonical source request shared by lookup and collection calls.
    pub(in crate::sns::report) fetch_request: &'a CollectionSourceRequest,
    /// Canonical Governance principal.
    pub(in crate::sns::report) governance_canister_id: &'a str,
    /// Complete-cache, lock, and attempt paths.
    pub(in crate::sns::report) paths: &'a SnapshotJsonPaths,
    /// Whether publication replaces an existing complete cache.
    pub(in crate::sns::report) replaced_existing_cache: bool,
}

/// Refresh a complete SNS neuron snapshot using the live SNS source.
pub async fn refresh_sns_neurons_cache(
    request: &SnsSnapshotRefreshRequest,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source_and_progress(
        request,
        &LiveSnsSource::default(),
        &mut crate::progress::IgnoreQueryProgress,
    )
    .await
}

/// Refresh with a caller-owned source and progress sink.
pub async fn refresh_sns_neurons_cache_with_source_and_progress(
    request: &SnsSnapshotRefreshRequest,
    source: &dyn SnsNeuronsSource,
    progress: &mut dyn QueryProgress,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    validate_sns_refresh_page_size(request.page_size)?;
    let lookup = resolve_neuron_target(&request.target)?;
    let paths = sns_snapshot_cache_paths(
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
            let context = SnsRefreshContext {
                request,
                fetch_request: &lookup.fetch_request,
                governance_canister_id: &lookup.governance_canister_id,
                paths: &paths,
                replaced_existing_cache: replaced_existing_snapshot,
            };
            run_snapshot_refresh_with_attempts(
                || write_starting_sns_refresh_attempt(context),
                async || {
                    let complete = fetch_complete_sns_neurons(context, source, progress).await?;
                    publish_complete_sns_snapshot(context, complete)
                },
                |error| write_failed_sns_refresh_attempt(context, error),
            )
            .await
        },
    )
    .await
}

/// Publish the complete neuron snapshot, then finalize its attempt sidecar.
fn publish_complete_sns_snapshot(
    context: SnsRefreshContext<'_>,
    complete: CompleteSnsNeurons,
) -> Result<SnsSnapshotRefreshReport, SnsHostError> {
    let page_count = complete.page_count;
    let row_count = complete.rows.len();
    let cache = SnapshotEnvelope {
        schema_version: super::SNS_NEURONS_CACHE_SCHEMA_VERSION,
        network: context.fetch_request.network.clone(),
        fetched_at: context.fetch_request.fetched_at.clone(),
        source_endpoint: context.fetch_request.endpoint.clone(),
        fetched_by: context.fetch_request.fetched_by.clone(),
        domain: "sns-neuron".to_string(),
        entity: context.governance_canister_id.to_string(),
        collection: "neurons".to_string(),
        scope: "full".to_string(),
        metadata: SnsCacheMetadata {
            governance_canister_id: context.governance_canister_id.to_string(),
        },
        completeness: CacheCollectionCompleteness::api_exhausted(
            context.request.page_size,
            page_count,
            row_count,
            false,
        ),
        data: SnsNeuronsCacheRows {
            neurons: complete.rows,
        },
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
                context,
                SnapshotRefreshProgress::new(page_count, row_count, complete.last_cursor),
            )
        },
    )?;
    let (report_context, _) = SnsReportContext::from_snapshot(cache);
    Ok(SnsSnapshotRefreshReport {
        schema_version: 1,
        context: report_context,
        collection: "neurons".to_string(),
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
