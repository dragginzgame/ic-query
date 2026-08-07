use super::*;
use crate::{
    ic_registry::proto::{
        CanisterId, CanisterIdRange, PrincipalId, RoutingTable, RoutingTableEntry, SubnetId,
        SubnetListRecord, SubnetRecord, SubnetType,
    },
    nns::registry::{
        NNS_CERTIFIED_REGISTRY_DELTA_BATCH_SCHEMA_VERSION, NnsAuthenticatedRegistryDeltaBatch,
        NnsCertifiedRegistryDeltaLimits, NnsCertifiedRegistryDeltaSource,
        NnsCertifiedRegistryDeltaSourceFuture, NnsCertifiedRegistryDeltaVersion,
        NnsCertifiedRegistryPrecondition, NnsCertifiedRegistryValueEncoding,
        NnsRegistryCertification, NnsRegistryHostError, nns_certified_registry_delta_limits,
    },
    subnet_catalog::{
        CATALOG_SCHEMA_VERSION, CatalogAssurance, CatalogError, CatalogValidationContext,
        MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, SubnetKind, SubnetSpecialization,
        ValidatedSubnetCatalog, catalog_to_pretty_json, format_utc_timestamp_secs,
        parse_catalog_json,
    },
};
use candid::Principal;
use prost::Message;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

const NOW: u64 = 1_780_531_200;
const PROJECTION_SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
const PROJECTION_CANISTER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

mod archive_cases;
mod cache;
mod projection;
mod session;

#[derive(Default)]
struct BootstrapSource {
    requested_versions: Mutex<Vec<u64>>,
}

impl BootstrapSource {
    fn requested_versions(&self) -> Vec<u64> {
        self.requested_versions
            .lock()
            .expect("bootstrap fixture request lock")
            .clone()
    }
}

impl NnsCertifiedRegistryDeltaSource for BootstrapSource {
    fn fetch_certified_registry_delta_batch<'a>(
        &'a self,
        request: &'a NnsCertifiedRegistryDeltaBatchRequest,
    ) -> NnsCertifiedRegistryDeltaSourceFuture<'a> {
        self.requested_versions
            .lock()
            .expect("bootstrap fixture request lock")
            .push(request.requested_version);
        Box::pin(async move {
            match request.requested_version {
                0 => Ok(report_versions(
                    request,
                    3,
                    vec![
                        version(
                            1,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Upsert,
                                b"a",
                                Some(b"one"),
                            )],
                        ),
                        version(
                            2,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Update,
                                b"a",
                                Some(b"two"),
                            )],
                        ),
                    ],
                )),
                2 => Ok(report_versions(
                    request,
                    4,
                    vec![
                        version(
                            3,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Update,
                                b"a",
                                Some(b"three"),
                            )],
                        ),
                        version(
                            4,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Upsert,
                                b"future",
                                Some(b"ignored"),
                            )],
                        ),
                    ],
                )),
                version => Err(NnsRegistryHostError::InvalidSourceData {
                    reason: format!("unexpected bootstrap fixture request after version {version}"),
                }),
            }
        })
    }
}

#[derive(Clone, Copy)]
enum ArchiveRefreshMode {
    Advancing,
    Unchanged,
}

struct ArchiveRefreshSource {
    mode: ArchiveRefreshMode,
    lock_path: PathBuf,
    requested_versions: Mutex<Vec<u64>>,
}

impl ArchiveRefreshSource {
    fn new(mode: ArchiveRefreshMode, lock_path: PathBuf) -> Self {
        Self {
            mode,
            lock_path,
            requested_versions: Mutex::new(Vec::new()),
        }
    }

    fn requested_versions(&self) -> Vec<u64> {
        self.requested_versions
            .lock()
            .expect("archive refresh fixture request lock")
            .clone()
    }
}

impl NnsCertifiedRegistryDeltaSource for ArchiveRefreshSource {
    fn fetch_certified_registry_delta_batch<'a>(
        &'a self,
        request: &'a NnsCertifiedRegistryDeltaBatchRequest,
    ) -> NnsCertifiedRegistryDeltaSourceFuture<'a> {
        assert!(self.lock_path.is_file(), "source call must run under lock");
        self.requested_versions
            .lock()
            .expect("archive refresh fixture request lock")
            .push(request.requested_version);
        Box::pin(async move {
            match (self.mode, request.requested_version) {
                (ArchiveRefreshMode::Unchanged, version) => {
                    Ok(report_versions(request, version, Vec::new()))
                }
                (ArchiveRefreshMode::Advancing, 3) => Ok(report_versions(
                    request,
                    5,
                    vec![version(
                        4,
                        vec![mutation(
                            NnsCertifiedRegistryMutationKind::Update,
                            b"a",
                            Some(b"four"),
                        )],
                    )],
                )),
                (ArchiveRefreshMode::Advancing, 4) => Ok(report_versions(
                    request,
                    6,
                    vec![
                        version(
                            5,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Upsert,
                                b"b",
                                Some(b"five"),
                            )],
                        ),
                        version(
                            6,
                            vec![mutation(
                                NnsCertifiedRegistryMutationKind::Upsert,
                                b"future",
                                Some(b"ignored"),
                            )],
                        ),
                    ],
                )),
                (ArchiveRefreshMode::Advancing, requested_version) => {
                    Err(NnsRegistryHostError::InvalidSourceData {
                        reason: format!(
                            "unexpected archive refresh fixture request after version {requested_version}"
                        ),
                    })
                }
            }
        })
    }
}

fn bootstrap_request(
    network: &str,
    max_batches: u64,
    max_query_calls: u64,
    max_response_bytes: u64,
) -> NnsCertifiedRegistryBootstrapRequest {
    NnsCertifiedRegistryBootstrapRequest::new(
        network,
        "https://icp-api.io",
        NOW,
        NnsRegistryReplaySessionLimits::new(
            3,
            max_batches,
            max_query_calls,
            max_response_bytes,
            NnsRegistryReplayLimits::new(10, 100),
        ),
    )
}

struct ProvenanceFixture {
    limits: NnsRegistryReplaySessionLimits,
    first_request: NnsCertifiedRegistryDeltaBatchRequest,
    first: NnsCertifiedRegistryDeltaBatchReport,
    second_request: NnsCertifiedRegistryDeltaBatchRequest,
    second: NnsCertifiedRegistryDeltaBatchReport,
}

fn provenance_fixture() -> ProvenanceFixture {
    let limits =
        NnsRegistryReplaySessionLimits::new(2, 2, 2, 128, NnsRegistryReplayLimits::new(10, 100));
    let first_request = request(0);
    let first = report(
        &first_request,
        2,
        1,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"a",
            Some(b"one"),
        )],
        Vec::new(),
    );
    let mut second_request = request(1);
    second_request.source_endpoint = "https://example.com".to_string();
    let mut second = report(
        &second_request,
        2,
        2,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"b",
            Some(b"two"),
        )],
        Vec::new(),
    );
    second.certification.certificate_time_nanos = (NOW - 60) * 1_000_000_000;
    second.certification.certificate_time = format_utc_timestamp_secs(NOW - 60);
    ProvenanceFixture {
        limits,
        first_request,
        first,
        second_request,
        second,
    }
}

fn complete_provenance_session(changed_raw_evidence: bool) -> NnsRegistryReplaySession {
    let mut fixture = provenance_fixture();
    if changed_raw_evidence {
        fixture.second.certification.certificate_hex = "ce".repeat(8);
    }
    let mut session = NnsRegistryReplaySession::new(fixture.limits);
    session
        .apply_batch(&fixture.first_request, &fixture.first)
        .expect("first complete-session provenance batch");
    session
        .apply_batch(&fixture.second_request, &fixture.second)
        .expect("second complete-session provenance batch");
    session
}

fn projection_session() -> NnsRegistryReplaySession {
    NnsRegistryReplaySession::new(projection_session_limits())
}

const fn projection_session_limits() -> NnsRegistryReplaySessionLimits {
    NnsRegistryReplaySessionLimits::new(1, 1, 1, 64, NnsRegistryReplayLimits::new(20, 10_000))
}

fn complete_catalog_projection_session(
    include_subnet_record: bool,
    invalid_subnet_list: bool,
) -> NnsRegistryReplaySession {
    complete_catalog_projection_session_from_parts(
        include_subnet_record,
        invalid_subnet_list,
        projection_routing_table_with_range(PROJECTION_SUBNET),
    )
}

fn complete_catalog_projection_session_with_routing(
    routing_table: RoutingTable,
) -> NnsRegistryReplaySession {
    complete_catalog_projection_session_from_parts(true, false, routing_table)
}

fn complete_catalog_projection_session_from_parts(
    include_subnet_record: bool,
    invalid_subnet_list: bool,
    routing_table: RoutingTable,
) -> NnsRegistryReplaySession {
    let (request, report) = catalog_projection_report_from_parts(
        include_subnet_record,
        invalid_subnet_list,
        routing_table,
    );
    let mut session = projection_session();
    session
        .apply_batch(&request, &report)
        .expect("complete catalog fixture replay");
    session
}

fn catalog_projection_report_from_parts(
    include_subnet_record: bool,
    invalid_subnet_list: bool,
    routing_table: RoutingTable,
) -> (
    NnsCertifiedRegistryDeltaBatchRequest,
    NnsCertifiedRegistryDeltaBatchReport,
) {
    let mutations =
        catalog_projection_mutations(include_subnet_record, invalid_subnet_list, routing_table);
    let request = request(0);
    let report = report(&request, 1, 1, mutations, Vec::new());
    (request, report)
}

fn catalog_projection_mutations(
    include_subnet_record: bool,
    invalid_subnet_list: bool,
    routing_table: RoutingTable,
) -> Vec<NnsCertifiedRegistryMutation> {
    let subnet_list = SubnetListRecord {
        subnets: vec![principal_bytes(PROJECTION_SUBNET)],
    };
    let subnet_record = SubnetRecord {
        membership: vec![
            principal_bytes("aaaaa-aa"),
            principal_bytes(PROJECTION_CANISTER),
        ],
        subnet_type: SubnetType::Application as i32,
        canister_cycles_cost_schedule: 0,
    };
    let mut records = vec![
        (b"routing_table".as_slice(), routing_table.encode_to_vec()),
        (
            b"subnet_list".as_slice(),
            if invalid_subnet_list {
                vec![0xff]
            } else {
                subnet_list.encode_to_vec()
            },
        ),
    ];
    let subnet_record_key = format!("subnet_record_{PROJECTION_SUBNET}");
    if include_subnet_record {
        records.push((subnet_record_key.as_bytes(), subnet_record.encode_to_vec()));
    }
    records.sort_by(|left, right| left.0.cmp(right.0));
    records
        .into_iter()
        .map(|(key, value)| mutation(NnsCertifiedRegistryMutationKind::Upsert, key, Some(&value)))
        .collect()
}

fn complete_catalog_archive(root: &Path) -> NnsAuthenticatedRegistryArchive {
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let (_, report) = catalog_projection_report_from_parts(
        true,
        false,
        projection_routing_table_with_range(PROJECTION_SUBNET),
    );
    let batch = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&report);
    let storage_limits = NnsCertifiedRegistryArchiveStorageLimits::new(
        100_000,
        NnsCertifiedRegistryArchiveLimits::new(1, 100_000, 100_000),
    );
    let mut publisher = NnsCertifiedRegistryArchivePublisher::new(
        root,
        &archive_root,
        projection_session_limits(),
        storage_limits,
    );
    publisher
        .apply_batch(&batch)
        .expect("certified catalog archive object");
    publisher.finish().expect("certified catalog archive")
}

fn superseded_catalog_archive(root: &Path) -> NnsAuthenticatedRegistryArchive {
    let archive_root = root.join("nns/ic/registry-certified-v1");
    let first_request = request(0);
    let first_report = report(
        &first_request,
        2,
        1,
        catalog_projection_mutations(
            true,
            false,
            projection_routing_table_with_range(PROJECTION_SUBNET),
        ),
        Vec::new(),
    );
    let second_request = request(1);
    let second_report = report(
        &second_request,
        3,
        2,
        vec![mutation(
            NnsCertifiedRegistryMutationKind::Upsert,
            b"superseded_marker",
            Some(b"present"),
        )],
        Vec::new(),
    );
    let first = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&first_report);
    let second = NnsAuthenticatedRegistryDeltaBatch::from_validated_fixture(&second_report);
    let replay_limits =
        NnsRegistryReplaySessionLimits::new(2, 2, 2, 128, NnsRegistryReplayLimits::new(20, 10_000));
    let storage_limits = NnsCertifiedRegistryArchiveStorageLimits::new(
        200_000,
        NnsCertifiedRegistryArchiveLimits::new(2, 100_000, 200_000),
    );
    let mut publisher = NnsCertifiedRegistryArchivePublisher::new(
        root,
        &archive_root,
        replay_limits,
        storage_limits,
    );
    publisher.apply_batch(&first).expect("first archive batch");
    publisher
        .apply_batch(&second)
        .expect("second archive batch");
    publisher.finish().expect("superseded catalog archive")
}

fn certified_catalog_projection_request(
    now_unix_secs: u64,
    maximum_certificate_age_seconds: u64,
    version_policy: NnsCertifiedSubnetCatalogVersionPolicy,
) -> NnsCertifiedSubnetCatalogProjectionRequest {
    NnsCertifiedSubnetCatalogProjectionRequest::new(
        CatalogValidationContext::new(
            MAINNET_NETWORK,
            MAINNET_REGISTRY_CANISTER_ID,
            now_unix_secs,
            0,
        ),
        maximum_certificate_age_seconds,
        version_policy,
    )
}

fn projection_routing_table_with_range(subnet: &str) -> RoutingTable {
    RoutingTable {
        entries: vec![RoutingTableEntry {
            range: Some(CanisterIdRange {
                start_canister_id: Some(canister_id(PROJECTION_CANISTER)),
                end_canister_id: Some(canister_id(PROJECTION_CANISTER)),
            }),
            subnet_id: Some(SubnetId {
                principal_id: Some(PrincipalId {
                    raw: principal_bytes(subnet),
                }),
            }),
        }],
    }
}

fn canister_id(principal: &str) -> CanisterId {
    CanisterId {
        principal_id: Some(PrincipalId {
            raw: principal_bytes(principal),
        }),
    }
}

fn principal_bytes(principal: &str) -> Vec<u8> {
    Principal::from_text(principal)
        .expect("projection fixture principal")
        .as_slice()
        .to_vec()
}

fn request(requested_version: u64) -> NnsCertifiedRegistryDeltaBatchRequest {
    NnsCertifiedRegistryDeltaBatchRequest::new(
        MAINNET_NETWORK,
        "https://icp-api.io",
        requested_version,
        NOW,
    )
}

fn report(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    certified_latest_version: u64,
    version: u64,
    mutations: Vec<NnsCertifiedRegistryMutation>,
    preconditions: Vec<NnsCertifiedRegistryPrecondition>,
) -> NnsCertifiedRegistryDeltaBatchReport {
    report_versions(
        request,
        certified_latest_version,
        vec![NnsCertifiedRegistryDeltaVersion {
            version,
            timestamp_nanoseconds: NOW * 1_000_000_000,
            mutations,
            preconditions,
        }],
    )
}

fn report_versions(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
    certified_latest_version: u64,
    versions: Vec<NnsCertifiedRegistryDeltaVersion>,
) -> NnsCertifiedRegistryDeltaBatchReport {
    let inline_value_bytes = versions
        .iter()
        .flat_map(|version| &version.mutations)
        .filter_map(|mutation| mutation.value_hex.as_ref())
        .map(|value| value.len() / 2)
        .sum();
    let mutation_count = versions.iter().map(|version| version.mutations.len()).sum();
    let precondition_count = versions
        .iter()
        .map(|version| version.preconditions.len())
        .sum();
    let first_version = versions.first().map(|version| version.version);
    let last_version = versions.last().map(|version| version.version);
    NnsCertifiedRegistryDeltaBatchReport {
        schema_version: 3,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        requested_version: request.requested_version,
        certified_latest_version,
        first_version,
        last_version,
        version_count: versions.len(),
        mutation_count,
        precondition_count,
        inline_value_bytes,
        chunk_value_bytes: 0,
        value_bytes: inline_value_bytes,
        chunk_reference_count: 0,
        chunk_evidence_bytes: 0,
        more_available: last_version.unwrap_or(request.requested_version)
            < certified_latest_version,
        fetched_at: format_utc_timestamp_secs(NOW),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        query_call_count: 1,
        chunk_query_call_count: 0,
        certified_response_bytes: 64,
        chunk_response_bytes: 0,
        response_bytes: 64,
        limits: limits(),
        versions,
        chunk_evidence: Vec::new(),
        certification: NnsRegistryCertification {
            certificate_verified: true,
            certificate_time_nanos: NOW * 1_000_000_000,
            certificate_time: format_utc_timestamp_secs(NOW),
            root_key_digest: "ab".repeat(32),
            certificate_hex: "cd".repeat(8),
            certificate_bytes: 8,
            hash_tree_hex: "ef".repeat(4),
            hash_tree_bytes: 4,
        },
    }
}

fn version(
    version: u64,
    mutations: Vec<NnsCertifiedRegistryMutation>,
) -> NnsCertifiedRegistryDeltaVersion {
    NnsCertifiedRegistryDeltaVersion {
        version,
        timestamp_nanoseconds: NOW * 1_000_000_000,
        mutations,
        preconditions: Vec::new(),
    }
}

fn mutation(
    kind: NnsCertifiedRegistryMutationKind,
    key: &[u8],
    value: Option<&[u8]>,
) -> NnsCertifiedRegistryMutation {
    NnsCertifiedRegistryMutation {
        mutation_type: match kind {
            NnsCertifiedRegistryMutationKind::Insert => 0,
            NnsCertifiedRegistryMutationKind::Update => 1,
            NnsCertifiedRegistryMutationKind::Delete => 2,
            NnsCertifiedRegistryMutationKind::Upsert => 4,
        },
        mutation_kind: kind,
        key_hex: crate::hex::hex_bytes(key),
        value_encoding: if value.is_some() {
            NnsCertifiedRegistryValueEncoding::Inline
        } else {
            NnsCertifiedRegistryValueEncoding::Absent
        },
        chunk_sha256_hexes: Vec::new(),
        value_hex: value.map(crate::hex::hex_bytes),
    }
}

const fn limits() -> NnsCertifiedRegistryDeltaLimits {
    nns_certified_registry_delta_limits()
}
