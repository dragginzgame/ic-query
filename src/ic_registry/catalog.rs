use super::{
    MainnetRegistryFetchRequest, RegistryFetchError, apply_mainnet_annotations, canister_id_text,
    decode_message, get_registry_value, principal_text_from_raw,
    proto::{RoutingTable, SubnetListRecord, SubnetRecord, SubnetType},
    subnet_id_text, subnet_record_key,
};
use crate::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, ClassificationSource, GeographicScope, MAINNET_NETWORK,
    MAINNET_REGISTRY_CANISTER_ID, RoutingRange, SubnetCatalog, SubnetInfo, SubnetKind,
    SubnetSpecialization,
};
use candid::Principal;
use ic_agent::Agent;

pub(super) async fn catalog_from_registry_records(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    agent: &Agent,
    registry_canister: &Principal,
    subnet_list: SubnetListRecord,
    routing_table: RoutingTable,
) -> Result<SubnetCatalog, RegistryFetchError> {
    if subnet_list.subnets.is_empty() {
        return Err(RegistryFetchError::EmptySubnetList);
    }
    if routing_table.entries.is_empty() {
        return Err(RegistryFetchError::EmptyRoutingTable);
    }

    let mut subnets = Vec::with_capacity(subnet_list.subnets.len());
    for subnet_raw in subnet_list.subnets {
        let subnet_principal = principal_text_from_raw(&subnet_raw, "subnet_list.subnets")?;
        let key = subnet_record_key(&subnet_principal);
        let record_bytes =
            get_registry_value(agent, registry_canister, &key, registry_version).await?;
        let record = decode_message::<SubnetRecord>("SubnetRecord", &record_bytes)?;
        subnets.push(subnet_info_from_record(&subnet_principal, &record));
    }

    subnets.sort_by(|left, right| left.subnet_principal.cmp(&right.subnet_principal));

    let mut routing_ranges = routing_ranges_from_table(&routing_table)?;
    routing_ranges.sort_by(|left, right| {
        left.start_canister_id
            .cmp(&right.start_canister_id)
            .then_with(|| left.end_canister_id.cmp(&right.end_canister_id))
            .then_with(|| left.subnet_principal.cmp(&right.subnet_principal))
    });

    let mut catalog = SubnetCatalog {
        catalog_schema_version: CATALOG_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        subnets,
        routing_ranges,
    };
    apply_mainnet_annotations(&mut catalog);
    catalog.validate()?;
    Ok(catalog)
}

pub(super) fn subnet_info_from_record(subnet_principal: &str, record: &SubnetRecord) -> SubnetInfo {
    let subnet_kind = match SubnetType::try_from(record.subnet_type).ok() {
        Some(SubnetType::Application | SubnetType::VerifiedApplication) => SubnetKind::Application,
        Some(SubnetType::CloudEngine) => SubnetKind::CloudEngine,
        Some(SubnetType::System) => SubnetKind::System,
        Some(SubnetType::Unspecified) | None => SubnetKind::Unknown,
    };
    let charges_apply_by_default = subnet_kind.charges_apply_by_default();
    SubnetInfo {
        subnet_principal: subnet_principal.to_string(),
        subnet_kind,
        subnet_kind_source: ClassificationSource::Registry,
        subnet_specialization: SubnetSpecialization::None,
        subnet_specialization_source: ClassificationSource::Computed,
        geographic_scope: GeographicScope::Global,
        geographic_scope_source: ClassificationSource::Computed,
        subnet_label: subnet_kind.as_str().to_string(),
        subnet_label_source: ClassificationSource::Computed,
        node_count: Some(u32::try_from(record.membership.len()).unwrap_or(u32::MAX)),
        charges_apply_by_default,
    }
}

pub(super) fn routing_ranges_from_table(
    table: &RoutingTable,
) -> Result<Vec<RoutingRange>, RegistryFetchError> {
    table
        .entries
        .iter()
        .map(|entry| {
            let range = entry
                .range
                .as_ref()
                .ok_or(RegistryFetchError::MissingField {
                    field: "routing_table.entries.range",
                })?;
            let subnet_id = entry
                .subnet_id
                .as_ref()
                .ok_or(RegistryFetchError::MissingField {
                    field: "routing_table.entries.subnet_id",
                })?;
            Ok(RoutingRange {
                start_canister_id: canister_id_text(
                    range.start_canister_id.as_ref(),
                    "range.start",
                )?,
                end_canister_id: canister_id_text(range.end_canister_id.as_ref(), "range.end")?,
                subnet_principal: subnet_id_text(subnet_id)?,
            })
        })
        .collect()
}
