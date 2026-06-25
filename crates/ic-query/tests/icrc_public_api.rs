use ic_query::icrc::{IcrcTokenReport, IcrcTokenRequest, icrc_token_report_text};

#[test]
fn public_icrc_token_api_is_constructible_and_renderable() {
    let request = IcrcTokenRequest {
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
    };

    assert_eq!(request.ledger_canister_id, "ryjl3-tyaaa-aaaaa-aaaba-cai");

    let report = IcrcTokenReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        token_name: "Internet Computer".to_string(),
        token_symbol: "ICP".to_string(),
        decimals: 8,
        transfer_fee: "10000".to_string(),
        total_supply: "100000000".to_string(),
        minting_account_owner: None,
        minting_account_subaccount_hex: None,
        supported_standards: Vec::new(),
        metadata: Vec::new(),
    };

    let text = icrc_token_report_text(&report);

    assert!(text.contains("ledger_canister_id: ryjl3-tyaaa-aaaaa-aaaba-cai"));
    assert!(text.contains("token_symbol: ICP"));
}
