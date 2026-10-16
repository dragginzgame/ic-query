use super::{fixtures::*, *};

#[tokio::test(flavor = "current_thread")]
async fn node_provider_refresh_writes_cache_and_list_reads_it() {
    let cache = test_cache_request(MAINNET_NETWORK, "refresh-cache");
    let refresh_request = NnsInventoryRefreshRequest {
        cache: cache.clone(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,
        lock_stale_after_seconds: DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
        dry_run: false,
        output_path: None,
    };
    let refresh_report = refresh_nns_node_provider_report_with_source(
        &refresh_request,
        &FixtureNodeProviderSource {
            node_providers: vec![MainnetNodeProvider {
                principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
                node_count: Some(13),
                reward_account_hex: Some("abcd".to_string()),
            }],
        },
    )
    .await
    .expect("refresh report");

    assert!(std::path::Path::new(&refresh_report.cache_path).is_file());
    assert!(refresh_report.wrote_cache);
    assert_eq!(refresh_report.row_count, 1);
    assert_eq!(refresh_report.collection, "node_providers");
    assert_eq!(refresh_report.schema_version, 1);
    assert_eq!(
        refresh_report.governance_canister_id.as_deref(),
        Some(MAINNET_GOVERNANCE_CANISTER_ID)
    );
    let result_json = serde_json::to_value(&refresh_report).expect("serialize refresh outcome");
    assert_eq!(result_json["collection"], "node_providers");
    assert_eq!(result_json["row_count"], 1);

    let list_request = NnsInventoryListRequest {
        cache,
        source_endpoint: "https://unused.example".to_string(),
        now_unix_secs: 1_780_531_300,
    };
    let report =
        build_nns_node_provider_list_report_with_source(&list_request, &FailingNodeProviderSource)
            .await
            .expect("cached report");

    assert_eq!(report.source_endpoint, "https://icp-api.io");
    assert_eq!(report.node_providers.len(), 1);
    assert_eq!(report.node_providers[0].node_count, Some(13));
    assert_eq!(refresh_report.registry_version, report.registry_version);
    assert_eq!(refresh_report.fetched_at, report.fetched_at);
    assert_eq!(refresh_report.source_endpoint, report.source_endpoint);

    let cache_bytes = std::fs::read(&refresh_report.cache_path).expect("read published inventory");
    let output_path = list_request
        .cache
        .cache_root
        .join("exports")
        .join("providers.json");
    let dry_run = refresh_nns_node_provider_report_with_source(
        &refresh_request
            .with_dry_run(true)
            .with_output_path(&output_path),
        &FixtureNodeProviderSource {
            node_providers: Vec::new(),
        },
    )
    .await
    .expect("dry-run export");
    assert!(dry_run.dry_run);
    assert!(!dry_run.wrote_cache);
    assert!(dry_run.replaced_existing_cache);
    assert_eq!(dry_run.row_count, 0);
    assert_eq!(dry_run.collection, "node_providers");
    assert_eq!(dry_run.output_path, Some(output_path.display().to_string()));
    assert_eq!(
        std::fs::read(&refresh_report.cache_path).expect("retained inventory"),
        cache_bytes
    );
    let export: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output_path).expect("read export"))
            .expect("parse export");
    assert_eq!(export["node_provider_count"], 0);
}
