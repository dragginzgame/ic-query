use std::path::PathBuf;

///
/// NnsDataCenterCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

///
/// NnsDataCenterListRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterListRequest {
    pub cache: NnsDataCenterCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsDataCenterInfoRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsDataCenterInfoRequest {
    pub cache: NnsDataCenterCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

///
/// NnsDataCenterRefreshRequest
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
impl_nns_leaf_cache_and_refresh_requests!(NnsDataCenterCacheRequest, NnsDataCenterRefreshRequest);
