use super::{
    NnsInventoryCacheRequest, NnsNodeHostError, NnsNodeListFilters, NnsNodeListReport,
    NnsNodeListRequest, NnsNodeRow, NnsNodeSource, build_nns_node_list_report_with_source,
    cache::load_cached_nns_node_report, filter_node_list_report, nns_node_cache_path,
    nns_node_list_report_text, nns_node_list_report_verbose_text, resolve_node,
};
use crate::ic_registry::MainnetNode;
use crate::nns::{LiveNnsSource, NnsSourceRequest};
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, SubnetKind};
use crate::test_support::temp_dir;
use std::fs;

#[test]
fn live_node_source_rejects_non_mainnet_before_agent_construction() {
    let request = NnsSourceRequest::new(
        "local",
        "not a valid replica endpoint",
        "2026-07-29T00:00:00Z",
        "test",
    );

    let error = LiveNnsSource
        .fetch_node_list_report(&request)
        .expect_err("unsupported network");

    assert!(matches!(
        error,
        NnsNodeHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn node_report_uses_live_registry_source() {
    let request = NnsNodeListRequest {
        cache: test_cache_request(MAINNET_NETWORK, "uses-live-source"),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
        filters: NnsNodeListFilters::default(),
    };
    let report = build_nns_node_list_report_with_source(
        &request,
        &FixtureNodeSource {
            nodes: vec![node_fixture()],
        },
    )
    .expect("node report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(report.registry_version, 42);
    assert_eq!(report.fetched_at, "2026-06-04T00:00:00Z");
    assert_eq!(report.node_count, 1);
    assert_eq!(
        report.nodes[0].node_principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
}

#[test]
fn node_report_refreshes_invalid_cache_but_cache_only_load_remains_strict() {
    let cache = test_cache_request(MAINNET_NETWORK, "invalid-cache-refresh");
    let path = nns_node_cache_path(&cache.cache_root, &cache.network);
    let mut invalid = node_report_fixture();
    invalid.node_count = 2;
    let invalid_json = serde_json::to_string_pretty(&invalid).expect("serialize invalid cache");
    crate::cache_file::write_managed_text_atomically(&cache.cache_root, &path, &invalid_json)
        .expect("write invalid cache");
    let request = NnsNodeListRequest {
        cache: cache.clone(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
        filters: NnsNodeListFilters::default(),
    };

    let error = load_cached_nns_node_report(&cache).expect_err("cache-only load is strict");
    assert!(matches!(
        error,
        NnsNodeHostError::Cache(crate::HostCacheError::InvalidCache { .. })
    ));

    let report = build_nns_node_list_report_with_source(
        &request,
        &FixtureNodeSource {
            nodes: vec![node_fixture()],
        },
    )
    .expect("invalid cache refreshes");

    assert_eq!(report.registry_version, 42);
    assert_ne!(
        fs::read_to_string(path).expect("refreshed cache"),
        invalid_json
    );
    let _ = fs::remove_dir_all(cache.cache_root);
}

#[test]
fn invalid_node_source_does_not_replace_existing_invalid_cache() {
    let cache = test_cache_request(MAINNET_NETWORK, "invalid-source-preserves-cache");
    let path = nns_node_cache_path(&cache.cache_root, &cache.network);
    crate::cache_file::write_managed_text_atomically(&cache.cache_root, &path, "not-json")
        .expect("write invalid cache");
    let request = NnsNodeListRequest {
        cache: cache.clone(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
        filters: NnsNodeListFilters::default(),
    };

    let error = build_nns_node_list_report_with_source(&request, &InvalidNodeSource)
        .expect_err("invalid source is rejected before publication");

    assert!(
        matches!(error, NnsNodeHostError::InvalidSourceData { reason } if reason.contains("node_count"))
    );
    assert_eq!(
        fs::read_to_string(path).expect("preserved invalid cache"),
        "not-json"
    );
    let _ = fs::remove_dir_all(cache.cache_root);
}

#[test]
fn node_text_keeps_compact_principals() {
    let report = node_report_fixture();

    let text = nns_node_list_report_text(&report);
    let verbose_text = nns_node_list_report_verbose_text(&report);

    assert!(text.contains("nodes: ic count 1"));
    assert!(text.contains("NODE"));
    assert!(text.contains("ryjl3"));
    assert!(!text.contains("ryjl3-tyaaa-aaaaa-aaaba-cai"));
    assert!(text.contains("fetched_at 2026-06-04T00:00:00Z\n\nNODE"));
    assert!(verbose_text.contains("fetched_by: test\n\nNODE"));
}

#[test]
fn node_report_subnet_kind_keeps_the_existing_cache_label() {
    let report = node_report_fixture();
    let json = serde_json::to_value(&report).expect("serialize node report");

    assert_eq!(json["nodes"][0]["subnet_kind"], "application");

    let decoded: NnsNodeListReport = serde_json::from_value(json).expect("deserialize node report");
    assert_eq!(decoded.nodes[0].subnet_kind, SubnetKind::Application);
}

#[test]
fn node_info_resolves_unique_prefix() {
    let report = node_report_fixture();

    let (node, resolved_from) = resolve_node(&report, "ryjl").expect("prefix resolves");

    assert_eq!(resolved_from, "node_principal_prefix");
    assert_eq!(node.node_principal, "ryjl3-tyaaa-aaaaa-aaaba-cai");
}

#[test]
fn node_list_filters_by_related_prefixes() {
    let mut report = node_report_fixture();
    report.node_count = 2;
    report.nodes.push(NnsNodeRow {
        node_principal: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        node_operator_principal: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        node_provider_principal: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        subnet_principal: "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe"
            .to_string(),
        subnet_kind: SubnetKind::System,
        data_center_id: "dc2".to_string(),
    });

    let filtered = filter_node_list_report(
        report,
        &NnsNodeListFilters {
            subnet: Some("pzp6e".to_string()),
            subnet_kind: Some(SubnetKind::Application),
            data_center: Some("dc".to_string()),
            node_provider: Some("rwlgt".to_string()),
            node_operator: Some("aaaaa-aa".to_string()),
        },
    );

    assert_eq!(filtered.node_count, 1);
    assert_eq!(
        filtered.nodes[0].node_principal,
        "ryjl3-tyaaa-aaaaa-aaaba-cai"
    );
}

fn node_report_fixture() -> NnsNodeListReport {
    NnsNodeListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 42,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "test".to_string(),
        node_count: 1,
        nodes: vec![NnsNodeRow {
            node_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
            node_operator_principal: "aaaaa-aa".to_string(),
            node_provider_principal: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
            subnet_principal: "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae"
                .to_string(),
            subnet_kind: SubnetKind::Application,
            data_center_id: "dc1".to_string(),
        }],
    }
}

fn node_fixture() -> MainnetNode {
    MainnetNode {
        principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        node_operator_principal: "aaaaa-aa".to_string(),
        node_provider_principal: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        subnet_principal: "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae"
            .to_string(),
        subnet_kind: SubnetKind::Application,
        data_center_id: "dc1".to_string(),
    }
}

fn test_cache_request(network: &str, name: &str) -> NnsInventoryCacheRequest {
    NnsInventoryCacheRequest {
        cache_root: temp_dir(&format!("ic-query-nns-node-{name}")),
        network: network.to_string(),
    }
}

struct FixtureNodeSource {
    nodes: Vec<MainnetNode>,
}

struct InvalidNodeSource;

impl NnsNodeSource for InvalidNodeSource {
    fn fetch_node_list_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsNodeListReport, NnsNodeHostError> {
        let mut report = node_report_fixture();
        report.fetched_at.clone_from(&request.fetched_at);
        report.source_endpoint.clone_from(&request.endpoint);
        report.node_count = 2;
        Ok(report)
    }
}

impl NnsNodeSource for FixtureNodeSource {
    fn fetch_node_list_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsNodeListReport, NnsNodeHostError> {
        let nodes = self
            .nodes
            .iter()
            .map(|node| NnsNodeRow {
                node_principal: node.principal.clone(),
                node_operator_principal: node.node_operator_principal.clone(),
                node_provider_principal: node.node_provider_principal.clone(),
                subnet_principal: node.subnet_principal.clone(),
                subnet_kind: node.subnet_kind,
                data_center_id: node.data_center_id.clone(),
            })
            .collect::<Vec<_>>();
        Ok(NnsNodeListReport {
            schema_version: 1,
            network: MAINNET_NETWORK.to_string(),
            registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            node_count: nodes.len(),
            nodes,
        })
    }
}
