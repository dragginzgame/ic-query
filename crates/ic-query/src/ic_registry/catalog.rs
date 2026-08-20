use super::{
    MainnetRegistryFetchRequest, ROUTING_TABLE_KEY, RegistryFetchError, SUBNET_LIST_KEY,
    SubnetCatalogRegistryFailure, canister_id_text, principal_text_from_raw,
    projection::subnet_kind_from_registry,
    proto::{RoutingTable, SubnetListRecord, SubnetRecord},
    subnet_id_text, subnet_record_key,
    transport::{RegistryQueryCounter, decode_message, get_registry_value_counted},
};
use crate::subnet_catalog::{
    ClassificationSource, GeographicScope, RawSubnetCatalog, RoutingRange, SubnetCatalogField,
    SubnetCatalogRegistryRecordKind, SubnetCatalogRegistryRecordSubject, SubnetCatalogSubject,
    SubnetInfo, SubnetSpecialization, UncertifiedCatalogCollection,
};
use candid::Principal;
use futures::future::try_join_all;
use ic_agent::Agent;

pub(super) async fn catalog_from_registry_records_detailed(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    agent: &Agent,
    registry_canister: &Principal,
    subnet_list: SubnetListRecord,
    routing_table: RoutingTable,
    query_counter: &RegistryQueryCounter,
) -> Result<RawSubnetCatalog, SubnetCatalogRegistryFailure> {
    if subnet_list.subnets.is_empty() {
        return Err(registry_record_failure(
            registry_version,
            SUBNET_LIST_KEY,
            SubnetCatalogRegistryRecordKind::SubnetList,
            RegistryFetchError::EmptySubnetList,
        ));
    }
    if routing_table.entries.is_empty() {
        return Err(registry_record_failure(
            registry_version,
            ROUTING_TABLE_KEY,
            SubnetCatalogRegistryRecordKind::RoutingTable,
            RegistryFetchError::EmptyRoutingTable,
        ));
    }

    let subnets = try_join_all(
        subnet_list
            .subnets
            .into_iter()
            .map(|subnet_raw| async move {
                let subnet_principal = principal_text_from_raw(&subnet_raw, "subnet_list.subnets")
                    .map_err(|source| {
                        SubnetCatalogRegistryFailure::new(
                            Some(registry_version),
                            Some(SubnetCatalogSubject::Field(
                                SubnetCatalogField::SubnetListSubnet,
                            )),
                            source,
                        )
                    })?;
                let subnet = Principal::from_text(&subnet_principal).map_err(|error| {
                    SubnetCatalogRegistryFailure::new(
                        Some(registry_version),
                        Some(SubnetCatalogSubject::Field(
                            SubnetCatalogField::SubnetListSubnet,
                        )),
                        RegistryFetchError::InvalidPrincipal {
                            field: "subnet_list.subnets",
                            reason: error.to_string(),
                        },
                    )
                })?;
                let key = subnet_record_key(&subnet_principal);
                let record_bytes = get_registry_value_counted(
                    agent,
                    registry_canister,
                    &key,
                    registry_version,
                    query_counter,
                )
                .await
                .map_err(|source| subnet_record_failure(registry_version, &key, subnet, source))?;
                let record = decode_message::<SubnetRecord>("SubnetRecord", &record_bytes)
                    .map_err(|source| {
                        subnet_record_failure(registry_version, &key, subnet, source)
                    })?;
                Ok::<_, SubnetCatalogRegistryFailure>(subnet_info_from_record(
                    &subnet_principal,
                    &record,
                ))
            }),
    )
    .await?;

    let routing_ranges = routing_ranges_from_table_detailed(&routing_table, registry_version)?;
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
    .map_err(|source| {
        SubnetCatalogRegistryFailure::new(Some(registry_version), None, source.into())
    })
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

#[cfg(feature = "certified-subnet-catalog-host")]
pub fn routing_ranges_from_table(
    table: &RoutingTable,
) -> Result<Vec<RoutingRange>, RegistryFetchError> {
    routing_ranges_from_table_inner(table).map_err(|failure| failure.source)
}

#[expect(
    clippy::result_large_err,
    reason = "typed Registry failures retain source and subject provenance"
)]
fn routing_ranges_from_table_detailed(
    table: &RoutingTable,
    registry_version: u64,
) -> Result<Vec<RoutingRange>, SubnetCatalogRegistryFailure> {
    routing_ranges_from_table_inner(table).map_err(|failure| SubnetCatalogRegistryFailure {
        registry_version: Some(registry_version),
        ..failure
    })
}

#[expect(
    clippy::result_large_err,
    reason = "typed Registry failures retain source and subject provenance"
)]
fn routing_ranges_from_table_inner(
    table: &RoutingTable,
) -> Result<Vec<RoutingRange>, SubnetCatalogRegistryFailure> {
    table
        .entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let range = entry.range.as_ref().ok_or_else(|| {
                routing_range_failure(
                    index,
                    SubnetCatalogField::RoutingTableRange,
                    RegistryFetchError::MissingField {
                        field: "routing_table.entries.range",
                    },
                )
            })?;
            let subnet_id = entry.subnet_id.as_ref().ok_or_else(|| {
                routing_range_failure(
                    index,
                    SubnetCatalogField::RoutingTableSubnetId,
                    RegistryFetchError::MissingField {
                        field: "routing_table.entries.subnet_id",
                    },
                )
            })?;
            Ok(RoutingRange {
                start_canister_id: canister_id_text(
                    range.start_canister_id.as_ref(),
                    "range.start",
                )
                .map_err(|source| {
                    routing_range_failure(index, SubnetCatalogField::RoutingRangeStart, source)
                })?,
                end_canister_id: canister_id_text(range.end_canister_id.as_ref(), "range.end")
                    .map_err(|source| {
                        routing_range_failure(index, SubnetCatalogField::RoutingRangeEnd, source)
                    })?,
                subnet_principal: subnet_id_text(subnet_id).map_err(|source| {
                    routing_range_failure(index, SubnetCatalogField::RoutingTableSubnetId, source)
                })?,
            })
        })
        .collect()
}

fn registry_record_failure(
    registry_version: u64,
    key: &str,
    kind: SubnetCatalogRegistryRecordKind,
    source: RegistryFetchError,
) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        Some(registry_version),
        Some(SubnetCatalogSubject::RegistryRecord(
            SubnetCatalogRegistryRecordSubject::keyed(kind, key),
        )),
        source,
    )
}

fn subnet_record_failure(
    registry_version: u64,
    key: &str,
    subnet: Principal,
    source: RegistryFetchError,
) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        Some(registry_version),
        Some(SubnetCatalogSubject::RegistryRecord(
            SubnetCatalogRegistryRecordSubject::subnet_record(key, subnet),
        )),
        source,
    )
}

const fn routing_range_failure(
    index: usize,
    field: SubnetCatalogField,
    source: RegistryFetchError,
) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        None,
        Some(SubnetCatalogSubject::RoutingRange {
            index,
            field: Some(field),
        }),
        source,
    )
}

#[cfg(test)]
mod detailed_failure_tests {
    use super::*;

    const SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";

    #[test]
    fn subnet_record_failure_retains_exact_pinned_subject() {
        let subnet = Principal::from_text(SUBNET).expect("subnet");
        let key = subnet_record_key(SUBNET);
        let failure = subnet_record_failure(
            882_110,
            &key,
            subnet,
            RegistryFetchError::MissingValue { key: key.clone() },
        );

        assert_eq!(failure.registry_version, Some(882_110));
        assert_eq!(
            failure.subject,
            Some(SubnetCatalogSubject::RegistryRecord(
                SubnetCatalogRegistryRecordSubject {
                    kind: SubnetCatalogRegistryRecordKind::SubnetRecord,
                    key: Some(key),
                    subnet: Some(subnet),
                }
            ))
        );
    }

    #[test]
    fn routing_table_and_range_failures_retain_pinned_typed_subjects() {
        let table_failure = registry_record_failure(
            882_111,
            ROUTING_TABLE_KEY,
            SubnetCatalogRegistryRecordKind::RoutingTable,
            RegistryFetchError::EmptyRoutingTable,
        );
        assert_eq!(table_failure.registry_version, Some(882_111));
        assert!(matches!(
            table_failure.subject,
            Some(SubnetCatalogSubject::RegistryRecord(
                SubnetCatalogRegistryRecordSubject {
                    kind: SubnetCatalogRegistryRecordKind::RoutingTable,
                    ..
                }
            ))
        ));

        let range_failure = routing_range_failure(
            7,
            SubnetCatalogField::RoutingRangeStart,
            RegistryFetchError::MissingField {
                field: "range.start",
            },
        );
        let range_failure = SubnetCatalogRegistryFailure {
            registry_version: Some(882_111),
            ..range_failure
        };
        assert_eq!(range_failure.registry_version, Some(882_111));
        assert_eq!(
            range_failure.subject,
            Some(SubnetCatalogSubject::RoutingRange {
                index: 7,
                field: Some(SubnetCatalogField::RoutingRangeStart),
            })
        );
    }
}
