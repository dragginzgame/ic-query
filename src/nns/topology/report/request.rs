use crate::{
    nns::data_center::report::{
        NnsDataCenterCacheRequest, NnsDataCenterListRequest, NnsDataCenterRefreshRequest,
    },
    nns::node::report::{
        NnsNodeCacheRequest, NnsNodeListFilters, NnsNodeListRequest, NnsNodeRefreshRequest,
    },
    nns::node_operator::report::{
        NnsNodeOperatorCacheRequest, NnsNodeOperatorListRequest, NnsNodeOperatorRefreshRequest,
    },
    nns::node_provider::report::{
        NnsNodeProviderCacheRequest, NnsNodeProviderListRequest, NnsNodeProviderRefreshRequest,
    },
    subnet_catalog::{
        DEFAULT_STALE_AFTER_SECONDS, SubnetCatalogCacheRequest, SubnetCatalogFilters,
        SubnetCatalogListRequest, SubnetCatalogRefreshRequest,
    },
};
use std::path::{Path, PathBuf};

pub(super) trait TopologyRequestParts {
    fn icp_root(&self) -> &Path;
    fn network(&self) -> &str;
    fn source_endpoint(&self) -> &str;
    fn now_unix_secs(&self) -> u64;
}

pub(super) trait TopologyRefreshParts: TopologyRequestParts {
    fn lock_stale_after_seconds(&self) -> u64;
    fn dry_run(&self) -> bool;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyReadRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

pub type NnsTopologySummaryRequest = NnsTopologyReadRequest;
pub type NnsTopologyCoverageRequest = NnsTopologyReadRequest;
pub type NnsTopologyVersionsRequest = NnsTopologyReadRequest;
pub type NnsTopologyHealthRequest = NnsTopologyReadRequest;
pub type NnsTopologyGapsRequest = NnsTopologyReadRequest;
pub type NnsTopologyCapacityRequest = NnsTopologyReadRequest;
pub type NnsTopologyRegionsRequest = NnsTopologyReadRequest;
pub type NnsTopologyProvidersRequest = NnsTopologyReadRequest;

impl TopologyRequestParts for NnsTopologyReadRequest {
    fn icp_root(&self) -> &Path {
        &self.icp_root
    }

    fn network(&self) -> &str {
        &self.network
    }

    fn source_endpoint(&self) -> &str {
        &self.source_endpoint
    }

    fn now_unix_secs(&self) -> u64 {
        self.now_unix_secs
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyRefreshRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
}

impl TopologyRequestParts for NnsTopologyRefreshRequest {
    fn icp_root(&self) -> &Path {
        &self.icp_root
    }

    fn network(&self) -> &str {
        &self.network
    }

    fn source_endpoint(&self) -> &str {
        &self.source_endpoint
    }

    fn now_unix_secs(&self) -> u64 {
        self.now_unix_secs
    }
}

impl TopologyRefreshParts for NnsTopologyRefreshRequest {
    fn lock_stale_after_seconds(&self) -> u64 {
        self.lock_stale_after_seconds
    }

    fn dry_run(&self) -> bool {
        self.dry_run
    }
}

pub(super) fn summary_request_from(
    request: &impl TopologyRequestParts,
) -> NnsTopologySummaryRequest {
    NnsTopologySummaryRequest {
        icp_root: request.icp_root().to_path_buf(),
        network: request.network().to_string(),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
    }
}

macro_rules! cache_request {
    ($name:ident, $request:path) => {
        fn $name(request: &impl TopologyRequestParts) -> $request {
            $request {
                icp_root: request.icp_root().to_path_buf(),
                network: request.network().to_string(),
            }
        }
    };
}

cache_request!(subnet_catalog_cache_request, SubnetCatalogCacheRequest);
cache_request!(node_cache_request, NnsNodeCacheRequest);
cache_request!(node_provider_cache_request, NnsNodeProviderCacheRequest);
cache_request!(node_operator_cache_request, NnsNodeOperatorCacheRequest);
cache_request!(data_center_cache_request, NnsDataCenterCacheRequest);

pub(super) fn subnet_catalog_list_request(
    request: &impl TopologyRequestParts,
) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest {
        cache: subnet_catalog_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
        filters: SubnetCatalogFilters::default(),
        show_ranges: false,
        range_limit: 1,
        range_offset: 0,
    }
}

pub(super) fn node_list_request(request: &impl TopologyRequestParts) -> NnsNodeListRequest {
    NnsNodeListRequest {
        cache: node_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        filters: NnsNodeListFilters::default(),
    }
}

macro_rules! component_list_request {
    ($name:ident, $request:path, $cache_request:ident) => {
        pub(super) fn $name(request: &impl TopologyRequestParts) -> $request {
            $request {
                cache: $cache_request(request),
                source_endpoint: request.source_endpoint().to_string(),
                now_unix_secs: request.now_unix_secs(),
            }
        }
    };
}

component_list_request!(
    node_provider_list_request,
    NnsNodeProviderListRequest,
    node_provider_cache_request
);
component_list_request!(
    node_operator_list_request,
    NnsNodeOperatorListRequest,
    node_operator_cache_request
);
component_list_request!(
    data_center_list_request,
    NnsDataCenterListRequest,
    data_center_cache_request
);

pub(super) fn subnet_catalog_refresh_request(
    request: &impl TopologyRefreshParts,
) -> SubnetCatalogRefreshRequest {
    SubnetCatalogRefreshRequest {
        cache: subnet_catalog_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        lock_stale_after_seconds: request.lock_stale_after_seconds(),
        dry_run: request.dry_run(),
        output_path: None,
    }
}

pub(super) fn node_refresh_request(request: &impl TopologyRefreshParts) -> NnsNodeRefreshRequest {
    NnsNodeRefreshRequest {
        cache: node_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        lock_stale_after_seconds: request.lock_stale_after_seconds(),
        dry_run: request.dry_run(),
        output_path: None,
    }
}

macro_rules! component_refresh_request {
    ($name:ident, $request:path, $cache_request:ident) => {
        pub(super) fn $name(request: &impl TopologyRefreshParts) -> $request {
            $request {
                cache: $cache_request(request),
                source_endpoint: request.source_endpoint().to_string(),
                now_unix_secs: request.now_unix_secs(),
                lock_stale_after_seconds: request.lock_stale_after_seconds(),
                dry_run: request.dry_run(),
                output_path: None,
            }
        }
    };
}

component_refresh_request!(
    node_provider_refresh_request,
    NnsNodeProviderRefreshRequest,
    node_provider_cache_request
);
component_refresh_request!(
    node_operator_refresh_request,
    NnsNodeOperatorRefreshRequest,
    node_operator_cache_request
);
component_refresh_request!(
    data_center_refresh_request,
    NnsDataCenterRefreshRequest,
    data_center_cache_request
);
