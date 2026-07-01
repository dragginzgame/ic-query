use super::{
    NnsNodeOperatorCacheRequest, NnsNodeOperatorHostError, NnsNodeOperatorListReport,
    NnsNodeOperatorListRequest, NnsNodeOperatorRow, NnsNodeOperatorSource,
    NnsNodeOperatorSourceRequest, build_nns_node_operator_list_report_with_source,
    nns_node_operator_list_report_text, resolve_node_operator,
};
use crate::ic_registry::MainnetNodeOperator;
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID};
use crate::test_support::temp_dir;

#[test]
fn node_operator_report_uses_live_registry_source() {
    let request = NnsNodeOperatorListRequest {
        cache: test_cache_request(MAINNET_NETWORK, "uses-live-source"),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
    };
    let report = build_nns_node_operator_list_report_with_source(
        &request,
        &FixtureNodeOperatorSource {
            node_operators: vec![MainnetNodeOperator {
                principal: "aaaaa-aa".to_string(),
                node_provider_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
                node_allowance: 4,
                data_center_id: "dc1".to_string(),
                node_count: Some(3),
            }],
        },
    )
    .expect("node operator report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(report.registry_version, 42);
    assert_eq!(report.fetched_at, "2026-06-04T00:00:00Z");
    assert_eq!(report.node_operator_count, 1);
    assert_eq!(report.node_operators[0].node_operator_principal, "aaaaa-aa");
    assert_eq!(
        report.node_operators[0].node_provider_principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
    assert_eq!(report.node_operators[0].node_allowance, 4);
    assert_eq!(report.node_operators[0].data_center_id, "dc1");
    assert_eq!(report.node_operators[0].node_count, Some(3));
}

#[test]
fn node_operator_text_keeps_compact_principals() {
    let report = node_operator_report_fixture();

    let text = nns_node_operator_list_report_text(&report);

    assert!(text.contains("node_operators: ic count 1"));
    assert!(text.contains("NODE_OPERATOR"));
    assert!(text.contains("ryjl3"));
    assert!(text.contains("aaaaa"));
    assert!(text.contains("13"));
    assert!(!text.contains("ryjl3-tyaaa-aaaaa-aaaba-cai"));
}

#[test]
fn node_operator_info_resolves_unique_prefix() {
    let report = node_operator_report_fixture();

    let (operator, resolved_from) =
        resolve_node_operator(&report, "ryjl").expect("prefix resolves");

    assert_eq!(resolved_from, "node_operator_principal_prefix");
    assert_eq!(
        operator.node_operator_principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
}

fn node_operator_report_fixture() -> NnsNodeOperatorListReport {
    NnsNodeOperatorListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 42,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        node_operator_count: 1,
        node_operators: vec![NnsNodeOperatorRow {
            node_operator_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
            node_provider_principal: "aaaaa-aa".to_string(),
            node_allowance: 7,
            data_center_id: "dc1".to_string(),
            node_count: Some(13),
        }],
    }
}

fn test_cache_request(network: &str, name: &str) -> NnsNodeOperatorCacheRequest {
    NnsNodeOperatorCacheRequest {
        icp_root: temp_dir(&format!("ic-query-nns-node-operator-{name}")),
        network: network.to_string(),
    }
}

struct FixtureNodeOperatorSource {
    node_operators: Vec<MainnetNodeOperator>,
}

impl NnsNodeOperatorSource for FixtureNodeOperatorSource {
    fn fetch_node_operator_list_report(
        &self,
        request: &NnsNodeOperatorSourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
        let node_operators = self
            .node_operators
            .iter()
            .map(|operator| NnsNodeOperatorRow {
                node_operator_principal: operator.principal.clone(),
                node_provider_principal: operator.node_provider_principal.clone(),
                node_allowance: operator.node_allowance,
                data_center_id: operator.data_center_id.clone(),
                node_count: operator.node_count,
            })
            .collect::<Vec<_>>();
        Ok(NnsNodeOperatorListReport {
            schema_version: 1,
            network: MAINNET_NETWORK.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            node_operator_count: node_operators.len(),
            node_operators,
        })
    }
}
