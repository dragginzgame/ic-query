use super::{
    MainnetRegistryFetchRequest, ROUTING_TABLE_KEY, RegistryFetchError,
    SubnetCatalogRegistryFailure, canister_id_text, principal_from_raw,
    projection::subnet_kind_from_registry,
    proto::{RoutingTable, SubnetListRecord, SubnetRecord},
    subnet_id_text,
    transport::decode_message,
    wire::{RegistryValueEncoding, RegistryVersionedValue, RegistryVersionedValueFailure},
};
use crate::subnet_catalog::{
    ClassificationSource, GeographicScope, RawSubnetCatalog, RoutingRange, SubnetCatalogField,
    SubnetCatalogRegistryRecordEvidence, SubnetCatalogRegistryRecordKind,
    SubnetCatalogRegistryRecordSubject, SubnetCatalogRegistryValueEncoding,
    SubnetCatalogRoutingSource, SubnetCatalogSubject, SubnetInfo, SubnetSpecialization,
    UncertifiedCatalogCollection, subject_from_catalog_error,
};
use std::future::Future;

///
/// CatalogRegistryReader
///
/// Pinned Registry key-family and value reads used by Subnet Catalog collection.
///

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
        return Err(record_failure(
            request,
            registry_version,
            SubnetCatalogRegistryRecordSubject::subnet_list(),
            None,
            registry_records,
            RegistryFetchError::EmptySubnetList,
        ));
    }
    if routing_table.entries.is_empty() {
        let key = match routing_source {
            SubnetCatalogRoutingSource::CanisterRanges => {
                crate::ic_registry::CANISTER_RANGES_KEY_PREFIX
            }
            SubnetCatalogRoutingSource::LegacyRoutingTable => ROUTING_TABLE_KEY,
        };
        return Err(record_failure(
            request,
            registry_version,
            SubnetCatalogRegistryRecordSubject::exact_keyed(
                SubnetCatalogRegistryRecordKind::RoutingTable,
                key,
            ),
            None,
            registry_records,
            RegistryFetchError::EmptyRoutingTable,
        ));
    }

    let mut subnets = Vec::with_capacity(subnet_list.subnets.len());
    for subnet_raw in subnet_list.subnets {
        let subnet = principal_from_raw(&subnet_raw, "subnet_list.subnets").map_err(|source| {
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
        let subnet_principal = subnet.to_text();
        let subject = SubnetCatalogRegistryRecordSubject::subnet_record(subnet);
        let value = get_catalog_record(
            reader,
            &subject,
            registry_version,
            request,
            &registry_records,
        )
        .await?;
        registry_records.push(record_evidence(
            request,
            registry_version,
            subject.clone(),
            &value,
        ));
        let record = decode_message::<SubnetRecord>(subject.kind.protobuf_schema(), &value.value)
            .map_err(|source| {
            record_failure(
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
        let subject = subject_from_catalog_error(&source);
        SubnetCatalogRegistryFailure::new(Some(registry_version), subject, source.into())
            .with_value_response(&request.endpoint, None)
            .with_registry_records(registry_records)
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

pub(super) async fn get_catalog_record<R>(
    reader: &R,
    subject: &SubnetCatalogRegistryRecordSubject,
    registry_version: u64,
    request: &MainnetRegistryFetchRequest,
    registry_records: &[SubnetCatalogRegistryRecordEvidence],
) -> Result<RegistryVersionedValue, SubnetCatalogRegistryFailure>
where
    R: CatalogRegistryReader,
{
    let value = reader
        .value(&subject.key, registry_version)
        .await
        .map_err(|failure| {
            record_failure(
                request,
                registry_version,
                subject.clone(),
                failure.returned_version,
                registry_records.to_vec(),
                failure.source,
            )
        })?;
    if value.version > registry_version {
        return Err(record_failure(
            request,
            registry_version,
            subject.clone(),
            Some(value.version),
            registry_records.to_vec(),
            RegistryFetchError::InvalidRegistryValueVersion {
                key: subject.key.clone(),
                requested_version: registry_version,
                returned_version: value.version,
            },
        ));
    }
    Ok(value)
}

pub(super) fn record_evidence(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    record: SubnetCatalogRegistryRecordSubject,
    value: &RegistryVersionedValue,
) -> SubnetCatalogRegistryRecordEvidence {
    SubnetCatalogRegistryRecordEvidence::uncertified_query(
        record,
        registry_version,
        value.version,
        value.timestamp_nanoseconds,
        &request.endpoint,
        match value.encoding {
            RegistryValueEncoding::Inline => SubnetCatalogRegistryValueEncoding::Inline,
            RegistryValueEncoding::Chunked => SubnetCatalogRegistryValueEncoding::Chunked,
        },
    )
}

pub(super) fn record_failure(
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
    use candid::Principal;

    const SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";

    #[test]
    fn subnet_record_failure_retains_exact_pinned_subject() {
        let request = MainnetRegistryFetchRequest {
            endpoint: "https://icp-api.io".to_string(),
            fetched_at: "2026-08-20T00:00:00Z".to_string(),
            fetched_by: "test".to_string(),
        };
        let subnet = Principal::from_text(SUBNET).expect("subnet");
        let key = crate::ic_registry::subnet_record_key(SUBNET);
        let failure = record_failure(
            &request,
            882_110,
            SubnetCatalogRegistryRecordSubject::subnet_record(subnet),
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
        let table_failure = record_failure(
            &request,
            882_111,
            SubnetCatalogRegistryRecordSubject::legacy_routing_table(),
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
