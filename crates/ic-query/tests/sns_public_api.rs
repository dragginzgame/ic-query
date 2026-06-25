use ic_query::sns::{SnsListReport, SnsListRequest, SnsListSort, sns_list_report_text};

#[test]
fn public_sns_list_api_is_constructible_and_renderable() {
    let request = SnsListRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        verbose: false,
        sort: SnsListSort::Id,
    };

    assert_eq!(request.sort.as_str(), "id");

    let report = SnsListReport {
        schema_version: 3,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        verbose: request.verbose,
        sort: request.sort.as_str().to_string(),
        sns_count: 0,
        metadata_error_count: 0,
        sns_instances: Vec::new(),
    };

    let text = sns_list_report_text(&report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("sns_count: 0"));
}
