#[cfg(feature = "host")]
use super::{
    MainnetDataCenterList, MainnetNodeList, MainnetNodeOperatorList, MainnetNodeProviderList,
    MainnetRegistryVersion, MainnetSubnetTopology,
    source::{
        fetch_mainnet_data_center_list_async, fetch_mainnet_node_list_async,
        fetch_mainnet_node_operator_list_async, fetch_mainnet_node_provider_list_async,
        fetch_mainnet_registry_version_async, fetch_mainnet_subnet_topology_async,
    },
};
use super::{
    MainnetRegistryFetchRequest, RegistryFetchError,
    source::fetch_mainnet_subnet_catalog_async as fetch_mainnet_subnet_catalog_from_source_async,
};
use crate::{runtime::block_on_current_thread, subnet_catalog::RawSubnetCatalog};

/// Fetch one exact-version mainnet Subnet Catalog through a synchronous runtime adapter.
pub fn fetch_mainnet_subnet_catalog(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_subnet_catalog_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

/// Fetch one exact-version mainnet Subnet Catalog on the caller's async runtime.
pub async fn fetch_mainnet_subnet_catalog_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, RegistryFetchError> {
    fetch_mainnet_subnet_catalog_from_source_async(request).await
}

#[cfg(feature = "host")]
pub fn fetch_mainnet_subnet_topology(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetSubnetTopology, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_subnet_topology_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

#[cfg(feature = "host")]
pub fn fetch_mainnet_registry_version(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetRegistryVersion, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_registry_version_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

#[cfg(feature = "host")]
pub fn fetch_mainnet_node_provider_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeProviderList, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_node_provider_list_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

#[cfg(feature = "host")]
pub fn fetch_mainnet_node_operator_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeOperatorList, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_node_operator_list_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

#[cfg(feature = "host")]
pub fn fetch_mainnet_node_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeList, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_node_list_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

#[cfg(feature = "host")]
pub fn fetch_mainnet_data_center_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetDataCenterList, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_data_center_list_async(request))
        .map_err(RegistryFetchError::Runtime)?
}
