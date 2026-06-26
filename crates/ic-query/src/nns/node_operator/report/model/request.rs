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
impl_nns_leaf_cache_and_refresh_requests!(
    NnsNodeOperatorCacheRequest,
    NnsNodeOperatorRefreshRequest
);
