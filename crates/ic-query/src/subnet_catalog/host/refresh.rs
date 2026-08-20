use super::{
    CatalogSourceSelection, SubnetCatalogCacheRequest, SubnetCatalogHostError, SubnetCatalogSource,
    SubnetCatalogSourceFailure, SubnetCatalogSubject,
    error::{enforce_mainnet_network, subnet_cache_error},
    failure::subject_from_catalog_error,
    source::collect_subnet_catalog_detailed,
    subnet_catalog_path, subnet_catalog_refresh_lock_path,
};
use crate::{
    cache_file::{
        RefreshLockRequest, create_managed_parent_directory, managed_file_exists,
        with_refresh_lock_async, write_managed_text_atomically, write_text_output,
    },
    nns::LiveNnsSource,
    runtime::block_on_current_thread,
    subnet_catalog::{
        CatalogValidationContext, DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
        MAINNET_REGISTRY_CANISTER_ID, SUBNET_CATALOG_REFRESH_REPORT_SCHEMA_VERSION,
        SubnetCatalogRefreshReport, ValidatedSubnetCatalog, catalog_to_pretty_json,
        format_utc_timestamp_secs,
    },
};
use std::{
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
    pub cache: SubnetCatalogCacheRequest,
    /// Explicit single-endpoint or bounded agreement source selection.
    pub source: CatalogSourceSelection,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub max_future_skew_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

impl SubnetCatalogRefreshRequest {
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

    #[must_use]
    pub const fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

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

pub fn refresh_subnet_catalog(
    request: &SubnetCatalogRefreshRequest,
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError> {
    block_on_current_thread(refresh_subnet_catalog_async(request))?
}

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
    refresh_subnet_catalog_with_source_async(request, &LiveNnsSource).await
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
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogSourceFailure> {
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
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogSourceFailure> {
    let replaced_existing_catalog = managed_file_exists(&request.cache.cache_root, catalog_path)
        .map_err(|error| cache_failure(error, None, catalog_path))?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let raw = collect_subnet_catalog_detailed(
        &request.cache.network,
        source_endpoints,
        &fetched_at,
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
    let catalog_json = catalog_to_pretty_json(catalog.raw())
        .map_err(|source| catalog_failure(source, registry_version))?;
    if let Some(output_path) = &request.output_path {
        write_text_output(output_path, &catalog_json)
            .map_err(|error| cache_failure(error, Some(registry_version), output_path))?;
    }
    if !request.dry_run {
        write_managed_text_atomically(&request.cache.cache_root, catalog_path, &catalog_json)
            .map_err(|error| cache_failure(error, Some(registry_version), catalog_path))?;
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
) -> SubnetCatalogSourceFailure {
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
) -> SubnetCatalogSourceFailure {
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
