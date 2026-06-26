use std::path::PathBuf;

///
/// NnsNodeOperatorCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeOperatorCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

impl NnsNodeOperatorCacheRequest {
    #[must_use]
    pub fn new(icp_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            icp_root: icp_root.into(),
            network: network.into(),
        }
    }
}

///
/// NnsNodeOperatorListRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeOperatorListRequest {
    pub cache: NnsNodeOperatorCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

impl NnsNodeOperatorListRequest {
    #[must_use]
    pub fn new(
        cache: NnsNodeOperatorCacheRequest,
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
/// NnsNodeOperatorInfoRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeOperatorInfoRequest {
    pub cache: NnsNodeOperatorCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

impl NnsNodeOperatorInfoRequest {
    #[must_use]
    pub fn new(
        cache: NnsNodeOperatorCacheRequest,
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
/// NnsNodeOperatorRefreshRequest
///
#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeOperatorRefreshRequest {
    pub cache: NnsNodeOperatorCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

#[cfg(feature = "host")]
impl NnsNodeOperatorRefreshRequest {
    #[must_use]
    pub fn new(
        cache: NnsNodeOperatorCacheRequest,
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
impl_nns_leaf_cache_and_refresh_requests!(
    NnsNodeOperatorCacheRequest,
    NnsNodeOperatorRefreshRequest
);
