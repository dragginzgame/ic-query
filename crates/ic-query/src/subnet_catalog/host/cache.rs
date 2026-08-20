//! Module: subnet_catalog::host::cache
//!
//! Responsibility: load validated catalogs under explicit caller freshness policy.
//! Does not own: live Registry collection, cache paths, or report rendering.
//! Boundary: returns both validated authority evidence and observable cache disposition.

use super::{
    CatalogSourceSelection, SubnetCatalogFailureCacheDisposition, SubnetCatalogHostError,
    SubnetCatalogLoadFailure, SubnetCatalogLoadStage, SubnetCatalogRefreshRequest,
    SubnetCatalogRefreshTrigger, SubnetCatalogSource, SubnetCatalogSourceFailure,
    SubnetCatalogSubject, error::enforce_mainnet_network, failure::subject_from_catalog_error,
    refresh::refresh_subnet_catalog_detailed_with_source_async, subnet_catalog_path,
};
use crate::{
    cache_file::read_managed_text,
    nns::LiveNnsSource,
    runtime::block_on_current_thread,
    subnet_catalog::{
        CatalogAssurance, CatalogValidationContext, DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS, MAINNET_REGISTRY_CANISTER_ID, ValidatedSubnetCatalog,
        catalog_stale_status, parse_catalog_json,
    },
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

///
/// SubnetCatalogCacheRequest
///
/// Cache root and network identity used to locate a Subnet Catalog snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogCacheRequest {
    /// Caller-owned cache root.
    pub cache_root: PathBuf,
    /// Canonical network identity.
    pub network: String,
}

impl SubnetCatalogCacheRequest {
    /// Build one cache identity.
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
        }
    }
}

///
/// CatalogReadPolicy
///
/// Exact cache and network behavior authorized for one catalog load.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogReadPolicy {
    /// Load existing valid content without making a network call.
    CacheOnly,
    /// Refresh only when the cache is absent.
    RefreshMissing {
        /// Explicit live Registry source used only when content is absent.
        source: CatalogSourceSelection,
    },
    /// Refresh when the cache is absent or recoverably invalid.
    RefreshMissingOrInvalid {
        /// Explicit live Registry source used for authorized repair.
        source: CatalogSourceSelection,
    },
    /// Refresh absent, recoverably invalid, or older valid content.
    RefreshMissingInvalidOrOlderThan {
        /// Explicit live Registry source used for authorized refresh.
        source: CatalogSourceSelection,
        /// Maximum accepted age of an otherwise valid catalog.
        max_age_seconds: u64,
    },
    /// Always collect and atomically replace the complete catalog.
    ForceRefresh {
        /// Explicit live Registry source used for forced collection.
        source: CatalogSourceSelection,
    },
}

impl CatalogReadPolicy {
    pub(super) const fn source(&self) -> Option<&CatalogSourceSelection> {
        match self {
            Self::CacheOnly => None,
            Self::RefreshMissing { source }
            | Self::RefreshMissingOrInvalid { source }
            | Self::RefreshMissingInvalidOrOlderThan { source, .. }
            | Self::ForceRefresh { source } => Some(source),
        }
    }
}

///
/// SubnetCatalogLoadRequest
///
/// Caller-owned cache identity, time, and exact read policy.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogLoadRequest {
    /// Cache identity.
    pub cache: SubnetCatalogCacheRequest,
    /// Caller-supplied current Unix time.
    pub now_unix_secs: u64,
    /// Maximum accepted future timestamp skew.
    pub max_future_skew_seconds: u64,
    /// Minimum authority level accepted from cache or refresh.
    pub minimum_assurance: CatalogAssurance,
    /// Authorized cache and network behavior.
    pub policy: CatalogReadPolicy,
}

impl SubnetCatalogLoadRequest {
    /// Build a cache-only request with the default future-skew allowance.
    #[must_use]
    pub const fn cache_only(cache: SubnetCatalogCacheRequest, now_unix_secs: u64) -> Self {
        Self {
            cache,
            now_unix_secs,
            max_future_skew_seconds: DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
            minimum_assurance: CatalogAssurance::UncertifiedQuery,
            policy: CatalogReadPolicy::CacheOnly,
        }
    }

    /// Build a missing-or-invalid read-through request.
    #[must_use]
    pub const fn refresh_missing_or_invalid(
        cache: SubnetCatalogCacheRequest,
        source: CatalogSourceSelection,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            cache,
            now_unix_secs,
            max_future_skew_seconds: DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
            minimum_assurance: CatalogAssurance::UncertifiedQuery,
            policy: CatalogReadPolicy::RefreshMissingOrInvalid { source },
        }
    }

    /// Build a missing/invalid/stale read-through request.
    #[must_use]
    pub const fn refresh_missing_invalid_or_older_than(
        cache: SubnetCatalogCacheRequest,
        source: CatalogSourceSelection,
        now_unix_secs: u64,
        max_age_seconds: u64,
    ) -> Self {
        Self {
            cache,
            now_unix_secs,
            max_future_skew_seconds: DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
            minimum_assurance: CatalogAssurance::UncertifiedQuery,
            policy: CatalogReadPolicy::RefreshMissingInvalidOrOlderThan {
                source,
                max_age_seconds,
            },
        }
    }

    /// Override the maximum accepted future timestamp skew.
    #[must_use]
    pub const fn with_max_future_skew_seconds(mut self, seconds: u64) -> Self {
        self.max_future_skew_seconds = seconds;
        self
    }

    /// Require loaded evidence to meet at least this assurance level.
    #[must_use]
    pub const fn with_minimum_assurance(mut self, minimum: CatalogAssurance) -> Self {
        self.minimum_assurance = minimum;
        self
    }

    /// Replace the exact cache/network policy.
    #[must_use]
    pub fn with_policy(mut self, policy: CatalogReadPolicy) -> Self {
        self.policy = policy;
        self
    }
}

///
/// CacheDisposition
///
/// Observable result of applying one catalog read policy.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheDisposition {
    /// Existing validated cache content was used.
    CacheHit,
    /// Missing content was collected and published.
    RefreshedMissing,
    /// Recoverably invalid content was replaced.
    RefreshedInvalid,
    /// Valid but older-than-policy content was replaced.
    RefreshedStale,
    /// Caller explicitly required replacement.
    ForcedRefresh,
}

///
/// CatalogAuthorityEvidence
///
/// Compact persistable authority identity for one successful catalog load.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CatalogAuthorityEvidence {
    /// Exact Registry version represented by the validated catalog.
    pub registry_version: u64,
    /// Lowercase SHA-256 digest of the validated catalog authority payload.
    pub catalog_digest: String,
    /// Assurance established for the loaded evidence.
    pub assurance: CatalogAssurance,
    /// Canonically ordered endpoints contributing to the evidence.
    pub source_endpoints: Vec<String>,
    /// Observable cache action used to supply the catalog.
    pub cache_disposition: CacheDisposition,
}

impl CacheDisposition {
    /// Return the stable JSON and report label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CacheHit => "cache_hit",
            Self::RefreshedMissing => "refreshed_missing",
            Self::RefreshedInvalid => "refreshed_invalid",
            Self::RefreshedStale => "refreshed_stale",
            Self::ForcedRefresh => "forced_refresh",
        }
    }
}

///
/// CatalogLoadOutcome
///
/// Validated catalog, supplying path, and observable cache disposition.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogLoadOutcome {
    /// Cache path supplying the validated catalog.
    pub path: PathBuf,
    /// Privately held validated authority evidence.
    pub catalog: ValidatedSubnetCatalog,
    /// Result of applying the requested cache policy.
    pub disposition: CacheDisposition,
}

impl CatalogLoadOutcome {
    /// Return compact authority evidence suitable for embedding in a durable plan.
    #[must_use]
    pub fn authority_evidence(&self) -> CatalogAuthorityEvidence {
        let provenance = self.catalog.provenance();
        CatalogAuthorityEvidence {
            registry_version: provenance.registry_version,
            catalog_digest: self.catalog.raw().catalog_digest.clone(),
            assurance: provenance.assurance,
            source_endpoints: provenance.source_endpoints.clone(),
            cache_disposition: self.disposition,
        }
    }
}

/// Load a catalog without making a live network call.
pub fn load_cached_subnet_catalog(
    request: &SubnetCatalogLoadRequest,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    load_cached_subnet_catalog_detailed(request).map_err(SubnetCatalogLoadFailure::into_source)
}

/// Load a catalog locally while retaining typed failure provenance.
#[expect(
    clippy::result_large_err,
    reason = "the public detailed failure intentionally retains complete typed provenance"
)]
pub fn load_cached_subnet_catalog_detailed(
    request: &SubnetCatalogLoadRequest,
) -> Result<CatalogLoadOutcome, SubnetCatalogLoadFailure> {
    if request.policy != CatalogReadPolicy::CacheOnly {
        return Err(load_failure(
            request,
            SubnetCatalogLoadStage::RequestValidation,
            SubnetCatalogFailureCacheDisposition::NotExamined,
            SubnetCatalogSourceFailure::from_source(SubnetCatalogHostError::InvalidReadPolicy {
                reason: "load_cached_subnet_catalog requires CatalogReadPolicy::CacheOnly"
                    .to_string(),
            }),
        ));
    }
    load_cached_with_disposition_detailed(request, CacheDisposition::CacheHit)
        .map_err(|failure| cache_load_failure(request, true, failure))
}

/// Apply an explicit catalog read policy using the live mainnet source when authorized.
pub fn load_subnet_catalog(
    request: &SubnetCatalogLoadRequest,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    load_subnet_catalog_detailed(request).map_err(SubnetCatalogLoadFailure::into_source)
}

/// Apply an explicit catalog read policy while retaining typed failure provenance.
#[expect(
    clippy::result_large_err,
    reason = "the public detailed failure intentionally retains complete typed provenance"
)]
pub fn load_subnet_catalog_detailed(
    request: &SubnetCatalogLoadRequest,
) -> Result<CatalogLoadOutcome, SubnetCatalogLoadFailure> {
    match block_on_current_thread(load_subnet_catalog_detailed_async(request)) {
        Ok(result) => result,
        Err(source) => Err(runtime_load_failure(request, source)),
    }
}

/// Apply an explicit catalog read policy using a caller-supplied fixture or live source.
pub fn load_subnet_catalog_with_source(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    load_subnet_catalog_detailed_with_source(request, source)
        .map_err(SubnetCatalogLoadFailure::into_source)
}

/// Apply a catalog read policy with a supplied source and typed failure provenance.
#[expect(
    clippy::result_large_err,
    reason = "the public detailed failure intentionally retains complete typed provenance"
)]
pub fn load_subnet_catalog_detailed_with_source(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<CatalogLoadOutcome, SubnetCatalogLoadFailure> {
    match block_on_current_thread(load_subnet_catalog_detailed_with_source_async(
        request, source,
    )) {
        Ok(result) => result,
        Err(error) => Err(runtime_load_failure(request, error)),
    }
}

/// Apply a catalog read policy on the caller's async runtime using the live source.
pub async fn load_subnet_catalog_async(
    request: &SubnetCatalogLoadRequest,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    load_subnet_catalog_detailed_async(request)
        .await
        .map_err(SubnetCatalogLoadFailure::into_source)
}

/// Apply a catalog policy asynchronously while retaining typed failure provenance.
pub async fn load_subnet_catalog_detailed_async(
    request: &SubnetCatalogLoadRequest,
) -> Result<CatalogLoadOutcome, SubnetCatalogLoadFailure> {
    load_subnet_catalog_detailed_with_source_async(request, &LiveNnsSource).await
}

/// Apply a catalog read policy on the caller's async runtime using a supplied source.
pub async fn load_subnet_catalog_with_source_async(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    load_subnet_catalog_detailed_with_source_async(request, source)
        .await
        .map_err(SubnetCatalogLoadFailure::into_source)
}

/// Apply a catalog policy asynchronously with a supplied source and typed failures.
pub async fn load_subnet_catalog_detailed_with_source_async(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<CatalogLoadOutcome, SubnetCatalogLoadFailure> {
    enforce_mainnet_network(&request.cache.network).map_err(|source| {
        load_failure(
            request,
            SubnetCatalogLoadStage::RequestValidation,
            SubnetCatalogFailureCacheDisposition::NotExamined,
            SubnetCatalogSourceFailure::new(
                None,
                Some(SubnetCatalogSubject::Network(request.cache.network.clone())),
                source,
            ),
        )
    })?;
    match &request.policy {
        CatalogReadPolicy::CacheOnly => {
            load_cached_with_disposition_detailed(request, CacheDisposition::CacheHit)
                .map_err(|failure| cache_load_failure(request, true, failure))
        }
        CatalogReadPolicy::ForceRefresh { .. } => {
            refresh_then_load_detailed(
                request,
                source,
                CacheDisposition::ForcedRefresh,
                SubnetCatalogRefreshTrigger::Forced,
            )
            .await
        }
        policy => {
            match load_cached_with_disposition_detailed(request, CacheDisposition::CacheHit) {
                Ok(cached) => {
                    let max_age_seconds = match policy {
                        CatalogReadPolicy::RefreshMissingInvalidOrOlderThan {
                            max_age_seconds,
                            ..
                        } => Some(*max_age_seconds),
                        _ => None,
                    };
                    if max_age_seconds.is_some_and(|max_age_seconds| {
                        catalog_stale_status(
                            cached.catalog.raw(),
                            request.now_unix_secs,
                            max_age_seconds,
                        )
                        .catalog_stale
                    }) {
                        refresh_then_load_detailed(
                            request,
                            source,
                            CacheDisposition::RefreshedStale,
                            SubnetCatalogRefreshTrigger::Stale,
                        )
                        .await
                    } else {
                        Ok(cached)
                    }
                }
                Err(failure)
                    if matches!(
                        failure.source,
                        SubnetCatalogHostError::MissingCatalog { .. }
                    ) =>
                {
                    refresh_then_load_detailed(
                        request,
                        source,
                        CacheDisposition::RefreshedMissing,
                        SubnetCatalogRefreshTrigger::Missing,
                    )
                    .await
                }
                Err(failure)
                    if matches!(
                        policy,
                        CatalogReadPolicy::RefreshMissingOrInvalid { .. }
                            | CatalogReadPolicy::RefreshMissingInvalidOrOlderThan { .. }
                    ) && matches!(failure.source, SubnetCatalogHostError::Catalog(_)) =>
                {
                    refresh_then_load_detailed(
                        request,
                        source,
                        CacheDisposition::RefreshedInvalid,
                        SubnetCatalogRefreshTrigger::Rejected,
                    )
                    .await
                }
                Err(failure) => Err(cache_load_failure(request, false, failure)),
            }
        }
    }
}

#[expect(
    clippy::result_large_err,
    reason = "the internal failure carries the original host error and typed subject"
)]
fn load_cached_with_disposition_detailed(
    request: &SubnetCatalogLoadRequest,
    disposition: CacheDisposition,
) -> Result<CatalogLoadOutcome, SubnetCatalogSourceFailure> {
    enforce_mainnet_network(&request.cache.network).map_err(|source| {
        SubnetCatalogSourceFailure::new(
            None,
            Some(SubnetCatalogSubject::Network(request.cache.network.clone())),
            source,
        )
    })?;
    let path = subnet_catalog_path(&request.cache.cache_root, &request.cache.network);
    let Some(data) = read_managed_text(&request.cache.cache_root, &path).map_err(|error| {
        SubnetCatalogSourceFailure::new(
            None,
            Some(SubnetCatalogSubject::CachePath(path.clone())),
            super::error::subnet_cache_error(error),
        )
    })?
    else {
        return Err(SubnetCatalogSourceFailure::new(
            None,
            Some(SubnetCatalogSubject::CachePath(path.clone())),
            SubnetCatalogHostError::MissingCatalog { path },
        ));
    };
    let raw = parse_catalog_json(&data).map_err(|source| {
        let subject = subject_from_catalog_error(&source);
        SubnetCatalogSourceFailure::new(None, subject, SubnetCatalogHostError::Catalog(source))
    })?;
    let registry_version = raw.provenance.registry_version;
    let validation = CatalogValidationContext::new(
        &request.cache.network,
        MAINNET_REGISTRY_CANISTER_ID,
        request.now_unix_secs,
        request.max_future_skew_seconds,
    );
    let catalog = ValidatedSubnetCatalog::try_from_raw(raw, &validation).map_err(|source| {
        let subject = subject_from_catalog_error(&source);
        SubnetCatalogSourceFailure::new(
            Some(registry_version),
            subject,
            SubnetCatalogHostError::Catalog(source),
        )
    })?;
    enforce_minimum_assurance(catalog.provenance().assurance, request.minimum_assurance)
        .map_err(|source| SubnetCatalogSourceFailure::new(Some(registry_version), None, source))?;
    Ok(CatalogLoadOutcome {
        path,
        catalog,
        disposition,
    })
}

async fn refresh_then_load_detailed(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
    disposition: CacheDisposition,
    trigger: SubnetCatalogRefreshTrigger,
) -> Result<CatalogLoadOutcome, SubnetCatalogLoadFailure> {
    let source_selection = request.policy.source().ok_or_else(|| {
        load_failure(
            request,
            SubnetCatalogLoadStage::RefreshAttempted,
            SubnetCatalogFailureCacheDisposition::RefreshAttempted(trigger),
            SubnetCatalogSourceFailure::from_source(SubnetCatalogHostError::InvalidReadPolicy {
                reason: "refresh policy is missing its source selection".to_string(),
            }),
        )
    })?;
    enforce_minimum_assurance(source_selection.assurance(), request.minimum_assurance).map_err(
        |source| {
            let (stage, cache_disposition) = if trigger == SubnetCatalogRefreshTrigger::Forced {
                (
                    SubnetCatalogLoadStage::CacheBypass,
                    SubnetCatalogFailureCacheDisposition::CacheBypassed,
                )
            } else {
                (
                    SubnetCatalogLoadStage::RefreshAttempted,
                    SubnetCatalogFailureCacheDisposition::RefreshAttempted(trigger),
                )
            };
            load_failure(
                request,
                stage,
                cache_disposition,
                SubnetCatalogSourceFailure::from_source(source),
            )
        },
    )?;
    let refresh_request = SubnetCatalogRefreshRequest::new(
        request.cache.clone(),
        source_selection.clone(),
        request.now_unix_secs,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_max_future_skew_seconds(request.max_future_skew_seconds);
    let refresh = refresh_subnet_catalog_detailed_with_source_async(&refresh_request, source)
        .await
        .map_err(|failure| {
            load_failure(
                request,
                SubnetCatalogLoadStage::RefreshFailed,
                SubnetCatalogFailureCacheDisposition::RefreshFailed(trigger),
                failure,
            )
        })?;
    load_cached_with_disposition_detailed(request, disposition).map_err(|mut failure| {
        if failure.registry_version.is_none() {
            failure.registry_version = Some(refresh.registry_version);
        }
        post_refresh_load_failure(request, trigger, failure)
    })
}

fn post_refresh_load_failure(
    request: &SubnetCatalogLoadRequest,
    trigger: SubnetCatalogRefreshTrigger,
    failure: SubnetCatalogSourceFailure,
) -> SubnetCatalogLoadFailure {
    load_failure(
        request,
        SubnetCatalogLoadStage::PostRefreshCacheLoadFailed,
        SubnetCatalogFailureCacheDisposition::PostRefreshLoadFailed(trigger),
        failure,
    )
}

fn cache_load_failure(
    request: &SubnetCatalogLoadRequest,
    cache_only: bool,
    failure: SubnetCatalogSourceFailure,
) -> SubnetCatalogLoadFailure {
    let (stage, cache_disposition) = match &failure.source {
        SubnetCatalogHostError::UnsupportedNetwork { .. } => (
            SubnetCatalogLoadStage::RequestValidation,
            SubnetCatalogFailureCacheDisposition::NotExamined,
        ),
        SubnetCatalogHostError::MissingCatalog { .. } => (
            SubnetCatalogLoadStage::CacheAbsence,
            SubnetCatalogFailureCacheDisposition::CacheMissing,
        ),
        SubnetCatalogHostError::Catalog(_)
        | SubnetCatalogHostError::InsufficientAssurance { .. } => (
            SubnetCatalogLoadStage::CacheRejection,
            SubnetCatalogFailureCacheDisposition::CacheRejected,
        ),
        _ if cache_only => (
            SubnetCatalogLoadStage::CacheOnlyLoad,
            SubnetCatalogFailureCacheDisposition::CacheOnly,
        ),
        _ => (
            SubnetCatalogLoadStage::CacheLookup,
            SubnetCatalogFailureCacheDisposition::CacheReadFailed,
        ),
    };
    load_failure(request, stage, cache_disposition, failure)
}

fn runtime_load_failure(
    request: &SubnetCatalogLoadRequest,
    source: crate::runtime::RuntimeError,
) -> SubnetCatalogLoadFailure {
    load_failure(
        request,
        SubnetCatalogLoadStage::RuntimeAdapter,
        SubnetCatalogFailureCacheDisposition::NotExamined,
        SubnetCatalogSourceFailure::from_source(SubnetCatalogHostError::RuntimeAdapter(source)),
    )
}

fn load_failure(
    request: &SubnetCatalogLoadRequest,
    stage: SubnetCatalogLoadStage,
    cache_disposition: SubnetCatalogFailureCacheDisposition,
    failure: SubnetCatalogSourceFailure,
) -> SubnetCatalogLoadFailure {
    SubnetCatalogLoadFailure::from_source_failure(request, stage, cache_disposition, failure)
}

const fn enforce_minimum_assurance(
    actual: CatalogAssurance,
    required: CatalogAssurance,
) -> Result<(), SubnetCatalogHostError> {
    if actual.satisfies(required) {
        Ok(())
    } else {
        Err(SubnetCatalogHostError::InsufficientAssurance { required, actual })
    }
}

#[cfg(test)]
mod detailed_failure_tests {
    use super::*;

    #[test]
    fn post_refresh_cache_load_failure_has_a_distinct_typed_stage_and_disposition() {
        let request = SubnetCatalogLoadRequest::cache_only(
            SubnetCatalogCacheRequest::new("/tmp/ic-query-post-refresh-test", "ic"),
            123,
        );
        let source = SubnetCatalogSourceFailure::new(
            Some(9001),
            Some(SubnetCatalogSubject::CachePath(PathBuf::from(
                "/tmp/ic-query-post-refresh-test/catalog.json",
            ))),
            SubnetCatalogHostError::MissingCatalog {
                path: PathBuf::from("/tmp/ic-query-post-refresh-test/catalog.json"),
            },
        );

        let failure =
            post_refresh_load_failure(&request, SubnetCatalogRefreshTrigger::Missing, source);

        assert_eq!(
            failure.stage,
            SubnetCatalogLoadStage::PostRefreshCacheLoadFailed
        );
        assert_eq!(
            failure.cache_disposition,
            SubnetCatalogFailureCacheDisposition::PostRefreshLoadFailed(
                SubnetCatalogRefreshTrigger::Missing
            )
        );
        assert_eq!(failure.registry_version, Some(9001));
    }
}
