use super::{
    MainnetRegistryFetchRequest, ROUTING_TABLE_KEY, RegistryFetchError, SUBNET_LIST_KEY,
    SubnetCatalogRegistryFailure, canister_id_text, principal_text_from_raw,
    projection::subnet_kind_from_registry,
    proto::{RoutingTable, SubnetListRecord, SubnetRecord},
    subnet_id_text, subnet_record_key,
    transport::decode_message,
    wire::{RegistryValueEncoding, RegistryVersionedValue, RegistryVersionedValueFailure},
};
use crate::subnet_catalog::{
    CatalogAssurance, CatalogError, ClassificationSource, GeographicScope, RawSubnetCatalog,
    RoutingRange, SubnetCatalogField, SubnetCatalogRegistryRecordEvidence,
    SubnetCatalogRegistryRecordKind, SubnetCatalogRegistryRecordSubject,
    SubnetCatalogRegistryValueEncoding, SubnetCatalogRoutingSource, SubnetCatalogSubject,
    SubnetInfo, SubnetSpecialization, UncertifiedCatalogCollection,
};
use candid::Principal;
use std::future::Future;

pub(super) trait CatalogRegistryReader: Sync {
    fn key_family(
        &self,
        prefix: &str,
        registry_version: u64,
    ) -> impl Future<Output = Result<Vec<String>, RegistryFetchError>> + Send;

    fn value(
        &self,
        key: &str,
        registry_version: u64,
    ) -> impl Future<Output = Result<RegistryVersionedValue, RegistryVersionedValueFailure>> + Send;

    fn query_call_count(&self) -> u64;
}

#[expect(
    clippy::too_many_lines,
    reason = "the collector keeps one visible fail-closed sequence for Subnet records and evidence"
)]
pub(super) async fn catalog_from_registry_records_detailed<R>(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    reader: &R,
    subnet_list: SubnetListRecord,
    routing_table: RoutingTable,
    routing_source: SubnetCatalogRoutingSource,
    mut registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
) -> Result<RawSubnetCatalog, SubnetCatalogRegistryFailure>
where
    R: CatalogRegistryReader,
{
    if subnet_list.subnets.is_empty() {
        return Err(registry_record_failure(
            request,
            registry_version,
            SUBNET_LIST_KEY,
            SubnetCatalogRegistryRecordKind::SubnetList,
            None,
            registry_records,
            RegistryFetchError::EmptySubnetList,
        ));
    }
    if routing_table.entries.is_empty() {
        let (key, kind) = match routing_source {
            SubnetCatalogRoutingSource::CanisterRanges => (
                crate::ic_registry::CANISTER_RANGES_KEY_PREFIX,
                SubnetCatalogRegistryRecordKind::RoutingTable,
            ),
            SubnetCatalogRoutingSource::LegacyRoutingTable => (
                ROUTING_TABLE_KEY,
                SubnetCatalogRegistryRecordKind::RoutingTable,
            ),
        };
        return Err(registry_record_failure(
            request,
            registry_version,
            key,
            kind,
            None,
            registry_records,
            RegistryFetchError::EmptyRoutingTable,
        ));
    }

    let mut subnets = Vec::with_capacity(subnet_list.subnets.len());
    for subnet_raw in subnet_list.subnets {
        let subnet_principal = principal_text_from_raw(&subnet_raw, "subnet_list.subnets")
            .map_err(|source| {
                SubnetCatalogRegistryFailure::new(
                    Some(registry_version),
                    Some(SubnetCatalogSubject::Field(
                        SubnetCatalogField::SubnetListEntry,
                    )),
                    source,
                )
                .with_value_response(&request.endpoint, None)
                .with_registry_records(registry_records.clone())
            })?;
        let subnet = Principal::from_text(&subnet_principal).map_err(|error| {
            SubnetCatalogRegistryFailure::new(
                Some(registry_version),
                Some(SubnetCatalogSubject::Field(
                    SubnetCatalogField::SubnetListEntry,
                )),
                RegistryFetchError::InvalidPrincipal {
                    field: "subnet_list.subnets",
                    reason: error.to_string(),
                },
            )
            .with_value_response(&request.endpoint, None)
            .with_registry_records(registry_records.clone())
        })?;
        let key = subnet_record_key(&subnet_principal);
        let subject = SubnetCatalogRegistryRecordSubject::subnet_record(&key, subnet);
        let value = reader
            .value(&key, registry_version)
            .await
            .map_err(|failure| {
                subnet_record_fetch_failure(
                    request,
                    registry_version,
                    subject.clone(),
                    registry_records.clone(),
                    failure,
                )
            })?;
        if value.version > registry_version {
            return Err(subnet_record_failure(
                request,
                registry_version,
                subject,
                Some(value.version),
                registry_records,
                RegistryFetchError::InvalidRegistryValueVersion {
                    key,
                    requested_version: registry_version,
                    returned_version: value.version,
                },
            ));
        }
        registry_records.push(SubnetCatalogRegistryRecordEvidence {
            record: subject.clone(),
            requested_registry_version: registry_version,
            returned_registry_version: value.version,
            timestamp_nanoseconds: value.timestamp_nanoseconds,
            source_endpoint: request.endpoint.clone(),
            assurance: CatalogAssurance::UncertifiedQuery,
            value_encoding: match value.encoding {
                RegistryValueEncoding::Inline => SubnetCatalogRegistryValueEncoding::Inline,
                RegistryValueEncoding::Chunked => SubnetCatalogRegistryValueEncoding::Chunked,
            },
        });
        let record = decode_message::<SubnetRecord>(subject.kind.protobuf_schema(), &value.value)
            .map_err(|source| {
            subnet_record_failure(
                request,
                registry_version,
                subject,
                Some(value.version),
                registry_records.clone(),
                source,
            )
        })?;
        subnets.push(subnet_info_from_record(&subnet_principal, &record));
    }

    let routing_ranges = routing_ranges_from_table_detailed(&routing_table, registry_version)
        .map_err(|failure| {
            failure
                .with_value_response(&request.endpoint, None)
                .with_registry_records(registry_records.clone())
        })?;
    RawSubnetCatalog::new_mainnet_uncertified(
        UncertifiedCatalogCollection::new(
            registry_version,
            &request.endpoint,
            &request.fetched_at,
            &request.fetched_by,
            env!("CARGO_PKG_VERSION"),
            reader.query_call_count(),
        )
        .with_registry_evidence(routing_source, registry_records.clone()),
        subnets,
        routing_ranges,
    )
    .map_err(|source| {
        let subject = catalog_error_subject(&source);
        SubnetCatalogRegistryFailure::new(Some(registry_version), subject, source.into())
            .with_value_response(&request.endpoint, None)
            .with_registry_records(registry_records)
    })
}

fn catalog_error_subject(source: &CatalogError) -> Option<SubnetCatalogSubject> {
    match source {
        CatalogError::UnknownRoutingSubnet { subnet_principal } => {
            Principal::from_text(subnet_principal)
                .ok()
                .map(|subnet| SubnetCatalogSubject::Subnet {
                    subnet,
                    field: Some(SubnetCatalogField::RoutingTableSubnetId),
                })
        }
        CatalogError::DuplicateSubnet { subnet_principal } => {
            Principal::from_text(subnet_principal)
                .ok()
                .map(|subnet| SubnetCatalogSubject::Subnet {
                    subnet,
                    field: Some(SubnetCatalogField::SubnetListEntry),
                })
        }
        CatalogError::InvalidRoutingRange {
            start_canister_id,
            end_canister_id,
            subnet_principal,
        } => Some(SubnetCatalogSubject::RoutingRange {
            range: RoutingRange {
                start_canister_id: start_canister_id.clone(),
                end_canister_id: end_canister_id.clone(),
                subnet_principal: subnet_principal.clone(),
            },
            field: None,
        }),
        CatalogError::OverlappingRoutingRanges { first, .. }
        | CatalogError::NonCanonicalRoutingOrder {
            previous: first, ..
        } => Some(SubnetCatalogSubject::RoutingRange {
            range: first.as_ref().clone(),
            field: None,
        }),
        _ => None,
    }
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
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    key: &str,
    kind: SubnetCatalogRegistryRecordKind,
    returned_registry_value_version: Option<u64>,
    registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
    source: RegistryFetchError,
) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        Some(registry_version),
        Some(SubnetCatalogSubject::RegistryRecord(
            SubnetCatalogRegistryRecordSubject::keyed(kind, key),
        )),
        source,
    )
    .with_value_response(&request.endpoint, returned_registry_value_version)
    .with_registry_records(registry_records)
}

fn subnet_record_failure(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    subject: SubnetCatalogRegistryRecordSubject,
    returned_registry_value_version: Option<u64>,
    registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
    source: RegistryFetchError,
) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        Some(registry_version),
        Some(SubnetCatalogSubject::RegistryRecord(subject)),
        source,
    )
    .with_value_response(&request.endpoint, returned_registry_value_version)
    .with_registry_records(registry_records)
}

fn subnet_record_fetch_failure(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    subject: SubnetCatalogRegistryRecordSubject,
    registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
    failure: RegistryVersionedValueFailure,
) -> SubnetCatalogRegistryFailure {
    subnet_record_failure(
        request,
        registry_version,
        subject,
        failure.returned_version,
        registry_records,
        failure.source,
    )
}

const fn routing_range_failure(
    index: usize,
    field: SubnetCatalogField,
    source: RegistryFetchError,
) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        None,
        Some(SubnetCatalogSubject::RegistryRoutingTableEntry {
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
        let request = MainnetRegistryFetchRequest {
            endpoint: "https://icp-api.io".to_string(),
            fetched_at: "2026-08-20T00:00:00Z".to_string(),
            fetched_by: "test".to_string(),
        };
        let subnet = Principal::from_text(SUBNET).expect("subnet");
        let key = subnet_record_key(SUBNET);
        let failure = subnet_record_failure(
            &request,
            882_110,
            SubnetCatalogRegistryRecordSubject::subnet_record(&key, subnet),
            Some(882_000),
            Vec::new(),
            RegistryFetchError::MissingValue { key: key.clone() },
        );

        assert_eq!(failure.registry_version, Some(882_110));
        assert_eq!(
            failure.subject,
            Some(SubnetCatalogSubject::RegistryRecord(
                SubnetCatalogRegistryRecordSubject {
                    kind: SubnetCatalogRegistryRecordKind::SubnetRecord,
                    key,
                    subnet: Some(subnet),
                    canister_range_start: None,
                }
            ))
        );
        assert_eq!(failure.returned_registry_value_version, Some(882_000));
        assert_eq!(
            failure.source_endpoint.as_deref(),
            Some(request.endpoint.as_str())
        );
    }

    #[test]
    fn routing_table_and_range_failures_retain_pinned_typed_subjects() {
        let request = MainnetRegistryFetchRequest {
            endpoint: "https://icp-api.io".to_string(),
            fetched_at: "2026-08-20T00:00:00Z".to_string(),
            fetched_by: "test".to_string(),
        };
        let table_failure = registry_record_failure(
            &request,
            882_111,
            ROUTING_TABLE_KEY,
            SubnetCatalogRegistryRecordKind::RoutingTable,
            None,
            Vec::new(),
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
            Some(SubnetCatalogSubject::RegistryRoutingTableEntry {
                index: 7,
                field: Some(SubnetCatalogField::RoutingRangeStart),
            })
        );
    }
}
