use super::*;
use proto::{CanisterIdRange, PrincipalId, RoutingTableEntry};

const SUBNET_A: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
const SUBNET_B: &str = "aaaaa-aa";
const CANISTER_A: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[test]
fn registry_records_convert_to_catalog_domain_structs() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let subnet_records = BTreeMap::from([
        (
            SUBNET_A.to_string(),
            subnet_record(SubnetType::Application, 34),
        ),
        (SUBNET_B.to_string(), subnet_record(SubnetType::System, 13)),
    ]);
    let catalog = catalog_from_parts_for_test(
        &request,
        42,
        subnet_list_record([SUBNET_A, SUBNET_B]),
        routing_table_record([(CANISTER_A, CANISTER_A, SUBNET_A)]),
        subnet_records,
    )
    .expect("catalog");

    assert_eq!(catalog.registry_version, 42);
    assert_eq!(catalog.subnets.len(), 2);
    assert_eq!(catalog.routing_ranges.len(), 1);
    let fiduciary = catalog.subnet_by_principal(SUBNET_A).expect("fiduciary");
    assert_eq!(
        fiduciary.subnet_specialization,
        SubnetSpecialization::Fiduciary
    );
    assert_eq!(fiduciary.node_count, Some(34));
    assert!(fiduciary.charges_apply_by_default);
    let system = catalog.subnet_by_principal(SUBNET_B).expect("system");
    assert_eq!(system.subnet_kind, SubnetKind::System);
    assert!(!system.charges_apply_by_default);
}

#[test]
fn registry_records_preserve_cloud_engine_subnet_type() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let subnet_records = BTreeMap::from([(
        SUBNET_A.to_string(),
        subnet_record(SubnetType::CloudEngine, 13),
    )]);
    let catalog = catalog_from_parts_for_test(
        &request,
        42,
        subnet_list_record([SUBNET_A]),
        routing_table_record([(CANISTER_A, CANISTER_A, SUBNET_A)]),
        subnet_records,
    )
    .expect("catalog");

    let subnet = catalog.subnet_by_principal(SUBNET_A).expect("subnet");
    assert_eq!(subnet.subnet_kind, SubnetKind::CloudEngine);
    assert!(subnet.charges_apply_by_default);
}

#[test]
fn get_value_response_reports_large_value_chunk_keys() {
    let response = RegistryGetValueResponse {
        error: None,
        version: 1,
        content: Some(registry_get_value_response::Content::LargeValueChunkKeys(
            proto::LargeValueChunkKeys {
                chunk_content_sha256s: vec![vec![1], vec![2]],
            },
        )),
        timestamp_nanoseconds: 0,
    };

    let content = registry_value_content_from_response("routing_table", response)
        .expect("large value chunk keys");

    match content {
        RegistryValueContent::LargeValueChunkKeys(keys) => {
            assert_eq!(keys.chunk_content_sha256s, vec![vec![1], vec![2]]);
        }
        RegistryValueContent::Value(value) => {
            panic!("expected chunk keys, got inline value {value:?}");
        }
    }
}

#[test]
fn registry_get_chunk_request_candid_round_trips() {
    let request = RegistryGetChunkRequest {
        content_sha256: Some(vec![1, 2, 3]),
    };

    let bytes = Encode!(&request).expect("encode");
    let decoded = Decode!(&bytes, RegistryGetChunkRequest).expect("decode");

    assert_eq!(decoded, request);
}

#[test]
fn governance_node_provider_response_converts_to_domain_structs() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let node_counts = BTreeMap::from([("aaaaa-aa".to_string(), 2)]);
    let response = ListNodeProvidersResponse {
        node_providers: vec![
            governance_node_provider("ryjl3-tyaaa-aaaaa-aaaba-cai", None),
            governance_node_provider("aaaaa-aa", Some(vec![0xab, 0xcd])),
        ],
    };

    let list = node_provider_list_from_response(&request, response, node_counts, 42)
        .expect("node providers");

    assert_eq!(list.network, MAINNET_NETWORK);
    assert_eq!(list.governance_canister_id, MAINNET_GOVERNANCE_CANISTER_ID);
    assert_eq!(list.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(list.registry_version, 42);
    assert_eq!(list.node_providers.len(), 2);
    assert_eq!(list.node_providers[0].principal, "aaaaa-aa");
    assert_eq!(list.node_providers[0].node_count, Some(2));
    assert_eq!(
        list.node_providers[0].reward_account_hex.as_deref(),
        Some("abcd")
    );
    assert_eq!(
        list.node_providers[1].principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
    assert_eq!(list.node_providers[1].node_count, Some(0));
    assert_eq!(list.node_providers[1].reward_account_hex, None);
}

#[test]
fn node_provider_counts_follow_subnet_nodes_to_providers() {
    let provider_a = Principal::self_authenticating(b"provider-a").to_text();
    let provider_b = Principal::self_authenticating(b"provider-b").to_text();
    let operator_a = Principal::self_authenticating(b"operator-a").to_text();
    let operator_b = Principal::self_authenticating(b"operator-b").to_text();
    let node_a = Principal::self_authenticating(b"node-a").to_text();
    let node_b = Principal::self_authenticating(b"node-b").to_text();
    let node_c = Principal::self_authenticating(b"node-c").to_text();
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let subnet_records = BTreeMap::from([(
        subnet,
        SubnetRecord {
            membership: vec![
                principal_raw(&node_a),
                principal_raw(&node_b),
                principal_raw(&node_c),
            ],
            subnet_type: SubnetType::Application as i32,
            canister_cycles_cost_schedule: 0,
        },
    )]);
    let node_principals =
        assigned_node_principals_from_subnets(&subnet_records).expect("node principals");
    let node_records = BTreeMap::from([
        (node_a, node_record(&operator_a)),
        (node_b, node_record(&operator_a)),
        (node_c, node_record(&operator_b)),
    ]);
    let node_operator_records = BTreeMap::from([
        (operator_a, node_operator_record(&provider_a)),
        (operator_b, node_operator_record(&provider_b)),
    ]);

    let counts =
        node_provider_counts_from_records(&node_principals, &node_records, &node_operator_records)
            .expect("provider counts");

    assert_eq!(counts.get(&provider_a), Some(&2));
    assert_eq!(counts.get(&provider_b), Some(&1));
}

#[test]
fn node_operator_list_follows_assigned_nodes_to_operator_records() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let provider = Principal::self_authenticating(b"provider").to_text();
    let primary_operator = Principal::self_authenticating(b"operator-a").to_text();
    let secondary_operator = Principal::self_authenticating(b"operator-b").to_text();
    let node_a = Principal::self_authenticating(b"node-a").to_text();
    let node_b = Principal::self_authenticating(b"node-b").to_text();
    let node_c = Principal::self_authenticating(b"node-c").to_text();
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node_a.clone(), node_b.clone(), node_c.clone()]),
        node_records: BTreeMap::from([
            (node_a.clone(), node_record(&primary_operator)),
            (node_b.clone(), node_record(&primary_operator)),
            (node_c.clone(), node_record(&secondary_operator)),
        ]),
        node_operator_records: BTreeMap::from([
            (
                primary_operator.clone(),
                NodeOperatorRecord {
                    node_operator_principal_id: principal_raw(&primary_operator),
                    node_allowance: 4,
                    node_provider_principal_id: principal_raw(&provider),
                    dc_id: "dc-a".to_string(),
                },
            ),
            (
                secondary_operator.clone(),
                NodeOperatorRecord {
                    node_operator_principal_id: principal_raw(&secondary_operator),
                    node_allowance: 7,
                    node_provider_principal_id: principal_raw(&provider),
                    dc_id: "dc-b".to_string(),
                },
            ),
        ]),
        subnet_records: BTreeMap::from([(
            subnet,
            SubnetRecord {
                membership: vec![
                    principal_raw(&node_a),
                    principal_raw(&node_b),
                    principal_raw(&node_c),
                ],
                subnet_type: SubnetType::Application as i32,
                canister_cycles_cost_schedule: 0,
            },
        )]),
        data_center_records: BTreeMap::new(),
    };

    let list = node_operator_list_from_inventory(&request, inventory, 42).expect("node operators");

    assert_eq!(list.network, MAINNET_NETWORK);
    assert_eq!(list.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(list.registry_version, 42);
    assert_eq!(list.node_operators.len(), 2);
    let primary_result = list
        .node_operators
        .iter()
        .find(|operator| operator.principal == primary_operator)
        .expect("primary operator");
    assert_eq!(primary_result.node_provider_principal, provider);
    assert_eq!(primary_result.node_allowance, 4);
    assert_eq!(primary_result.data_center_id, "dc-a");
    assert_eq!(primary_result.node_count, Some(2));
    let secondary_result = list
        .node_operators
        .iter()
        .find(|operator| operator.principal == secondary_operator)
        .expect("secondary operator");
    assert_eq!(secondary_result.node_count, Some(1));
}

#[test]
fn node_list_follows_nodes_to_subnets_operators_and_providers() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let provider = Principal::self_authenticating(b"provider").to_text();
    let operator = Principal::self_authenticating(b"operator").to_text();
    let node = Principal::self_authenticating(b"node").to_text();
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node.clone()]),
        node_records: BTreeMap::from([(node.clone(), node_record(&operator))]),
        node_operator_records: BTreeMap::from([(
            operator.clone(),
            NodeOperatorRecord {
                node_operator_principal_id: principal_raw(&operator),
                node_allowance: 4,
                node_provider_principal_id: principal_raw(&provider),
                dc_id: "dc-a".to_string(),
            },
        )]),
        subnet_records: BTreeMap::from([(
            subnet.clone(),
            SubnetRecord {
                membership: vec![principal_raw(&node)],
                subnet_type: SubnetType::Application as i32,
                canister_cycles_cost_schedule: 0,
            },
        )]),
        data_center_records: BTreeMap::new(),
    };

    let list = node_list_from_inventory(&request, inventory, 42).expect("nodes");

    assert_eq!(list.registry_version, 42);
    assert_eq!(list.nodes.len(), 1);
    assert_eq!(list.nodes[0].principal, node);
    assert_eq!(list.nodes[0].node_operator_principal, operator);
    assert_eq!(list.nodes[0].node_provider_principal, provider);
    assert_eq!(list.nodes[0].subnet_principal, subnet);
    assert_eq!(list.nodes[0].subnet_kind, "application");
    assert_eq!(list.nodes[0].data_center_id, "dc-a");
}

#[test]
fn data_center_list_aggregates_registry_relations() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let provider_a = Principal::self_authenticating(b"provider-a").to_text();
    let provider_b = Principal::self_authenticating(b"provider-b").to_text();
    let operator_a = Principal::self_authenticating(b"operator-a").to_text();
    let operator_b = Principal::self_authenticating(b"operator-b").to_text();
    let node_a = Principal::self_authenticating(b"node-a").to_text();
    let node_b = Principal::self_authenticating(b"node-b").to_text();
    let node_c = Principal::self_authenticating(b"node-c").to_text();
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node_a.clone(), node_b.clone(), node_c.clone()]),
        node_records: BTreeMap::from([
            (node_a.clone(), node_record(&operator_a)),
            (node_b.clone(), node_record(&operator_a)),
            (node_c.clone(), node_record(&operator_b)),
        ]),
        node_operator_records: BTreeMap::from([
            (
                operator_a.clone(),
                NodeOperatorRecord {
                    node_operator_principal_id: principal_raw(&operator_a),
                    node_allowance: 4,
                    node_provider_principal_id: principal_raw(&provider_a),
                    dc_id: "dc-a".to_string(),
                },
            ),
            (
                operator_b.clone(),
                NodeOperatorRecord {
                    node_operator_principal_id: principal_raw(&operator_b),
                    node_allowance: 7,
                    node_provider_principal_id: principal_raw(&provider_b),
                    dc_id: "DC-A".to_string(),
                },
            ),
        ]),
        subnet_records: BTreeMap::from([(
            subnet,
            SubnetRecord {
                membership: vec![
                    principal_raw(&node_a),
                    principal_raw(&node_b),
                    principal_raw(&node_c),
                ],
                subnet_type: SubnetType::Application as i32,
                canister_cycles_cost_schedule: 0,
            },
        )]),
        data_center_records: BTreeMap::from([(
            "dc-a".to_string(),
            DataCenterRecord {
                id: "dc-a".to_string(),
                region: "eu-west".to_string(),
                owner: "example owner".to_string(),
                gps: Some(proto::Gps {
                    latitude: 48.8566,
                    longitude: 2.3522,
                }),
            },
        )]),
    };

    let list = data_center_list_from_inventory(&request, inventory, 42).expect("data centers");

    assert_eq!(list.registry_version, 42);
    assert_eq!(list.data_centers.len(), 1);
    assert_eq!(list.data_centers[0].id, "dc-a");
    assert_eq!(list.data_centers[0].region, "eu-west");
    assert_eq!(list.data_centers[0].owner, "example owner");
    assert_eq!(list.data_centers[0].latitude, Some(48.8566));
    assert_eq!(list.data_centers[0].longitude, Some(2.3522));
    assert_eq!(list.data_centers[0].node_operator_count, 2);
    assert_eq!(list.data_centers[0].node_provider_count, 2);
    assert_eq!(list.data_centers[0].node_count, 3);
}

#[test]
fn governance_node_provider_requires_principal() {
    let err = node_provider_from_governance(
        GovernanceNodeProvider {
            id: None,
            reward_account: None,
        },
        &BTreeMap::new(),
    )
    .expect_err("missing principal");

    assert!(matches!(
        err,
        RegistryFetchError::MissingField { field } if field == "node_provider.id"
    ));
}

#[test]
fn validated_chunk_append_concatenates_matching_chunks() {
    let first = b"hello ".to_vec();
    let second = b"world".to_vec();
    let first_hash = sha256_digest(&first);
    let second_hash = sha256_digest(&second);
    let mut value = Vec::new();

    append_validated_chunk(&mut value, &first_hash, first).expect("first chunk");
    append_validated_chunk(&mut value, &second_hash, second).expect("second chunk");

    assert_eq!(value, b"hello world");
}

#[test]
fn validated_chunk_append_rejects_hash_mismatch() {
    let expected = sha256_digest(b"expected");

    let err = append_validated_chunk(&mut Vec::new(), &expected, b"actual".to_vec())
        .expect_err("hash mismatch");

    assert!(matches!(
        err,
        RegistryFetchError::ChunkHashMismatch {
            sha256,
            actual_sha256
        } if sha256 == hex_bytes(&expected)
            && actual_sha256 == hex_bytes(&sha256_digest(b"actual"))
    ));
}

#[test]
fn get_value_response_reports_registry_errors() {
    let response = RegistryGetValueResponse {
        error: Some(proto::RegistryError {
            code: RegistryErrorCode::KeyNotPresent as i32,
            reason: "missing".to_string(),
            key: b"routing_table".to_vec(),
        }),
        version: 1,
        content: None,
        timestamp_nanoseconds: 0,
    };

    let err =
        registry_value_content_from_response("routing_table", response).expect_err("registry");

    assert!(matches!(
        err,
        RegistryFetchError::RegistryValue {
            key,
            code,
            reason
        } if key == "routing_table" && code == "key_not_present" && reason == "missing"
    ));
}

fn catalog_from_parts_for_test(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    subnet_list: SubnetListRecord,
    routing_table: RoutingTable,
    subnet_records: BTreeMap<String, SubnetRecord>,
) -> Result<SubnetCatalog, RegistryFetchError> {
    if subnet_list.subnets.is_empty() {
        return Err(RegistryFetchError::EmptySubnetList);
    }
    if routing_table.entries.is_empty() {
        return Err(RegistryFetchError::EmptyRoutingTable);
    }
    let mut subnets = subnet_list
        .subnets
        .iter()
        .map(|subnet_raw| {
            let subnet_principal = principal_text_from_raw(subnet_raw, "subnet_list.subnets")?;
            let record =
                subnet_records
                    .get(&subnet_principal)
                    .ok_or(RegistryFetchError::MissingField {
                        field: "subnet_record",
                    })?;
            Ok(subnet_info_from_record(&subnet_principal, record))
        })
        .collect::<Result<Vec<_>, RegistryFetchError>>()?;
    subnets.sort_by(|left, right| left.subnet_principal.cmp(&right.subnet_principal));
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
        routing_ranges: routing_ranges_from_table(&routing_table)?,
    };
    apply_mainnet_annotations(&mut catalog);
    catalog.validate()?;
    Ok(catalog)
}

fn subnet_list_record<const N: usize>(subnets: [&str; N]) -> SubnetListRecord {
    SubnetListRecord {
        subnets: subnets.iter().map(|subnet| principal_raw(subnet)).collect(),
    }
}

fn subnet_record(subnet_type: SubnetType, members: usize) -> SubnetRecord {
    SubnetRecord {
        membership: (0..members)
            .map(|index| {
                let index = u8::try_from(index).expect("fixture member index fits in u8");
                principal_raw(&Principal::self_authenticating([index]).to_text())
            })
            .collect(),
        subnet_type: subnet_type as i32,
        canister_cycles_cost_schedule: 0,
    }
}

fn routing_table_record<const N: usize>(ranges: [(&str, &str, &str); N]) -> RoutingTable {
    RoutingTable {
        entries: ranges
            .iter()
            .map(|(start, end, subnet)| RoutingTableEntry {
                range: Some(CanisterIdRange {
                    start_canister_id: Some(canister_id(start)),
                    end_canister_id: Some(canister_id(end)),
                }),
                subnet_id: Some(subnet_id(subnet)),
            })
            .collect(),
    }
}

fn canister_id(principal: &str) -> CanisterId {
    CanisterId {
        principal_id: Some(PrincipalId {
            raw: principal_raw(principal),
        }),
    }
}

fn subnet_id(principal: &str) -> SubnetId {
    SubnetId {
        principal_id: Some(PrincipalId {
            raw: principal_raw(principal),
        }),
    }
}

fn principal_raw(principal: &str) -> Vec<u8> {
    Principal::from_text(principal)
        .expect("principal")
        .as_slice()
        .to_vec()
}

fn governance_node_provider(
    principal: &str,
    reward_account_hash: Option<Vec<u8>>,
) -> GovernanceNodeProvider {
    GovernanceNodeProvider {
        id: Some(Principal::from_text(principal).expect("principal")),
        reward_account: reward_account_hash.map(|hash| GovernanceAccountIdentifier { hash }),
    }
}

fn node_record(node_operator: &str) -> NodeRecord {
    NodeRecord {
        node_operator_id: principal_raw(node_operator),
    }
}

fn node_operator_record(node_provider: &str) -> NodeOperatorRecord {
    NodeOperatorRecord {
        node_operator_principal_id: Vec::new(),
        node_allowance: 0,
        node_provider_principal_id: principal_raw(node_provider),
        dc_id: "dc1".to_string(),
    }
}
