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

///
/// NnsTopologySummaryRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologySummaryRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsTopologyCoverageRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyCoverageRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsTopologyVersionsRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyVersionsRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsTopologyHealthRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyHealthRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsTopologyGapsRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyGapsRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsTopologyCapacityRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyCapacityRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsTopologyRegionsRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyRegionsRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsTopologyProvidersRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyProvidersRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

///
/// NnsTopologyRefreshRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyRefreshRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
}

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

macro_rules! impl_topology_request_parts {
    ($($request:ty),+ $(,)?) => {
        $(
            impl TopologyRequestParts for $request {
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
        )+
    };
}

impl_topology_request_parts!(
    NnsTopologySummaryRequest,
    NnsTopologyCoverageRequest,
    NnsTopologyVersionsRequest,
    NnsTopologyHealthRequest,
    NnsTopologyGapsRequest,
    NnsTopologyCapacityRequest,
    NnsTopologyRegionsRequest,
    NnsTopologyProvidersRequest,
    NnsTopologyRefreshRequest,
);

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

fn subnet_catalog_cache_request(request: &impl TopologyRequestParts) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest {
        icp_root: request.icp_root().to_path_buf(),
        network: request.network().to_string(),
    }
}

fn node_cache_request(request: &impl TopologyRequestParts) -> NnsNodeCacheRequest {
    NnsNodeCacheRequest {
        icp_root: request.icp_root().to_path_buf(),
        network: request.network().to_string(),
    }
}

fn node_provider_cache_request(request: &impl TopologyRequestParts) -> NnsNodeProviderCacheRequest {
    NnsNodeProviderCacheRequest {
        icp_root: request.icp_root().to_path_buf(),
        network: request.network().to_string(),
    }
}

fn node_operator_cache_request(request: &impl TopologyRequestParts) -> NnsNodeOperatorCacheRequest {
    NnsNodeOperatorCacheRequest {
        icp_root: request.icp_root().to_path_buf(),
        network: request.network().to_string(),
    }
}

fn data_center_cache_request(request: &impl TopologyRequestParts) -> NnsDataCenterCacheRequest {
    NnsDataCenterCacheRequest {
        icp_root: request.icp_root().to_path_buf(),
        network: request.network().to_string(),
    }
}

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

pub(super) fn node_provider_list_request(
    request: &impl TopologyRequestParts,
) -> NnsNodeProviderListRequest {
    NnsNodeProviderListRequest {
        cache: node_provider_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
    }
}

pub(super) fn node_operator_list_request(
    request: &impl TopologyRequestParts,
) -> NnsNodeOperatorListRequest {
    NnsNodeOperatorListRequest {
        cache: node_operator_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
    }
}

pub(super) fn data_center_list_request(
    request: &impl TopologyRequestParts,
) -> NnsDataCenterListRequest {
    NnsDataCenterListRequest {
        cache: data_center_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
    }
}

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

pub(super) fn node_provider_refresh_request(
    request: &impl TopologyRefreshParts,
) -> NnsNodeProviderRefreshRequest {
    NnsNodeProviderRefreshRequest {
        cache: node_provider_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        lock_stale_after_seconds: request.lock_stale_after_seconds(),
        dry_run: request.dry_run(),
        output_path: None,
    }
}

pub(super) fn node_operator_refresh_request(
    request: &impl TopologyRefreshParts,
) -> NnsNodeOperatorRefreshRequest {
    NnsNodeOperatorRefreshRequest {
        cache: node_operator_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        lock_stale_after_seconds: request.lock_stale_after_seconds(),
        dry_run: request.dry_run(),
        output_path: None,
    }
}

pub(super) fn data_center_refresh_request(
    request: &impl TopologyRefreshParts,
) -> NnsDataCenterRefreshRequest {
    NnsDataCenterRefreshRequest {
        cache: data_center_cache_request(request),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
        lock_stale_after_seconds: request.lock_stale_after_seconds(),
        dry_run: request.dry_run(),
        output_path: None,
    }
}
