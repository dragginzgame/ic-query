use super::fixtures::*;
use super::*;
use crate::icrc::IcrcIndexData;
use ic_platform::icrc_ledger_types::MetadataValue;

#[tokio::test(flavor = "current_thread")]
async fn sns_token_resolves_list_id_and_renders_ledger_metadata() {
    let request = token_request("1");

    let report = build_sns_token_report_with_source(
        &request,
        &FixtureSnsTokenSource(IcrcIndexData::Found(INDEX_A.to_string())),
    )
    .await
    .expect("sns token report");

    let json = serde_json::to_value(&report).expect("serialize SNS token report");

    assert_eq!(report.schema_version, SNS_TOKEN_REPORT_SCHEMA_VERSION);
    assert_eq!(report.context.id, 1);
    assert_eq!(report.context.name, "Fixture SNS");
    assert_eq!(report.context.root_canister_id, ROOT_A);
    assert_eq!(report.ledger_canister_id, LEDGER_A);
    assert_eq!(report.sns_index_canister_id, INDEX_A);
    assert_eq!(
        report.ledger_index,
        IcrcIndexData::Found(INDEX_A.to_string())
    );
    assert_eq!(report.token.token_name, "Fixture Token");
    assert_eq!(report.token.token_symbol, "FIX");
    assert_eq!(report.token.decimals, 8);
    assert_eq!(report.token.transfer_fee, "10_000");
    assert_eq!(report.token.total_supply, "1_000_000_000");
    assert_eq!(
        report.token.minting_account_owner.as_deref(),
        Some(GOVERNANCE_A)
    );
    assert_eq!(
        report.token.minting_account_subaccount_hex.as_deref(),
        Some("000102")
    );
    assert_eq!(report.token.supported_standards[0].name, "ICRC-1");
    assert_eq!(report.token.metadata[0].key, "icrc1:name");
    assert!(
        report
            .token
            .metadata
            .iter()
            .any(|row| row.key == "icrc1:logo"
                && row.value_type == IcrcMetadataValueKind::Text
                && row.value == serde_json::json!("data:image/png;base64,fixture"))
    );
    assert_eq!(json["metadata"][0]["value_type"], serde_json::json!("text"));
}

#[test]
fn sns_token_logo_metadata_preserves_raw_text() {
    let row = metadata_row(
        "icrc1:logo".to_string(),
        MetadataValue::Text("data:image/png;base64,large-logo".to_string()),
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
        MetadataValue::Text(" ".to_string()),
    );

    assert_eq!(row.value_type, IcrcMetadataValueKind::Text);
    assert_eq!(row.value, serde_json::json!(" "));
}

#[tokio::test(flavor = "current_thread")]
async fn sns_token_retains_metadata_when_ledger_index_discovery_is_unavailable() {
    let error = "ledger index discovery request timed out";
    let report = build_sns_token_report_with_source(
        &token_request("1"),
        &FixtureSnsTokenSource(IcrcIndexData::Unavailable(error.to_string())),
    )
    .await
    .expect("metadata survives index discovery failure");
    assert_eq!(report.token.token_symbol, "FIX");
    assert_eq!(report.sns_index_canister_id, INDEX_A);
    assert_eq!(
        report.ledger_index,
        IcrcIndexData::Unavailable(error.to_string())
    );
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(
        json["ledger_index"],
        serde_json::json!({"unavailable": error})
    );
}
