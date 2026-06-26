use std::path::PathBuf;

///
/// NnsNodeCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsNodeCacheRequest {
    pub icp_root: PathBuf,
    pub network: String,
}

impl NnsNodeCacheRequest {
    #[must_use]
    pub fn new(icp_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            icp_root: icp_root.into(),
            network: network.into(),
        }
    }
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

impl NnsNodeListRequest {
    #[must_use]
    pub fn new(
        cache: NnsNodeCacheRequest,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            cache,
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            filters: NnsNodeListFilters::default(),
        }
    }

    #[must_use]
    pub fn with_filters(mut self, filters: NnsNodeListFilters) -> Self {
        self.filters = filters;
        self
    }

    #[must_use]
    pub fn with_subnet(mut self, subnet: impl Into<String>) -> Self {
        self.filters.subnet = Some(subnet.into());
        self
    }

    #[must_use]
    pub fn with_subnet_kind(mut self, subnet_kind: impl Into<String>) -> Self {
        self.filters.subnet_kind = Some(subnet_kind.into());
        self
    }

    #[must_use]
    pub fn with_data_center(mut self, data_center: impl Into<String>) -> Self {
        self.filters.data_center = Some(data_center.into());
        self
    }

    #[must_use]
    pub fn with_node_provider(mut self, node_provider: impl Into<String>) -> Self {
        self.filters.node_provider = Some(node_provider.into());
        self
    }

    #[must_use]
    pub fn with_node_operator(mut self, node_operator: impl Into<String>) -> Self {
        self.filters.node_operator = Some(node_operator.into());
        self
    }
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

impl NnsNodeInfoRequest {
    #[must_use]
    pub fn new(
        cache: NnsNodeCacheRequest,
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
impl NnsNodeRefreshRequest {
    #[must_use]
    pub fn new(
        cache: NnsNodeCacheRequest,
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
