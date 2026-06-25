use ic_query::nns::registry::{
    NnsRegistryVersionReport, NnsRegistryVersionRequest, nns_registry_version_report_text,
};

#[test]
fn public_nns_registry_api_is_constructible_and_renderable() {
    let request = NnsRegistryVersionRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
    };

    assert_eq!(request.network, "ic");

    let report = NnsRegistryVersionReport {
        schema_version: 1,
        network: request.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
    };

    let text = nns_registry_version_report_text(&report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("registry_version: 42"));
}
