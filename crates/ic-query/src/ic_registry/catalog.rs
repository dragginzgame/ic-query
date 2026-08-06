use super::{
    MainnetRegistryFetchRequest, RegistryFetchError, canister_id_text, principal_text_from_raw,
    projection::subnet_kind_from_registry,
    proto::{RoutingTable, SubnetListRecord, SubnetRecord},
    subnet_id_text, subnet_record_key,
    transport::{RegistryQueryCounter, decode_message, get_registry_value_counted},
};
use crate::subnet_catalog::{
    ClassificationSource, GeographicScope, RawSubnetCatalog, RoutingRange, SubnetInfo,
    SubnetSpecialization, UncertifiedCatalogCollection,
};
use candid::Principal;
use futures::future::try_join_all;
use ic_agent::Agent;

pub(super) async fn catalog_from_registry_records(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    agent: &Agent,
    registry_canister: &Principal,
    subnet_list: SubnetListRecord,
    routing_table: RoutingTable,
    query_counter: &RegistryQueryCounter,
) -> Result<RawSubnetCatalog, RegistryFetchError> {
    if subnet_list.subnets.is_empty() {
        return Err(RegistryFetchError::EmptySubnetList);
    }
    if routing_table.entries.is_empty() {
        return Err(RegistryFetchError::EmptyRoutingTable);
    }

    let subnets = try_join_all(
        subnet_list
            .subnets
            .into_iter()
            .map(|subnet_raw| async move {
                let subnet_principal = principal_text_from_raw(&subnet_raw, "subnet_list.subnets")?;
                let key = subnet_record_key(&subnet_principal);
                let record_bytes = get_registry_value_counted(
                    agent,
                    registry_canister,
                    &key,
                    registry_version,
                    query_counter,
                )
                .await?;
                let record = decode_message::<SubnetRecord>("SubnetRecord", &record_bytes)?;
                Ok::<_, RegistryFetchError>(subnet_info_from_record(&subnet_principal, &record))
            }),
    )
    .await?;

    let routing_ranges = routing_ranges_from_table(&routing_table)?;
    RawSubnetCatalog::new_mainnet_uncertified(
        UncertifiedCatalogCollection::new(
            registry_version,
            &request.endpoint,
            &request.fetched_at,
            &request.fetched_by,
            env!("CARGO_PKG_VERSION"),
            query_counter.call_count(),
        ),
        subnets,
        routing_ranges,
    )
    .map_err(RegistryFetchError::from)
}

pub fn subnet_info_from_record(subnet_principal: &str, record: &SubnetRecord) -> SubnetInfo {
    let subnet_kind = subnet_kind_from_registry(record.subnet_type);
    let charges_apply_by_default = subnet_kind.charges_apply_by_default();
    SubnetInfo {
        subnet_principal: subnet_principal.to_string(),
        registry_subnet_type: record.subnet_type,
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

pub fn routing_ranges_from_table(
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
