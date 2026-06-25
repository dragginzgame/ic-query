use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, ROUTING_TABLE_KEY, RegistryFetchError, SUBNET_LIST_KEY,
        catalog::catalog_from_registry_records,
        proto::{RoutingTable, SubnetListRecord},
        transport::{decode_message, get_latest_version, get_registry_value},
    },
    subnet_catalog::SubnetCatalog,
};

pub(in crate::ic_registry) async fn fetch_mainnet_subnet_catalog_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<SubnetCatalog, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let subnet_list_bytes = get_registry_value(
        &agent,
        &registry_canister,
        SUBNET_LIST_KEY,
        registry_version,
    )
    .await?;
    let routing_table_bytes = get_registry_value(
        &agent,
        &registry_canister,
        ROUTING_TABLE_KEY,
        registry_version,
    )
    .await?;
    let subnet_list = decode_message::<SubnetListRecord>("SubnetListRecord", &subnet_list_bytes)?;
    let routing_table = decode_message::<RoutingTable>("RoutingTable", &routing_table_bytes)?;
    catalog_from_registry_records(
        request,
        registry_version,
        &agent,
        &registry_canister,
        subnet_list,
        routing_table,
    )
    .await
}
