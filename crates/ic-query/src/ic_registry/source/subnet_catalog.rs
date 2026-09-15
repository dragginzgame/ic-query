use super::agent::{mainnet_agent, mainnet_registry_canister};
use crate::{
    ic_registry::{
        CANISTER_RANGES_KEY_PREFIX, MainnetRegistryFetchRequest, RegistryFetchError,
        SubnetCatalogRegistryFailure,
        catalog::{
            CatalogRegistryReader, catalog_from_registry_records_detailed, get_catalog_record,
            record_evidence, record_failure,
        },
        proto::{RoutingTable, SubnetListRecord},
        routing_shards::{canister_range_start_from_key, validate_routing_table_shard_bounds},
        transport::{
            RegistryQueryCounter, decode_message, get_latest_version_counted,
            get_registry_key_family_counted, get_registry_versioned_value_counted,
        },
        wire::{RegistryVersionedValue, RegistryVersionedValueFailure},
    },
    subnet_catalog::{
        RawSubnetCatalog, SubnetCatalogField, SubnetCatalogRegistryRecordKind,
        SubnetCatalogRegistryRecordSubject, SubnetCatalogRoutingSource, SubnetCatalogSubject,
    },
};
use candid::Principal;

pub(in crate::ic_registry) async fn fetch_mainnet_subnet_catalog_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, RegistryFetchError> {
    fetch_mainnet_subnet_catalog_detailed_async(request)
        .await
        .map_err(|failure| failure.source)
}

pub(in crate::ic_registry) async fn fetch_mainnet_subnet_catalog_detailed_async(
    request: &MainnetRegistryFetchRequest,
) -> Result<RawSubnetCatalog, SubnetCatalogRegistryFailure> {
    let agent = mainnet_agent(request).map_err(|source| {
        SubnetCatalogRegistryFailure::new(
            None,
            Some(SubnetCatalogSubject::Endpoint(request.endpoint.clone())),
            source,
        )
    })?;
    let registry_canister = mainnet_registry_canister().map_err(|source| {
        SubnetCatalogRegistryFailure::new(
            None,
            Some(SubnetCatalogSubject::Field(
                SubnetCatalogField::RegistryCanister,
            )),
            source,
        )
    })?;
    let query_counter = RegistryQueryCounter::default();
    let registry_version = get_latest_version_counted(&agent, &registry_canister, &query_counter)
        .await
        .map_err(latest_version_failure)?;
    let reader = AgentCatalogRegistryReader {
        agent: &agent,
        registry_canister: &registry_canister,
        query_counter: &query_counter,
    };
    collect_pinned_catalog(request, registry_version, &reader).await
}

#[expect(
    clippy::too_many_lines,
    reason = "routing-source selection and its evidence stay in one auditable fail-closed sequence"
)]
async fn collect_pinned_catalog<R>(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    reader: &R,
) -> Result<RawSubnetCatalog, SubnetCatalogRegistryFailure>
where
    R: CatalogRegistryReader,
{
    let mut registry_records = Vec::new();

    let subnet_list_subject = SubnetCatalogRegistryRecordSubject::subnet_list();
    let subnet_list_value = get_catalog_record(
        reader,
        &subnet_list_subject,
        registry_version,
        request,
        &registry_records,
    )
    .await?;
    registry_records.push(record_evidence(
        request,
        registry_version,
        subnet_list_subject.clone(),
        &subnet_list_value,
    ));
    let subnet_list = decode_message::<SubnetListRecord>(
        subnet_list_subject.kind.protobuf_schema(),
        &subnet_list_value.value,
    )
    .map_err(|source| {
        record_failure(
            request,
            registry_version,
            subnet_list_subject,
            Some(subnet_list_value.version),
            registry_records.clone(),
            source,
        )
    })?;

    let shard_keys = reader
        .key_family(CANISTER_RANGES_KEY_PREFIX, registry_version)
        .await
        .map_err(|source| {
            record_failure(
                request,
                registry_version,
                SubnetCatalogRegistryRecordSubject::exact_keyed(
                    SubnetCatalogRegistryRecordKind::RoutingTable,
                    format!("{CANISTER_RANGES_KEY_PREFIX}*"),
                ),
                None,
                registry_records.clone(),
                source,
            )
        })?;

    if shard_keys.is_empty() {
        return Err(record_failure(
            request,
            registry_version,
            SubnetCatalogRegistryRecordSubject::exact_keyed(
                SubnetCatalogRegistryRecordKind::RoutingTable,
                format!("{CANISTER_RANGES_KEY_PREFIX}*"),
            ),
            None,
            registry_records,
            RegistryFetchError::InvalidRegistryKeyFamily {
                reason: format!(
                    "pinned mainnet Registry version {registry_version} has no authoritative canister-ranges shards"
                ),
            },
        ));
    }

    let mut shard_subjects = Vec::with_capacity(shard_keys.len());
    for key in shard_keys {
        let range_start =
            canister_range_start_from_key(CANISTER_RANGES_KEY_PREFIX, &key).map_err(|reason| {
                record_failure(
                    request,
                    registry_version,
                    SubnetCatalogRegistryRecordSubject::exact_keyed(
                        SubnetCatalogRegistryRecordKind::RoutingTable,
                        &key,
                    ),
                    None,
                    registry_records.clone(),
                    RegistryFetchError::InvalidRegistryKeyFamily { reason },
                )
            })?;
        shard_subjects.push(SubnetCatalogRegistryRecordSubject::canister_ranges(
            range_start,
        ));
    }
    let mut routing_table = RoutingTable::default();
    for (index, subject) in shard_subjects.iter().enumerate() {
        let value = get_catalog_record(
            reader,
            subject,
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
        let shard = decode_message::<RoutingTable>(subject.kind.protobuf_schema(), &value.value)
            .map_err(|source| {
                record_failure(
                    request,
                    registry_version,
                    subject.clone(),
                    Some(value.version),
                    registry_records.clone(),
                    source,
                )
            })?;
        let next_start = shard_subjects
            .get(index + 1)
            .and_then(|next| next.canister_range_start.as_ref());
        validate_routing_table_shard_bounds(
            &subject.key,
            subject
                .canister_range_start
                .as_ref()
                .expect("canister-ranges subject has a range start"),
            &shard,
            next_start,
        )
        .map_err(|reason| {
            record_failure(
                request,
                registry_version,
                subject.clone(),
                Some(value.version),
                registry_records.clone(),
                RegistryFetchError::InvalidRegistryKeyFamily { reason },
            )
        })?;
        routing_table.entries.extend(shard.entries);
    }
    let routing_source = SubnetCatalogRoutingSource::CanisterRanges;

    catalog_from_registry_records_detailed(
        request,
        registry_version,
        reader,
        subnet_list,
        routing_table,
        routing_source,
        registry_records,
    )
    .await
}

struct AgentCatalogRegistryReader<'a> {
    agent: &'a ic_agent::Agent,
    registry_canister: &'a Principal,
    query_counter: &'a RegistryQueryCounter,
}

impl CatalogRegistryReader for AgentCatalogRegistryReader<'_> {
    async fn key_family(
        &self,
        prefix: &str,
        registry_version: u64,
    ) -> Result<Vec<String>, RegistryFetchError> {
        get_registry_key_family_counted(
            self.agent,
            self.registry_canister,
            prefix,
            registry_version,
            self.query_counter,
        )
        .await
    }

    async fn value(
        &self,
        key: &str,
        registry_version: u64,
    ) -> Result<RegistryVersionedValue, RegistryVersionedValueFailure> {
        get_registry_versioned_value_counted(
            self.agent,
            self.registry_canister,
            key,
            registry_version,
            self.query_counter,
        )
        .await
    }

    fn query_call_count(&self) -> u64 {
        self.query_counter.call_count()
    }
}

const fn latest_version_failure(source: RegistryFetchError) -> SubnetCatalogRegistryFailure {
    SubnetCatalogRegistryFailure::new(
        None,
        Some(SubnetCatalogSubject::RegistryLatestVersion),
        source,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ic_registry::{
        ROUTING_TABLE_KEY, SUBNET_LIST_KEY,
        proto::{
            CanisterId, CanisterIdRange, PrincipalId, RoutingTableEntry, SubnetId, SubnetRecord,
        },
        wire::RegistryValueEncoding,
    };
    use crate::subnet_catalog::{CatalogAssurance, SubnetCatalogRegistryValueEncoding};
    use prost::Message;
    use std::{collections::BTreeMap, sync::Mutex};

    const ACTIVE_SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
    const DELETED_SUBNET: &str = "mkbc3-fzim5-s5pye-pbnzo-uj5yv-raphe-ceecn-ejd6g-5poxm-dzuot-iae";
    const PINNED_VERSION: u64 = 63_438;

    #[derive(Clone)]
    struct FixtureValue {
        value: Vec<u8>,
        version: u64,
        timestamp_nanoseconds: u64,
        encoding: RegistryValueEncoding,
    }

    #[derive(Default)]
    struct FixtureReader {
        keys: Vec<String>,
        values: BTreeMap<String, FixtureValue>,
        failures: BTreeMap<String, u64>,
        family_reads: Mutex<Vec<(String, u64)>>,
        value_reads: Mutex<Vec<(String, u64)>>,
    }

    impl FixtureReader {
        fn insert_message<M: Message>(
            &mut self,
            key: impl Into<String>,
            message: &M,
            version: u64,
            encoding: RegistryValueEncoding,
        ) {
            self.values.insert(
                key.into(),
                FixtureValue {
                    value: message.encode_to_vec(),
                    version,
                    timestamp_nanoseconds: version * 1_000,
                    encoding,
                },
            );
        }
    }

    impl CatalogRegistryReader for FixtureReader {
        async fn key_family(
            &self,
            prefix: &str,
            registry_version: u64,
        ) -> Result<Vec<String>, RegistryFetchError> {
            self.family_reads
                .lock()
                .expect("fixture family reads lock")
                .push((prefix.to_string(), registry_version));
            Ok(self.keys.clone())
        }

        async fn value(
            &self,
            key: &str,
            registry_version: u64,
        ) -> Result<RegistryVersionedValue, RegistryVersionedValueFailure> {
            self.value_reads
                .lock()
                .expect("fixture value reads lock")
                .push((key.to_string(), registry_version));
            if let Some(returned_version) = self.failures.get(key) {
                return Err(RegistryVersionedValueFailure {
                    source: RegistryFetchError::MissingValue {
                        key: key.to_string(),
                    },
                    returned_version: Some(*returned_version),
                });
            }
            let value = self
                .values
                .get(key)
                .ok_or_else(|| RegistryVersionedValueFailure {
                    source: RegistryFetchError::MissingValue {
                        key: key.to_string(),
                    },
                    returned_version: None,
                })?;
            Ok(RegistryVersionedValue {
                value: value.value.clone(),
                version: value.version,
                timestamp_nanoseconds: value.timestamp_nanoseconds,
                encoding: value.encoding,
            })
        }

        fn query_call_count(&self) -> u64 {
            1 + u64::try_from(
                self.family_reads
                    .lock()
                    .expect("fixture family reads lock")
                    .len(),
            )
            .expect("fixture call count")
                + u64::try_from(
                    self.value_reads
                        .lock()
                        .expect("fixture value reads lock")
                        .len(),
                )
                .expect("fixture call count")
        }
    }

    fn request() -> MainnetRegistryFetchRequest {
        MainnetRegistryFetchRequest {
            endpoint: "https://icp-api.io".to_string(),
            fetched_at: "2026-08-20T00:00:00Z".to_string(),
            fetched_by: "test".to_string(),
        }
    }

    fn principal_id(principal: Principal) -> PrincipalId {
        PrincipalId {
            raw: principal.as_slice().to_vec(),
        }
    }

    fn canister_id(number: u64) -> Principal {
        let mut bytes = number.to_be_bytes().to_vec();
        bytes.extend_from_slice(&[1, 1]);
        Principal::from_slice(&bytes)
    }

    fn routing_table(start: Principal, end: Principal, subnet: Principal) -> RoutingTable {
        RoutingTable {
            entries: vec![RoutingTableEntry {
                range: Some(CanisterIdRange {
                    start_canister_id: Some(CanisterId {
                        principal_id: Some(principal_id(start)),
                    }),
                    end_canister_id: Some(CanisterId {
                        principal_id: Some(principal_id(end)),
                    }),
                }),
                subnet_id: Some(SubnetId {
                    principal_id: Some(principal_id(subnet)),
                }),
            }],
        }
    }

    fn shard_key(start: Principal) -> String {
        format!(
            "{CANISTER_RANGES_KEY_PREFIX}{}",
            crate::hex::hex_bytes(start.as_slice())
        )
    }

    fn base_reader() -> (FixtureReader, Principal, Principal, String) {
        let active = Principal::from_text(ACTIVE_SUBNET).expect("active subnet");
        let deleted = Principal::from_text(DELETED_SUBNET).expect("deleted subnet");
        let start = canister_id(1);
        let end = canister_id(9);
        let key = shard_key(start);
        let mut reader = FixtureReader {
            keys: vec![key.clone()],
            ..FixtureReader::default()
        };
        reader.insert_message(
            SUBNET_LIST_KEY,
            &SubnetListRecord {
                subnets: vec![active.as_slice().to_vec()],
            },
            63_348,
            RegistryValueEncoding::Inline,
        );
        reader.insert_message(
            &key,
            &routing_table(start, end, active),
            63_420,
            RegistryValueEncoding::Chunked,
        );
        reader.insert_message(
            crate::ic_registry::subnet_record_key(ACTIVE_SUBNET),
            &SubnetRecord {
                membership: Vec::new(),
                subnet_type: 1,
                canister_cycles_cost_schedule: 1,
            },
            63_300,
            RegistryValueEncoding::Inline,
        );
        reader.insert_message(
            ROUTING_TABLE_KEY,
            &routing_table(start, end, deleted),
            51_979,
            RegistryValueEncoding::Inline,
        );
        (reader, start, end, key)
    }

    fn shard(start: Principal) -> RoutingTable {
        RoutingTable {
            entries: vec![RoutingTableEntry {
                range: Some(CanisterIdRange {
                    start_canister_id: Some(CanisterId {
                        principal_id: Some(PrincipalId {
                            raw: start.as_slice().to_vec(),
                        }),
                    }),
                    end_canister_id: Some(CanisterId {
                        principal_id: Some(PrincipalId {
                            raw: start.as_slice().to_vec(),
                        }),
                    }),
                }),
                subnet_id: None,
            }],
        }
    }

    #[test]
    fn canister_ranges_key_round_trips_its_range_start() {
        let start = canister_id(1);
        let key = format!(
            "{CANISTER_RANGES_KEY_PREFIX}{}",
            crate::hex::hex_bytes(start.as_slice())
        );
        assert_eq!(
            canister_range_start_from_key(CANISTER_RANGES_KEY_PREFIX, &key).expect("range start"),
            start
        );
    }

    #[test]
    fn shard_must_be_nonempty_and_stay_within_its_key_interval() {
        let start = canister_id(1);
        let next = canister_id(5);
        let below = canister_id(0);
        let inside = canister_id(2);
        let subject = SubnetCatalogRegistryRecordSubject::canister_ranges(start);

        let lower_bound = subject
            .canister_range_start
            .as_ref()
            .expect("range-start subject");
        validate_routing_table_shard_bounds(&subject.key, lower_bound, &shard(start), Some(&next))
            .expect("matching shard");
        validate_routing_table_shard_bounds(&subject.key, lower_bound, &shard(inside), Some(&next))
            .expect("range may start above its shard key");
        assert!(
            validate_routing_table_shard_bounds(
                &subject.key,
                lower_bound,
                &shard(below),
                Some(&next)
            )
            .is_err()
        );
        assert!(
            validate_routing_table_shard_bounds(
                &subject.key,
                lower_bound,
                &shard(next),
                Some(&next)
            )
            .is_err()
        );
        assert!(
            validate_routing_table_shard_bounds(
                &subject.key,
                lower_bound,
                &RoutingTable::default(),
                Some(&next)
            )
            .is_err()
        );
    }

    #[test]
    fn modern_shards_override_stale_legacy_routing_without_resurrecting_deleted_subnet() {
        let (reader, _, _, shard_key) = base_reader();

        let catalog = futures::executor::block_on(collect_pinned_catalog(
            &request(),
            PINNED_VERSION,
            &reader,
        ))
        .expect("coherent modern catalog");

        assert_eq!(
            catalog.provenance.routing_source,
            SubnetCatalogRoutingSource::CanisterRanges
        );
        assert_eq!(catalog.subnets.len(), 1);
        assert_eq!(catalog.subnets[0].subnet_principal, ACTIVE_SUBNET);
        assert!(
            catalog
                .routing_ranges
                .iter()
                .all(|range| range.subnet_principal != DELETED_SUBNET)
        );
        let value_reads = reader.value_reads.lock().expect("fixture value reads lock");
        assert!(
            value_reads
                .iter()
                .all(|(_, version)| *version == PINNED_VERSION)
        );
        assert!(!value_reads.iter().any(|(key, _)| key == ROUTING_TABLE_KEY));
        drop(value_reads);
        assert_eq!(
            reader
                .family_reads
                .lock()
                .expect("fixture family reads lock")
                .as_slice(),
            &[(CANISTER_RANGES_KEY_PREFIX.to_string(), PINNED_VERSION)]
        );
        let shard_evidence = catalog
            .provenance
            .registry_records
            .iter()
            .find(|evidence| evidence.record.key == shard_key)
            .expect("shard evidence");
        assert_eq!(shard_evidence.requested_registry_version, PINNED_VERSION);
        assert_eq!(shard_evidence.returned_registry_version, 63_420);
        assert_eq!(
            shard_evidence.value_encoding,
            SubnetCatalogRegistryValueEncoding::Chunked
        );
        assert!(catalog.provenance.registry_records.iter().any(|evidence| {
            evidence.record.kind == SubnetCatalogRegistryRecordKind::SubnetList
                && evidence.returned_registry_version == 63_348
        }));
    }

    #[test]
    fn empty_modern_family_fails_closed_without_reading_legacy_routing() {
        let (mut reader, _, _, _) = base_reader();
        reader.keys.clear();

        let failure = futures::executor::block_on(collect_pinned_catalog(
            &request(),
            PINNED_VERSION,
            &reader,
        ))
        .expect_err("live mainnet requires modern routing authority");

        assert_eq!(failure.registry_version, Some(PINNED_VERSION));
        assert!(matches!(
            failure.source,
            RegistryFetchError::InvalidRegistryKeyFamily { .. }
        ));
        assert!(matches!(
            failure.subject,
            Some(SubnetCatalogSubject::RegistryRecord(
                SubnetCatalogRegistryRecordSubject { key, .. }
            )) if key == format!("{CANISTER_RANGES_KEY_PREFIX}*")
        ));
        assert_eq!(failure.registry_records.len(), 1);
        assert_eq!(
            failure.registry_records[0].record.kind,
            SubnetCatalogRegistryRecordKind::SubnetList
        );
        assert!(
            !reader
                .value_reads
                .lock()
                .expect("fixture value reads lock")
                .iter()
                .any(|(key, _)| key == ROUTING_TABLE_KEY)
        );
    }

    #[test]
    fn missing_authoritative_shard_fails_with_pinned_and_returned_versions() {
        let (mut reader, _, _, _) = base_reader();
        let missing_start = canister_id(10);
        let missing_key = shard_key(missing_start);
        reader.keys.push(missing_key.clone());
        reader.failures.insert(missing_key.clone(), 63_410);

        let failure = futures::executor::block_on(collect_pinned_catalog(
            &request(),
            PINNED_VERSION,
            &reader,
        ))
        .expect_err("missing shard must fail closed");

        assert_eq!(failure.registry_version, Some(PINNED_VERSION));
        assert_eq!(failure.returned_registry_value_version, Some(63_410));
        assert_eq!(
            failure.source_endpoint.as_deref(),
            Some("https://icp-api.io")
        );
        assert_eq!(failure.assurance, Some(CatalogAssurance::UncertifiedQuery));
        assert!(matches!(
            failure.subject,
            Some(SubnetCatalogSubject::RegistryRecord(
                SubnetCatalogRegistryRecordSubject { key, .. }
            )) if key == missing_key
        ));
        assert!(failure.registry_records.iter().all(|evidence| {
            evidence.requested_registry_version == PINNED_VERSION
                && evidence.returned_registry_version <= PINNED_VERSION
        }));
        assert!(
            !reader
                .value_reads
                .lock()
                .expect("fixture value reads lock")
                .iter()
                .any(|(key, _)| key == ROUTING_TABLE_KEY)
        );
    }

    #[test]
    fn overlapping_authoritative_shards_fail_closed() {
        let (mut reader, _, _, _) = base_reader();
        let active = Principal::from_text(ACTIVE_SUBNET).expect("active subnet");
        let second_start = canister_id(5);
        let second_end = canister_id(12);
        let second_key = shard_key(second_start);
        reader.keys.push(second_key.clone());
        reader.insert_message(
            second_key,
            &routing_table(second_start, second_end, active),
            63_421,
            RegistryValueEncoding::Inline,
        );

        let failure = futures::executor::block_on(collect_pinned_catalog(
            &request(),
            PINNED_VERSION,
            &reader,
        ))
        .expect_err("overlapping shards must fail closed");

        assert!(matches!(
            failure.source,
            RegistryFetchError::InvalidRegistryKeyFamily { .. }
                | RegistryFetchError::Catalog(
                    crate::subnet_catalog::CatalogError::OverlappingRoutingRanges { .. }
                )
        ));
        assert!(
            !reader
                .value_reads
                .lock()
                .expect("fixture value reads lock")
                .iter()
                .any(|(key, _)| key == ROUTING_TABLE_KEY)
        );
    }
}
