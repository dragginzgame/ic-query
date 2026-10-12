#[cfg(feature = "nns-host")]
use super::{
    MainnetDataCenterList, MainnetNodeList, MainnetNodeOperatorList, MainnetNodeProviderList,
    source::{
        fetch_mainnet_data_center_list_async, fetch_mainnet_node_list_async,
        fetch_mainnet_node_operator_list_async, fetch_mainnet_node_provider_list_async,
    },
};
use super::{
    MainnetRegistryFetchRequest, RegistryFetchError,
    source::fetch_mainnet_subnet_catalog_async as fetch_mainnet_subnet_catalog_from_source_async,
};
#[cfg(feature = "certified-subnet-catalog-host")]
use super::{
    MainnetRegistryVersion,
    source::{
        fetch_mainnet_certified_registry_delta_batch_async as fetch_mainnet_certified_registry_delta_batch_from_source_async,
        fetch_mainnet_registry_version_async,
    },
};
#[cfg(feature = "nns-topology-host")]
use super::{MainnetSubnetTopology, source::fetch_mainnet_subnet_topology_async};

use crate::subnet_catalog::RawSubnetCatalog;

#[cfg(feature = "certified-subnet-catalog-host")]
use super::CertifiedRegistryDeltaBatch;

/// Fetch one exact-version mainnet Subnet Catalog on the caller's async runtime.
pub async fn fetch_mainnet_subnet_catalog_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, RegistryFetchError> {
    fetch_mainnet_subnet_catalog_from_source_async(request).await
}

/// Fetch one authenticated, bounded Registry delta batch on the caller's async runtime.
#[cfg(feature = "certified-subnet-catalog-host")]
pub async fn fetch_mainnet_certified_registry_delta_batch_async(
    request: &MainnetRegistryFetchRequest,
    requested_version: u64,
) -> Result<CertifiedRegistryDeltaBatch, RegistryFetchError> {
    fetch_mainnet_certified_registry_delta_batch_from_source_async(request, requested_version).await
}

#[cfg(feature = "nns-topology-host")]
pub async fn fetch_mainnet_subnet_topology(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetSubnetTopology, RegistryFetchError> {
    fetch_mainnet_subnet_topology_async(request).await
}

#[cfg(feature = "certified-subnet-catalog-host")]
pub async fn fetch_mainnet_registry_version(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetRegistryVersion, RegistryFetchError> {
    fetch_mainnet_registry_version_async(request).await
}

#[cfg(feature = "nns-host")]
pub async fn fetch_mainnet_node_provider_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeProviderList, RegistryFetchError> {
    fetch_mainnet_node_provider_list_async(request).await
}

#[cfg(feature = "nns-host")]
pub async fn fetch_mainnet_node_operator_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeOperatorList, RegistryFetchError> {
    fetch_mainnet_node_operator_list_async(request).await
}

#[cfg(feature = "nns-host")]
pub async fn fetch_mainnet_node_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeList, RegistryFetchError> {
    fetch_mainnet_node_list_async(request).await
}

#[cfg(feature = "nns-host")]
pub async fn fetch_mainnet_data_center_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetDataCenterList, RegistryFetchError> {
    fetch_mainnet_data_center_list_async(request).await
}
