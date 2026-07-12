use std::path::PathBuf;

///
/// NnsDataCenterCacheRequest
///
/// Cache identity used by NNS data center report reads.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

impl NnsDataCenterCacheRequest {
    #[must_use]
    pub fn new(icp_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            icp_root: icp_root.into(),
            network: network.into(),
        }
    }
}

///
/// NnsDataCenterListRequest
///
/// Request for the complete NNS data center inventory report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterListRequest {
    pub cache: NnsDataCenterCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

impl NnsDataCenterListRequest {
    #[must_use]
    pub fn new(
        cache: NnsDataCenterCacheRequest,
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
/// NnsDataCenterInfoRequest
///
/// Request for one NNS data center selected by identifier or prefix.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterInfoRequest {
    pub cache: NnsDataCenterCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

impl NnsDataCenterInfoRequest {
    #[must_use]
    pub fn new(
        cache: NnsDataCenterCacheRequest,
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
/// NnsDataCenterRefreshRequest
///
/// Host request for refreshing the cached NNS data center inventory.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterRefreshRequest {
    pub cache: NnsDataCenterCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

#[cfg(feature = "host")]
impl NnsDataCenterRefreshRequest {
    #[must_use]
    pub fn new(
        cache: NnsDataCenterCacheRequest,
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

#[cfg(feature = "host")]
impl_nns_leaf_cache_and_refresh_requests!(NnsDataCenterCacheRequest, NnsDataCenterRefreshRequest);
