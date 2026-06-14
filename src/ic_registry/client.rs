use super::{
    MAINNET_GOVERNANCE_CANISTER_ID, MainnetDataCenterList, MainnetNodeList,
    MainnetNodeOperatorList, MainnetNodeProviderList, MainnetRegistryFetchRequest,
    MainnetRegistryVersion, ROUTING_TABLE_KEY, RegistryFetchError, SUBNET_LIST_KEY,
    fetch_node_provider_node_counts, fetch_registry_relation_inventory,
};
use super::{
    catalog::catalog_from_registry_records,
    projection::{
        data_center_list_from_inventory, node_list_from_inventory,
        node_operator_list_from_inventory, node_provider_list_from_response,
    },
    proto::{RoutingTable, SubnetListRecord},
    relations::RegistryRelationInventoryScope,
    transport::{decode_message, get_latest_version, get_registry_value},
    wire::ListNodeProvidersResponse,
};
use crate::{
    runtime::block_on_current_thread,
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, SubnetCatalog},
};
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;

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

fn mainnet_agent(request: &MainnetRegistryFetchRequest) -> Result<Agent, RegistryFetchError> {
    Agent::builder()
        .with_url(&request.endpoint)
        .build()
        .map_err(|err| RegistryFetchError::AgentBuild {
            endpoint: request.endpoint.clone(),
            reason: err.to_string(),
        })
}

fn mainnet_registry_canister() -> Result<Principal, RegistryFetchError> {
    principal_from_text(MAINNET_REGISTRY_CANISTER_ID, "registry_canister_id")
}

fn mainnet_governance_canister() -> Result<Principal, RegistryFetchError> {
    principal_from_text(MAINNET_GOVERNANCE_CANISTER_ID, "governance_canister_id")
}

fn principal_from_text(value: &str, field: &'static str) -> Result<Principal, RegistryFetchError> {
    Principal::from_text(value).map_err(|err| RegistryFetchError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}

async fn fetch_mainnet_registry_version_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetRegistryVersion, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    Ok(MainnetRegistryVersion {
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
    })
}

async fn fetch_mainnet_subnet_catalog_async(
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

async fn fetch_mainnet_node_provider_list_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeProviderList, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let governance_canister = mainnet_governance_canister()?;
    let arg = Encode!().map_err(|err| RegistryFetchError::CandidEncode {
        message: "list_node_providers",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&governance_canister, "list_node_providers")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| RegistryFetchError::AgentCall {
            method: "list_node_providers",
            reason: err.to_string(),
        })?;
    let response = Decode!(&bytes, ListNodeProvidersResponse).map_err(|err| {
        RegistryFetchError::CandidDecode {
            message: "ListNodeProvidersResponse",
            reason: err.to_string(),
        }
    })?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let node_counts =
        fetch_node_provider_node_counts(&agent, &registry_canister, registry_version).await?;
    node_provider_list_from_response(request, response, node_counts, registry_version)
}

async fn fetch_mainnet_node_operator_list_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeOperatorList, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let inventory = fetch_registry_relation_inventory(
        &agent,
        &registry_canister,
        registry_version,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    node_operator_list_from_inventory(request, inventory, registry_version)
}

async fn fetch_mainnet_node_list_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetNodeList, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let inventory = fetch_registry_relation_inventory(
        &agent,
        &registry_canister,
        registry_version,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    node_list_from_inventory(request, inventory, registry_version)
}

async fn fetch_mainnet_data_center_list_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<MainnetDataCenterList, RegistryFetchError> {
    let agent = mainnet_agent(request)?;
    let registry_canister = mainnet_registry_canister()?;
    let registry_version = get_latest_version(&agent, &registry_canister).await?;
    let inventory = fetch_registry_relation_inventory(
        &agent,
        &registry_canister,
        registry_version,
        RegistryRelationInventoryScope::WithDataCenters,
    )
    .await?;
    data_center_list_from_inventory(request, inventory, registry_version)
}
