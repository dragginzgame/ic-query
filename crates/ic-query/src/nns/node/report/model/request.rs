use std::path::PathBuf;

///
/// NnsNodeCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

///
/// NnsNodeListRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeListRequest {
    pub cache: NnsNodeCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub filters: NnsNodeListFilters,
}

///
/// NnsNodeInfoRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeInfoRequest {
    pub cache: NnsNodeCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub now_unix_secs: u64,
}

///
/// NnsNodeRefreshRequest
///
#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeRefreshRequest {
    pub cache: NnsNodeCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
    pub output_path: Option<PathBuf>,
}

#[cfg(feature = "host")]
impl_nns_leaf_cache_and_refresh_requests!(NnsNodeCacheRequest, NnsNodeRefreshRequest);

///
/// NnsNodeListFilters
///
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NnsNodeListFilters {
    pub subnet: Option<String>,
    pub subnet_kind: Option<String>,
    pub data_center: Option<String>,
    pub node_provider: Option<String>,
    pub node_operator: Option<String>,
}

impl NnsNodeListFilters {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.subnet.is_none()
            && self.subnet_kind.is_none()
            && self.data_center.is_none()
            && self.node_provider.is_none()
            && self.node_operator.is_none()
    }
}
