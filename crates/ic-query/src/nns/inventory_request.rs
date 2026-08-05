//! Module: nns::inventory_request
//!
//! Responsibility: define shared request contracts for Registry-derived NNS inventory reports.
//! Does not own: family-specific filters, source calls, cache IO, or report projection.
//! Boundary: keeps identical cache, list, info, and refresh provenance from drifting by family.

use std::path::PathBuf;

///
/// NnsInventoryCacheRequest
///
/// Shared cache identity for Registry-derived NNS inventory reports.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsInventoryCacheRequest {
    pub cache_root: PathBuf,
    pub network: String,
}

impl NnsInventoryCacheRequest {
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
        }
    }
}

///
/// NnsInventoryListRequest
///
/// Shared request for a complete Registry-derived NNS inventory report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsInventoryListRequest {
    pub cache: NnsInventoryCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

impl NnsInventoryListRequest {
    #[must_use]
    pub fn new(
        cache: NnsInventoryCacheRequest,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            cache,
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
        }
    }
}

///
/// NnsInventoryInfoRequest
///
/// Shared request for one Registry-derived NNS inventory row selected by id or prefix.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsInventoryInfoRequest {
    pub cache: NnsInventoryCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

impl NnsInventoryInfoRequest {
    #[must_use]
    pub fn new(
        cache: NnsInventoryCacheRequest,
        source_endpoint: impl Into<String>,
        input: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            cache,
            source_endpoint: source_endpoint.into(),
            input: input.into(),
            now_unix_secs,
        }
    }
}

///
/// NnsInventoryRefreshRequest
///
/// Shared host request for refreshing one Registry-derived NNS inventory cache.
///

#[cfg(feature = "nns-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsInventoryRefreshRequest {
    pub cache: NnsInventoryCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

#[cfg(feature = "nns-host")]
impl NnsInventoryRefreshRequest {
    #[must_use]
    pub fn new(
        cache: NnsInventoryCacheRequest,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            cache,
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            lock_stale_after_seconds,
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
}

#[cfg(feature = "nns-host")]
impl_nns_leaf_cache_and_refresh_requests!(NnsInventoryCacheRequest, NnsInventoryRefreshRequest);
