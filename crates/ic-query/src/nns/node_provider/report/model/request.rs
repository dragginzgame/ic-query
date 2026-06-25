use std::path::PathBuf;

///
/// NnsNodeProviderCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

///
/// NnsNodeProviderListRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderListRequest {
    pub cache: NnsNodeProviderCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsNodeProviderInfoRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderInfoRequest {
    pub cache: NnsNodeProviderCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

///
/// NnsNodeProviderRefreshRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeProviderRefreshRequest {
    pub cache: NnsNodeProviderCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}
