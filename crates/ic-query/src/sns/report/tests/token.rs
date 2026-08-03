use super::{fixtures::*, *};

#[test]
fn sns_token_resolves_list_id_and_renders_ledger_metadata() {
    let request = token_request("1");

    let report = build_sns_token_report_with_source(&request, &FixtureSnsTokenSource)
        .expect("sns token report");
    let text = sns_token_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize SNS token report");

    assert_eq!(report.schema_version, SNS_TOKEN_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.ledger_canister_id, LEDGER_A);
    assert_eq!(report.sns_index_canister_id, INDEX_A);
    assert_eq!(report.ledger_index_canister_id.as_deref(), Some(INDEX_A));
    assert_eq!(report.token_name, "Fixture Token");
    assert_eq!(report.token_symbol, "FIX");
    assert_eq!(report.decimals, 8);
    assert_eq!(report.transfer_fee, "10_000");
    assert_eq!(report.total_supply, "1_000_000_000");
    assert_eq!(report.minting_account_owner.as_deref(), Some(GOVERNANCE_A));
    assert_eq!(
        report.minting_account_subaccount_hex.as_deref(),
        Some("000102")
    );
    assert_eq!(report.supported_standards[0].name, "ICRC-1");
    assert_eq!(report.metadata[0].key, "icrc1:name");
    assert!(report.metadata.iter().any(|row| row.key == "icrc1:logo"
        && row.value_type == IcrcMetadataValueKind::Text
        && row.value == serde_json::json!("data:image/png;base64,fixture")));
    assert_eq!(json["metadata"][0]["value_type"], serde_json::json!("text"));
    assert!(text.contains("token_symbol: FIX"));
    assert!(text.contains("transfer_fee: 0.00"));
    assert!(text.contains("total_supply: 10.00"));
    assert!(!text.contains("10_000"));
    assert!(!text.contains("1_000_000_000"));
    assert!(text.contains("ledger_index_canister_id: bw4dl-smaaa-aaaaa-qaacq-cai"));
    assert!(text.contains("ICRC-1"));
    assert!(text.contains("icrc1:name"));
    assert!(text.contains("icrc1:fee"));
    assert!(text.contains("icrc1:logo"));
    assert!(text.contains("present"));
    assert!(!text.contains("data:image"));
}

#[test]
fn sns_token_logo_metadata_preserves_raw_text() {
    let row = metadata_row(
        "icrc1:logo".to_string(),
        IcrcMetadataValue::Text("data:image/png;base64,large-logo".to_string()),
    );

    assert_eq!(row.key, "icrc1:logo");
    assert_eq!(row.value_type, IcrcMetadataValueKind::Text);
    assert_eq!(
        row.value,
        serde_json::json!("data:image/png;base64,large-logo")
    );
}

#[test]
fn sns_token_empty_logo_metadata_remains_raw() {
    let row = metadata_row(
        "icrc1:logo".to_string(),
        IcrcMetadataValue::Text(" ".to_string()),
    );

    assert_eq!(row.value_type, IcrcMetadataValueKind::Text);
    assert_eq!(row.value, serde_json::json!(" "));
}
