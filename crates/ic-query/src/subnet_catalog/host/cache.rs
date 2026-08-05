//! Module: subnet_catalog::host::cache
//!
//! Responsibility: load validated catalogs under explicit caller freshness policy.
//! Does not own: live Registry collection, cache paths, or report rendering.
//! Boundary: returns both validated authority evidence and observable cache disposition.

use super::{
    SubnetCatalogHostError, SubnetCatalogRefreshRequest, SubnetCatalogSource,
    error::enforce_mainnet_network, refresh_subnet_catalog_with_source, subnet_catalog_path,
};
use crate::{
    cache_file::HostCacheError,
    nns::LiveNnsSource,
    subnet_catalog::{
        CatalogValidationContext, DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS, MAINNET_REGISTRY_CANISTER_ID, ValidatedSubnetCatalog,
        catalog_stale_status, parse_catalog_json,
    },
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

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
        /// Explicit live Registry endpoint used only when content is absent.
        source_endpoint: String,
    },
    /// Refresh when the cache is absent or recoverably invalid.
    RefreshMissingOrInvalid {
        /// Explicit live Registry endpoint used for authorized repair.
        source_endpoint: String,
    },
    /// Refresh absent, recoverably invalid, or older valid content.
    RefreshMissingInvalidOrOlderThan {
        /// Explicit live Registry endpoint used for authorized refresh.
        source_endpoint: String,
        /// Maximum accepted age of an otherwise valid catalog.
        max_age_seconds: u64,
    },
    /// Always collect and atomically replace the complete catalog.
    ForceRefresh {
        /// Explicit live Registry endpoint used for forced collection.
        source_endpoint: String,
    },
}

impl CatalogReadPolicy {
    fn source_endpoint(&self) -> Option<&str> {
        match self {
            Self::CacheOnly => None,
            Self::RefreshMissing { source_endpoint }
            | Self::RefreshMissingOrInvalid { source_endpoint }
            | Self::RefreshMissingInvalidOrOlderThan {
                source_endpoint, ..
            }
            | Self::ForceRefresh { source_endpoint } => Some(source_endpoint),
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
            policy: CatalogReadPolicy::CacheOnly,
        }
    }

    /// Build a missing-or-invalid read-through request.
    #[must_use]
    pub fn refresh_missing_or_invalid(
        cache: SubnetCatalogCacheRequest,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            cache,
            now_unix_secs,
            max_future_skew_seconds: DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
            policy: CatalogReadPolicy::RefreshMissingOrInvalid {
                source_endpoint: source_endpoint.into(),
            },
        }
    }

    /// Override the maximum accepted future timestamp skew.
    #[must_use]
    pub const fn with_max_future_skew_seconds(mut self, seconds: u64) -> Self {
        self.max_future_skew_seconds = seconds;
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
    load_subnet_catalog_with_source(request, &LiveNnsSource)
}

/// Apply an explicit catalog read policy using a caller-supplied fixture or live source.
pub fn load_subnet_catalog_with_source(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    match &request.policy {
        CatalogReadPolicy::CacheOnly => {
            load_cached_with_disposition(request, CacheDisposition::CacheHit)
        }
        CatalogReadPolicy::ForceRefresh { .. } => {
            refresh_then_load(request, source, CacheDisposition::ForcedRefresh)
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
                    refresh_then_load(request, source, CacheDisposition::RefreshedStale)
                } else {
                    Ok(cached)
                }
            }
            Err(SubnetCatalogHostError::MissingCatalog { .. }) => {
                refresh_then_load(request, source, CacheDisposition::RefreshedMissing)
            }
            Err(SubnetCatalogHostError::Catalog(_))
                if matches!(
                    policy,
                    CatalogReadPolicy::RefreshMissingOrInvalid { .. }
                        | CatalogReadPolicy::RefreshMissingInvalidOrOlderThan { .. }
                ) =>
            {
                refresh_then_load(request, source, CacheDisposition::RefreshedInvalid)
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
    if !path.is_file() {
        return Err(SubnetCatalogHostError::MissingCatalog { path });
    }
    let data = fs::read_to_string(&path)
        .map_err(|source| HostCacheError::read_cache("subnet catalog", path.clone(), source))?;
    let raw = parse_catalog_json(&data)?;
    let validation = CatalogValidationContext::new(
        &request.cache.network,
        MAINNET_REGISTRY_CANISTER_ID,
        request.now_unix_secs,
        request.max_future_skew_seconds,
    );
    let catalog = ValidatedSubnetCatalog::try_from_raw(raw, &validation)?;
    Ok(CatalogLoadOutcome {
        path,
        catalog,
        disposition,
    })
}

fn refresh_then_load(
    request: &SubnetCatalogLoadRequest,
    source: &dyn SubnetCatalogSource,
    disposition: CacheDisposition,
) -> Result<CatalogLoadOutcome, SubnetCatalogHostError> {
    let source_endpoint = request.policy.source_endpoint().ok_or_else(|| {
        SubnetCatalogHostError::InvalidReadPolicy {
            reason: "refresh policy is missing its source endpoint".to_string(),
        }
    })?;
    let refresh_request = SubnetCatalogRefreshRequest::new(
        request.cache.clone(),
        source_endpoint,
        request.now_unix_secs,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_max_future_skew_seconds(request.max_future_skew_seconds);
    refresh_subnet_catalog_with_source(&refresh_request, source)?;
    load_cached_with_disposition(request, disposition)
}
