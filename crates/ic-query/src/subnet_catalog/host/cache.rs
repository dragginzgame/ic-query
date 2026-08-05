//! Module: subnet_catalog::host::cache
//!
//! Responsibility: load validated catalogs under explicit caller freshness policy.
//! Does not own: live Registry collection, cache paths, or report rendering.
//! Boundary: returns both validated authority evidence and observable cache disposition.

use super::{
    CatalogSourceSelection, SubnetCatalogHostError, SubnetCatalogRefreshRequest,
    SubnetCatalogSource, error::enforce_mainnet_network, refresh_subnet_catalog_with_source_async,
    subnet_catalog_path,
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
    const fn source(&self) -> Option<&CatalogSourceSelection> {
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
    if request.policy != CatalogReadPolicy::CacheOnly {
        return Err(SubnetCatalogHostError::InvalidReadPolicy {
            reason: "load_cached_subnet_catalog requires CatalogReadPolicy::CacheOnly".to_string(),
        });
    }
    load_cached_with_disposition(request, CacheDisposition::CacheHit)
}

/// Apply an explicit catalog read policy using the live mainnet source when authorized.
pub fn load_subnet_catalog(
    request: &SubnetCatalogLoadRequest,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    block_on_current_thread(load_subnet_catalog_async(request))?
}

/// Apply an explicit catalog read policy using a caller-supplied fixture or live source.
pub fn load_subnet_catalog_with_source(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    block_on_current_thread(load_subnet_catalog_with_source_async(request, source))?
}

/// Apply a catalog read policy on the caller's async runtime using the live source.
pub async fn load_subnet_catalog_async(
    request: &SubnetCatalogLoadRequest,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    load_subnet_catalog_with_source_async(request, &LiveNnsSource).await
}

/// Apply a catalog read policy on the caller's async runtime using a supplied source.
pub async fn load_subnet_catalog_with_source_async(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    match &request.policy {
        CatalogReadPolicy::CacheOnly => {
            load_cached_with_disposition(request, CacheDisposition::CacheHit)
        }
        CatalogReadPolicy::ForceRefresh { .. } => {
            refresh_then_load(request, source, CacheDisposition::ForcedRefresh).await
        }
        policy => match load_cached_with_disposition(request, CacheDisposition::CacheHit) {
            Ok(cached) => {
                let max_age_seconds = match policy {
                    CatalogReadPolicy::RefreshMissingInvalidOrOlderThan {
                        max_age_seconds, ..
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
                    refresh_then_load(request, source, CacheDisposition::RefreshedStale).await
                } else {
                    Ok(cached)
                }
            }
            Err(SubnetCatalogHostError::MissingCatalog { .. }) => {
                refresh_then_load(request, source, CacheDisposition::RefreshedMissing).await
            }
            Err(SubnetCatalogHostError::Catalog(_))
                if matches!(
                    policy,
                    CatalogReadPolicy::RefreshMissingOrInvalid { .. }
                        | CatalogReadPolicy::RefreshMissingInvalidOrOlderThan { .. }
                ) =>
            {
                refresh_then_load(request, source, CacheDisposition::RefreshedInvalid).await
            }
            Err(error) => Err(error),
        },
    }
}

fn load_cached_with_disposition(
    request: &SubnetCatalogLoadRequest,
    disposition: CacheDisposition,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let path = subnet_catalog_path(&request.cache.cache_root, &request.cache.network);
    let Some(data) = read_managed_text(&request.cache.cache_root, &path)
        .map_err(super::error::subnet_cache_error)?
    else {
        return Err(SubnetCatalogHostError::MissingCatalog { path });
    };
    let raw = parse_catalog_json(&data)?;
    let validation = CatalogValidationContext::new(
        &request.cache.network,
        MAINNET_REGISTRY_CANISTER_ID,
        request.now_unix_secs,
        request.max_future_skew_seconds,
    );
    let catalog = ValidatedSubnetCatalog::try_from_raw(raw, &validation)?;
    enforce_minimum_assurance(catalog.provenance().assurance, request.minimum_assurance)?;
    Ok(CatalogLoadOutcome {
        path,
        catalog,
        disposition,
    })
}

async fn refresh_then_load(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
    disposition: CacheDisposition,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    let source_selection =
        request
            .policy
            .source()
            .ok_or_else(|| SubnetCatalogHostError::InvalidReadPolicy {
                reason: "refresh policy is missing its source selection".to_string(),
            })?;
    enforce_minimum_assurance(source_selection.assurance(), request.minimum_assurance)?;
    let refresh_request = SubnetCatalogRefreshRequest::new(
        request.cache.clone(),
        source_selection.clone(),
        request.now_unix_secs,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_max_future_skew_seconds(request.max_future_skew_seconds);
    refresh_subnet_catalog_with_source_async(&refresh_request, source).await?;
    load_cached_with_disposition(request, disposition)
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
