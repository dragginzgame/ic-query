use super::*;
use proto::{CanisterIdRange, RoutingTableEntry};

pub(super) const SUBNET_A: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
pub(super) const SUBNET_B: &str = "aaaaa-aa";
pub(super) const CANISTER_A: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

pub(super) fn registry_fetch_request() -> MainnetRegistryFetchRequest {
    MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    }
}

pub(super) fn catalog_from_parts_for_test(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    subnet_list: SubnetListRecord,
    routing_table: RoutingTable,
    subnet_records: BTreeMap<String, SubnetRecord>,
) -> Result<RawSubnetCatalog, RegistryFetchError> {
    if subnet_list.subnets.is_empty() {
        return Err(RegistryFetchError::EmptySubnetList);
    }
    if routing_table.entries.is_empty() {
        return Err(RegistryFetchError::EmptyRoutingTable);
    }
    let subnets = subnet_list
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
    RawSubnetCatalog::new_mainnet_uncertified(
        UncertifiedCatalogCollection::new(
            registry_version,
            &request.endpoint,
            &request.fetched_at,
            &request.fetched_by,
            "test",
            3 + u64::try_from(subnet_records.len()).expect("fixture count fits u64"),
        ),
        subnets,
        routing_ranges_from_table(&routing_table)?,
    )
    .map_err(RegistryFetchError::from)
}

pub(super) fn subnet_list_record<const N: usize>(subnets: [&str; N]) -> SubnetListRecord {
    SubnetListRecord {
        subnets: subnets.iter().map(|subnet| principal_raw(subnet)).collect(),
    }
}

pub(super) fn subnet_record(subnet_type: SubnetType, members: usize) -> SubnetRecord {
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

pub(super) fn routing_table_record<const N: usize>(
    ranges: [(&str, &str, &str); N],
) -> RoutingTable {
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

pub(super) fn principal_raw(principal: &str) -> Vec<u8> {
    Principal::from_text(principal)
        .expect("principal")
        .as_slice()
        .to_vec()
}

pub(super) fn governance_node_provider(
    principal: &str,
    reward_account_hash: Option<Vec<u8>>,
) -> GovernanceNodeProvider {
    GovernanceNodeProvider {
        id: Some(Principal::from_text(principal).expect("principal")),
        reward_account: reward_account_hash.map(|hash| GovernanceAccountIdentifier { hash }),
    }
}

pub(super) fn node_record(node_operator: &str) -> NodeRecord {
    NodeRecord {
        node_operator_id: principal_raw(node_operator),
    }
}

pub(super) fn node_operator_record(node_provider: &str) -> NodeOperatorRecord {
    NodeOperatorRecord {
        node_operator_principal_id: Vec::new(),
        node_allowance: 0,
        node_provider_principal_id: principal_raw(node_provider),
        dc_id: "dc1".to_string(),
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
