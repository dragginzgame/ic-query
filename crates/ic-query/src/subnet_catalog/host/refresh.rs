use super::{
    CatalogSourceSelection, LiveSubnetCatalogSource, SubnetCatalogCacheRequest,
    SubnetCatalogHostError, SubnetCatalogSource, SubnetCatalogSourceFailure, SubnetCatalogSubject,
    error::{enforce_mainnet_network, subnet_cache_error},
    failure::subject_from_catalog_error,
    source::collect_subnet_catalog_detailed,
    subnet_catalog_path, subnet_catalog_refresh_lock_path,
};
use crate::{QueryProgress, QueryProgressEvent, progress::IgnoreQueryProgress};
use crate::{
    cache_file::{
        MAX_JSON_SNAPSHOT_BYTES, RefreshLockRequest, create_managed_parent_directory,
        ensure_managed_write_size, managed_file_exists, validate_output_path,
        with_refresh_lock_async, write_managed_json_pretty_atomically,
        write_managed_text_atomically, write_text_output,
    },
    runtime::block_on_current_thread,
    subnet_catalog::{
        CatalogValidationContext, DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
        MAINNET_REGISTRY_CANISTER_ID, SUBNET_CATALOG_REFRESH_REPORT_SCHEMA_VERSION,
        SubnetCatalogRefreshReport, ValidatedSubnetCatalog, catalog_to_pretty_json,
        format_utc_timestamp_secs,
    },
};
use std::{
    io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

///
/// SubnetCatalogRefreshRequest
///
/// Host cache refresh inputs for replacing or previewing a subnet catalog snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogRefreshRequest {
    /// Network and root directory identifying the managed catalog snapshot.
    pub cache: SubnetCatalogCacheRequest,
    /// Explicit single-endpoint or bounded agreement source selection.
    pub source: CatalogSourceSelection,
    /// Caller-supplied observation time in Unix seconds.
    pub now_unix_secs: u64,
    /// Age in seconds beyond which a retained refresh lock is stale.
    pub lock_stale_after_seconds: u64,
    /// Maximum accepted lead of the collection timestamp over the observation time.
    pub max_future_skew_seconds: u64,
    /// Validate a collected catalog without replacing the managed snapshot.
    pub dry_run: bool,
    /// Optional export that must not alias the managed catalog, history, or their locks.
    pub output_path: Option<PathBuf>,
}

impl SubnetCatalogRefreshRequest {
    /// Create an explicit-source refresh request with publication enabled and no export.
    #[must_use]
    pub const fn new(
        cache: SubnetCatalogCacheRequest,
        source: CatalogSourceSelection,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            cache,
            source,
            now_unix_secs,
            lock_stale_after_seconds,
            max_future_skew_seconds: DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
            dry_run: false,
            output_path: None,
        }
    }

    /// Select validation and optional export without replacing the managed catalog.
    #[must_use]
    pub const fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Export the validated catalog to a path distinct from managed snapshots and locks.
    #[must_use]
    pub fn with_output_path(mut self, output_path: impl Into<PathBuf>) -> Self {
        self.output_path = Some(output_path.into());
        self
    }

    /// Override the maximum accepted future timestamp skew.
    #[must_use]
    pub const fn with_max_future_skew_seconds(mut self, seconds: u64) -> Self {
        self.max_future_skew_seconds = seconds;
        self
    }
}

/// Collect and validate a live catalog using the explicitly selected source policy.
pub fn refresh_subnet_catalog(
    request: &SubnetCatalogRefreshRequest,
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError> {
    block_on_current_thread(refresh_subnet_catalog_async(request))?
}

/// Collect through a caller-owned source and publish only a validated complete catalog.
pub fn refresh_subnet_catalog_with_source(
    request: &SubnetCatalogRefreshRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError> {
    block_on_current_thread(refresh_subnet_catalog_with_source_async(request, source))?
}

/// Refresh a catalog on the caller's async runtime using the live mainnet source.
pub async fn refresh_subnet_catalog_async(
    request: &SubnetCatalogRefreshRequest,
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError> {
    let source = if request.dry_run {
        LiveSubnetCatalogSource::default()
    } else {
        LiveSubnetCatalogSource::default().with_history_cache(&request.cache.cache_root)
    };
    refresh_subnet_catalog_with_source_async(request, &source).await
}

/// Refresh a catalog on the caller's async runtime using a supplied source.
pub async fn refresh_subnet_catalog_with_source_async(
    request: &SubnetCatalogRefreshRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError> {
    refresh_subnet_catalog_detailed_with_source_async(request, source)
        .await
        .map_err(SubnetCatalogSourceFailure::into_source)
}

pub(super) async fn refresh_subnet_catalog_detailed_with_source_async(
    request: &SubnetCatalogRefreshRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<SubnetCatalogRefreshReport, Box<SubnetCatalogSourceFailure>> {
    refresh_subnet_catalog_detailed_with_progress_async(request, source, &mut IgnoreQueryProgress)
        .await
}

pub(super) async fn refresh_subnet_catalog_detailed_with_progress_async(
    request: &SubnetCatalogRefreshRequest,
    source: &dyn SubnetCatalogSource,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<SubnetCatalogRefreshReport, Box<SubnetCatalogSourceFailure>> {
    enforce_mainnet_network(&request.cache.network).map_err(|source| {
        SubnetCatalogSourceFailure::new(
            None,
            Some(SubnetCatalogSubject::Network(request.cache.network.clone())),
            source,
        )
    })?;
    let source_endpoints = request.source.validated_endpoints()?;
    let catalog_path = subnet_catalog_path(&request.cache.cache_root, &request.cache.network);
    let lock_path =
        subnet_catalog_refresh_lock_path(&request.cache.cache_root, &request.cache.network);
    let history_path =
        super::subnet_catalog_history_path(&request.cache.cache_root, &request.cache.network);
    let history_lock_path =
        super::subnet_catalog_history_lock_path(&request.cache.cache_root, &request.cache.network);
    if let Some(output_path) = &request.output_path {
        let mut protected = vec![
            &catalog_path as &Path,
            &lock_path,
            &history_path,
            &history_lock_path,
        ];
        if let Some(paths) = source.history_cache_paths() {
            protected.extend(<[&Path; 2]>::from(paths));
        }
        validate_output_path(output_path, &protected)
            .map_err(|error| cache_failure(error, None, output_path))?;
    }
    create_managed_parent_directory(&request.cache.cache_root, &catalog_path)
        .map_err(|error| cache_failure(error, None, &catalog_path))?;
    let known_registry_version = Arc::new(AtomicU64::new(0));
    let cache_error_version = Arc::clone(&known_registry_version);
    let cache_error_path = catalog_path.clone();
    with_refresh_lock_async(
        RefreshLockRequest {
            cache_root: &request.cache.cache_root,
            lock_path: &lock_path,
            target_path: &catalog_path,
            network: &request.cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        move |error| {
            cache_failure(
                error,
                nonzero_version(cache_error_version.load(Ordering::Relaxed)),
                &cache_error_path,
            )
        },
        || {
            refresh_subnet_catalog_under_lock(
                request,
                source,
                source_endpoints,
                &catalog_path,
                &lock_path,
                &known_registry_version,
                progress,
            )
        },
    )
    .await
}

async fn refresh_subnet_catalog_under_lock(
    request: &SubnetCatalogRefreshRequest,
    source: &dyn SubnetCatalogSource,
    source_endpoints: Vec<String>,
    catalog_path: &Path,
    lock_path: &Path,
    known_registry_version: &AtomicU64,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<SubnetCatalogRefreshReport, Box<SubnetCatalogSourceFailure>> {
    let replaced_existing_catalog = managed_file_exists(&request.cache.cache_root, catalog_path)
        .map_err(|error| cache_failure(error, None, catalog_path))?;
    progress.report(QueryProgressEvent::CacheRefresh {
        component: "subnet catalog".to_string(),
        path: catalog_path.to_path_buf(),
        source_endpoint: source_endpoints.join(", "),
    });
    let raw = collect_subnet_catalog_detailed(
        &request.cache.network,
        source_endpoints,
        &format_utc_timestamp_secs(request.now_unix_secs),
        "ic-query",
        request.now_unix_secs,
        request.max_future_skew_seconds,
        source,
    )
    .await?;
    let registry_version = raw.provenance.registry_version;
    known_registry_version.store(registry_version, Ordering::Relaxed);
    let validation = CatalogValidationContext::new(
        &request.cache.network,
        MAINNET_REGISTRY_CANISTER_ID,
        request.now_unix_secs,
        request.max_future_skew_seconds,
    );
    let catalog = ValidatedSubnetCatalog::try_from_raw(raw, &validation)
        .map_err(|source| catalog_failure(source, registry_version))?;
    if let Some(output_path) = &request.output_path {
        let catalog_json = catalog_to_pretty_json(catalog.raw())
            .map_err(|source| catalog_failure(source, registry_version))?;
        if !request.dry_run {
            ensure_managed_write_size(
                catalog_path,
                catalog_json.len() as u64,
                MAX_JSON_SNAPSHOT_BYTES,
            )
            .map_err(|error| cache_failure(error, Some(registry_version), catalog_path))?;
        }
        let history_path =
            super::subnet_catalog_history_path(&request.cache.cache_root, &request.cache.network);
        let history_lock_path = super::subnet_catalog_history_lock_path(
            &request.cache.cache_root,
            &request.cache.network,
        );
        let mut protected = vec![catalog_path, lock_path, &history_path, &history_lock_path];
        if let Some(paths) = source.history_cache_paths() {
            protected.extend(<[&Path; 2]>::from(paths));
        }
        write_text_output(output_path, &catalog_json, &protected)
            .map_err(|error| cache_failure(error, Some(registry_version), output_path))?;
        if !request.dry_run {
            write_managed_text_atomically(&request.cache.cache_root, catalog_path, &catalog_json)
                .map_err(|error| cache_failure(error, Some(registry_version), catalog_path))?;
        }
    } else if request.dry_run {
        serde_json::to_writer_pretty(io::sink(), catalog.raw())
            .map_err(|source| catalog_failure(source.into(), registry_version))?;
    } else {
        write_managed_json_pretty_atomically(
            &request.cache.cache_root,
            catalog_path,
            catalog.raw(),
            MAX_JSON_SNAPSHOT_BYTES,
            |_, source| catalog_failure(source.into(), registry_version),
            |error| cache_failure(error, Some(registry_version), catalog_path),
        )?;
    }
    Ok(SubnetCatalogRefreshReport {
        schema_version: SUBNET_CATALOG_REFRESH_REPORT_SCHEMA_VERSION,
        network: catalog.provenance().network.clone(),
        catalog_path: catalog_path.display().to_string(),
        refresh_lock_path: lock_path.display().to_string(),
        output_path: request
            .output_path
            .as_ref()
            .map(|path| path.display().to_string()),
        registry_canister_id: catalog.provenance().registry_canister_id.clone(),
        registry_version: catalog.provenance().registry_version,
        assurance: catalog.provenance().assurance,
        source_endpoints: catalog.provenance().source_endpoints.clone(),
        agreement_digest: catalog.provenance().agreement_digest.clone(),
        registry_query_call_count: catalog.provenance().registry_query_call_count,
        routing_source: catalog.provenance().routing_source,
        registry_records: catalog.provenance().registry_records.clone(),
        catalog_digest: catalog.raw().catalog_digest.clone(),
        fetched_at: catalog.provenance().fetched_at.clone(),
        fetched_by: catalog.provenance().fetched_by.clone(),
        collector_version: catalog.provenance().collector_version.clone(),
        classification_schema_version: catalog.provenance().classification_schema_version,
        classification_policy_digest: catalog.provenance().classification_policy_digest.clone(),
        resolver_schema_version: catalog.provenance().resolver_schema_version,
        resolver_backend: catalog.provenance().resolver_backend.clone(),
        dry_run: request.dry_run,
        wrote_catalog: !request.dry_run,
        replaced_existing_catalog,
        subnet_count: catalog.subnets().len(),
        routing_range_count: catalog.routing_ranges().len(),
    })
}

fn catalog_failure(
    source: crate::subnet_catalog::CatalogError,
    registry_version: u64,
) -> Box<SubnetCatalogSourceFailure> {
    let subject = subject_from_catalog_error(&source);
    SubnetCatalogSourceFailure::new(
        Some(registry_version),
        subject,
        SubnetCatalogHostError::Catalog(source),
    )
}

fn cache_failure(
    error: crate::cache_file::CacheFileError,
    registry_version: Option<u64>,
    path: &Path,
) -> Box<SubnetCatalogSourceFailure> {
    SubnetCatalogSourceFailure::new(
        registry_version,
        Some(SubnetCatalogSubject::CachePath(path.to_path_buf())),
        subnet_cache_error(error),
    )
}

const fn nonzero_version(registry_version: u64) -> Option<u64> {
    if registry_version == 0 {
        None
    } else {
        Some(registry_version)
    }
}
