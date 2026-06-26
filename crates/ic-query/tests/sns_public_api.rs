use ic_query::sns::{
    SnsInfoReport, SnsInfoRequest, SnsListReport, SnsListRequest, SnsListSort, SnsTokenMetadataRow,
    SnsTokenReport, SnsTokenRequest, SnsTokenStandardRow, sns_info_report_text,
    sns_list_report_text, sns_token_report_text,
};
use serde_json::json;

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

#[test]
fn public_sns_info_api_is_constructible_and_renderable() {
    let request = SnsInfoRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "1".to_string(),
    };

    let report = SnsInfoReport {
        schema_version: 2,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        description: Some("Example description".to_string()),
        url: None,
        root_canister_id: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
        governance_canister_id: "csyra-haaaa-aaaaa-qaaeq-cai".to_string(),
        ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        swap_canister_id: "ca6gz-lqaaa-aaaaa-qaacu-cai".to_string(),
        index_canister_id: "qhbym-qaaaa-aaaaa-aaafq-cai".to_string(),
        metadata_error: None,
    };

    assert_eq!(request.input, "1");

    let text = sns_info_report_text(&report);

    assert!(text.contains("sns_id: 1"));
    assert!(text.contains("description: Example description"));
    assert!(text.contains("url: -"));
}

#[test]
fn public_sns_token_api_is_constructible_and_renderable() {
    let request = SnsTokenRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
    };

    let report = SnsTokenReport {
        schema_version: 1,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: request.input,
        ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        sns_index_canister_id: "qhbym-qaaaa-aaaaa-aaafq-cai".to_string(),
        token_name: "Example Token".to_string(),
        token_symbol: "EXT".to_string(),
        decimals: 8,
        transfer_fee: "100_000_000".to_string(),
        total_supply: "1_000_000_000".to_string(),
        minting_account_owner: Some("aaaaa-aa".to_string()),
        minting_account_subaccount_hex: None,
        ledger_index_canister_id: None,
        ledger_index_error: Some("not configured".to_string()),
        supported_standards: vec![SnsTokenStandardRow {
            name: "ICRC-1".to_string(),
            url: "https://github.com/dfinity/ICRC-1".to_string(),
        }],
        metadata: vec![SnsTokenMetadataRow {
            key: "icrc1:symbol".to_string(),
            value_type: "Text".to_string(),
            value: json!("EXT"),
        }],
    };

    let text = sns_token_report_text(&report);

    assert!(text.contains("token_symbol: EXT"));
    assert!(text.contains("transfer_fee: 1.00"));
    assert!(text.contains("ledger_index_error: not configured"));
    assert!(text.contains("ICRC-1"));
}
