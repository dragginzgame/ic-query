use super::{
    MainnetRegistryFetchRequest, ROUTING_TABLE_KEY, RegistryFetchError,
    SubnetCatalogRegistryFailure, canister_id_text, principal_from_raw,
    projection::subnet_kind_from_registry,
    subnet_id_text,
    transport::decode_message,
    wire::{RegistryVersionedValue, RegistryVersionedValueFailure},
};
use crate::subnet_catalog::{
    ClassificationSource, GeographicScope, RawSubnetCatalog, RoutingRange, SubnetCatalogField,
    SubnetCatalogRegistryRecordEvidence, SubnetCatalogRegistryRecordKind,
    SubnetCatalogRegistryRecordSubject, SubnetCatalogRoutingSource, SubnetCatalogSubject,
    SubnetInfo, SubnetSpecialization, UncertifiedCatalogCollection, subject_from_catalog_error,
};
use ic_host_tools::registry::proto::{RoutingTable, SubnetListRecord, SubnetRecord};
use prost::Message;
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

///
/// CatalogReadContext
///
/// Pinned collection identity and completed Registry response evidence.
///

pub(super) struct CatalogReadContext<'a, R> {
    request: &'a MainnetRegistryFetchRequest,
    registry_version: u64,
    reader: &'a R,
    registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
}

impl<'a, R> CatalogReadContext<'a, R> {
    pub(super) const fn new(
        request: &'a MainnetRegistryFetchRequest,
        registry_version: u64,
        reader: &'a R,
    ) -> Self {
        Self {
            request,
            registry_version,
            reader,
            registry_records: Vec::new(),
        }
    }

    pub(super) fn record_failure(
        &self,
        subject: SubnetCatalogRegistryRecordSubject,
        returned_registry_value_version: Option<u64>,
        source: RegistryFetchError,
    ) -> Box<SubnetCatalogRegistryFailure> {
        SubnetCatalogRegistryFailure::new(
            Some(self.registry_version),
            Some(SubnetCatalogSubject::RegistryRecord(subject)),
            source,
        )
        .with_value_response(&self.request.endpoint, returned_registry_value_version)
        .with_registry_records(self.registry_records.clone())
    }

    pub(super) async fn record<Record>(
        &mut self,
        subject: &SubnetCatalogRegistryRecordSubject,
    ) -> Result<(Record, u64), Box<SubnetCatalogRegistryFailure>>
    where
        R: CatalogRegistryReader,
        Record: Message + Default,
    {
        let value = self
            .reader
            .value(&subject.key, self.registry_version)
            .await
            .map_err(|failure| {
                self.record_failure(subject.clone(), failure.returned_version, failure.source)
            })?;
        if value.version > self.registry_version {
            return Err(self.record_failure(
                subject.clone(),
                Some(value.version),
                RegistryFetchError::InvalidRegistryValueVersion {
                    key: subject.key.clone(),
                    requested_version: self.registry_version,
                    returned_version: value.version,
                },
            ));
        }
        self.registry_records.push(record_evidence(
            self.request,
            self.registry_version,
            subject.clone(),
            &value,
        ));
        let record = decode_message(subject.kind.protobuf_schema(), &value.value)
            .map_err(|source| self.record_failure(subject.clone(), Some(value.version), source))?;
        Ok((record, value.version))
    }
}

pub(super) async fn catalog_from_registry_records_detailed<R>(
    mut context: CatalogReadContext<'_, R>,
    subnet_list: SubnetListRecord,
    routing_table: RoutingTable,
    routing_source: SubnetCatalogRoutingSource,
) -> Result<RawSubnetCatalog, Box<SubnetCatalogRegistryFailure>>
where
    R: CatalogRegistryReader,
{
    let request = context.request;
    let registry_version = context.registry_version;
    if subnet_list.subnets.is_empty() {
        return Err(context.record_failure(
            SubnetCatalogRegistryRecordSubject::subnet_list(),
            None,
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
        return Err(context.record_failure(
            SubnetCatalogRegistryRecordSubject::keyed(
                SubnetCatalogRegistryRecordKind::RoutingTable,
                key,
            ),
            None,
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
            .with_registry_records(context.registry_records.clone())
        })?;
        let subnet_principal = subnet.to_text();
        let subject = SubnetCatalogRegistryRecordSubject::subnet_record(subnet);
        let (record, _) = context.record::<SubnetRecord>(&subject).await?;
        subnets.push(subnet_info_from_record(&subnet_principal, &record));
    }

    let routing_ranges = routing_ranges_from_table_detailed(&routing_table, registry_version)
        .map_err(|failure| {
            failure
                .with_value_response(&request.endpoint, None)
                .with_registry_records(context.registry_records.clone())
        })?;
    RawSubnetCatalog::new_mainnet_uncertified(
        UncertifiedCatalogCollection {
            registry_version,
            source_endpoint: request.endpoint.clone(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            collector_version: env!("CARGO_PKG_VERSION").to_string(),
            registry_query_call_count: context.reader.query_call_count(),
            routing_source,
            registry_records: context.registry_records.clone(),
        },
        subnets,
        routing_ranges,
    )
    .map_err(|source| {
        let subject = subject_from_catalog_error(&source);
        SubnetCatalogRegistryFailure::new(Some(registry_version), subject, source.into())
            .with_value_response(&request.endpoint, None)
            .with_registry_records(context.registry_records)
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

fn routing_ranges_from_table_detailed(
    table: &RoutingTable,
    registry_version: u64,
) -> Result<Vec<RoutingRange>, Box<SubnetCatalogRegistryFailure>> {
    routing_ranges_from_table_inner(table).map_err(|mut failure| {
        failure.registry_version = Some(registry_version);
        failure
    })
}

fn routing_ranges_from_table_inner(
    table: &RoutingTable,
) -> Result<Vec<RoutingRange>, Box<SubnetCatalogRegistryFailure>> {
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

fn record_evidence(
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
        value.encoding,
    )
}

fn routing_range_failure(
    index: usize,
    field: SubnetCatalogField,
    source: RegistryFetchError,
) -> Box<SubnetCatalogRegistryFailure> {
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
    use ic_platform::candid::contract::Principal;

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
        let failure = CatalogReadContext::new(&request, 882_110, &()).record_failure(
            SubnetCatalogRegistryRecordSubject::subnet_record(subnet),
            Some(882_000),
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
        let table_failure = CatalogReadContext::new(&request, 882_111, &()).record_failure(
            SubnetCatalogRegistryRecordSubject::legacy_routing_table(),
            None,
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
        let mut range_failure = range_failure;
        range_failure.registry_version = Some(882_111);
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
