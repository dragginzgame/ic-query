use super::{
    MainnetDataCenterList, MainnetNodeList, MainnetNodeOperatorList, MainnetNodeProviderList,
    MainnetRegistryFetchRequest, MainnetRegistryVersion, RegistryFetchError,
    source::{
        fetch_mainnet_data_center_list_async, fetch_mainnet_node_list_async,
        fetch_mainnet_node_operator_list_async, fetch_mainnet_node_provider_list_async,
        fetch_mainnet_registry_version_async, fetch_mainnet_subnet_catalog_async,
    },
};
use crate::{runtime::block_on_current_thread, subnet_catalog::SubnetCatalog};

pub fn fetch_mainnet_subnet_catalog(
    request: &MainnetRegistryFetchRequest,
) -> Result<SubnetCatalog, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_subnet_catalog_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

pub fn fetch_mainnet_registry_version(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetRegistryVersion, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_registry_version_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

pub fn fetch_mainnet_node_provider_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeProviderList, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_node_provider_list_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

pub fn fetch_mainnet_node_operator_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeOperatorList, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_node_operator_list_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

pub fn fetch_mainnet_node_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeList, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_node_list_async(request))
        .map_err(RegistryFetchError::Runtime)?
}

pub fn fetch_mainnet_data_center_list(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetDataCenterList, RegistryFetchError> {
    block_on_current_thread(fetch_mainnet_data_center_list_async(request))
        .map_err(RegistryFetchError::Runtime)?
}
