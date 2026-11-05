use super::{fixtures::*, *};

#[tokio::test(flavor = "current_thread")]
async fn node_provider_refresh_writes_cache_and_list_reads_it() {
    let cache = test_cache_request(MAINNET_NETWORK, "refresh-cache");
    let refresh_request = NnsInventoryRefreshRequest {
        cache: cache.clone(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_780_531_200,

        dry_run: false,
        output_path: None,
    };
    let refresh_report = refresh_nns_node_provider_report_with_source(
        &refresh_request,
        &FixtureNodeProviderSource {
            node_providers: vec![NnsNodeProviderRow {
                name: None,
                node_provider_principal: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
                node_count: Some(13),
                reward_account_hex: Some("abcd".to_string()),
            }],
        },
    )
    .await
    .expect("refresh report");

    assert!(std::path::Path::new(&refresh_report.metadata.cache_path).is_file());
    assert!(refresh_report.metadata.wrote_cache);
    assert_eq!(refresh_report.metadata.row_count, 1);
    assert_eq!(refresh_report.metadata.collection, "node_provider_metadata");
    assert_eq!(refresh_report.metadata.schema_version, 1);
    assert_eq!(
        refresh_report.metadata.governance_canister_id.as_deref(),
        Some(MAINNET_GOVERNANCE_CANISTER_ID)
    );
    let result_json = serde_json::to_value(&refresh_report).expect("serialize refresh outcome");
    assert_eq!(
        result_json["metadata"]["collection"],
        "node_provider_metadata"
    );
    assert_eq!(result_json["metadata"]["row_count"], 1);

    let list_request = NnsInventoryListRequest {
        cache,
        source_endpoint: "https://unused.example".to_string(),
        now_unix_secs: 1_780_531_300,
    };
    let report = build_nns_node_provider_list_report_with_source_and_progress(
        &list_request,
        &FailingNodeProviderSource,
        &mut |_| {},
    )
    .await
    .expect("cached report");

    assert_eq!(report.provenance.source_endpoint, "https://icp-api.io");
    assert_eq!(report.node_providers.len(), 1);
    assert_eq!(report.node_providers[0].node_count, Some(13));
    assert_eq!(
        refresh_report.metadata.provenance.registry_version,
        report.provenance.registry_version
    );
    assert_eq!(
        refresh_report.metadata.provenance.fetched_at,
        report.provenance.fetched_at
    );
    assert_eq!(
        refresh_report.metadata.provenance.source_endpoint,
        report.provenance.source_endpoint
    );

    let cache_bytes =
        std::fs::read(&refresh_report.metadata.cache_path).expect("read published inventory");
    let output_path = list_request
        .cache
        .cache_root
        .join("exports")
        .join("providers.json");
    let dry_run = refresh_nns_node_provider_report_with_source(
        &{
            let mut configured_request = refresh_request;
            configured_request.dry_run = true;
            configured_request.output_path = Some((&output_path).into());
            configured_request
        },
        &FixtureNodeProviderSource {
            node_providers: Vec::new(),
        },
    )
    .await
    .expect("dry-run export");
    assert!(dry_run.metadata.dry_run);
    assert!(!dry_run.metadata.wrote_cache);
    assert!(dry_run.metadata.replaced_existing_cache);
    assert_eq!(dry_run.metadata.row_count, 0);
    assert_eq!(dry_run.metadata.collection, "node_provider_metadata");
    assert_eq!(
        dry_run.metadata.output_path,
        Some(output_path.display().to_string())
    );
    assert_eq!(
        std::fs::read(&refresh_report.metadata.cache_path).expect("retained inventory"),
        cache_bytes
    );
    let export: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output_path).expect("read export"))
            .expect("parse export");
    assert_eq!(export["node_providers"], serde_json::json!([]));
}

#[tokio::test(flavor = "current_thread")]
async fn provider_metadata_admission_preserves_governance_ownership() {
    let source = FixtureNodeProviderSource {
        node_providers: node_provider_report_fixture().node_providers,
    };
    let request = crate::report::CollectionSourceRequest::from_unix_secs(
        "ic",
        "https://icp-api.io",
        1_780_531_200,
        "fixture",
    );
    let mut metadata = source.fetch_node_provider_metadata(&request).await.unwrap();
    assert!(metadata.validate().is_ok());
    let original = serde_json::to_string(&metadata).unwrap();
    assert_eq!(
        serde_json::from_str::<NnsNodeProviderMetadata>(&original).unwrap(),
        metadata
    );
    for extra in [
        "\"unexpected\":true,",
        "\"schema_version\":1,",
        "\"network\":\"ic\",",
    ] {
        let bad = format!("{{{extra}{}", &original[1..]);
        assert!(serde_json::from_str::<NnsNodeProviderMetadata>(&bad).is_err());
    }
    let mut nested = serde_json::to_value(&metadata).unwrap();
    nested["node_providers"][0]["unexpected"] = true.into();
    assert!(serde_json::from_value::<NnsNodeProviderMetadata>(nested).is_err());
    metadata.node_providers[0].node_count = Some(0);
    assert!(metadata.validate().is_err());
}
