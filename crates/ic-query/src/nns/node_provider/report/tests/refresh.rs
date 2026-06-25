use super::{fixtures::*, *};

#[test]
fn node_provider_refresh_writes_cache_and_list_reads_it() {
    let cache = test_cache_request(MAINNET_NETWORK, "refresh-cache");
    let refresh_request = NnsNodeProviderRefreshRequest {
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
    .expect("refresh report");

    assert!(std::path::Path::new(&refresh_report.cache_path).is_file());
    assert!(refresh_report.wrote_cache);
    assert_eq!(refresh_report.node_provider_count, 1);

    let list_request = NnsNodeProviderListRequest {
        cache,
        source_endpoint: "https://unused.example".to_string(),
        now_unix_secs: 1_780_531_300,
    };
    let report =
        build_nns_node_provider_list_report_with_source(&list_request, &FailingNodeProviderSource)
            .expect("cached report");

    assert_eq!(report.source_endpoint, "https://icp-api.io");
    assert_eq!(report.node_providers.len(), 1);
    assert_eq!(report.node_providers[0].node_count, Some(13));
}
