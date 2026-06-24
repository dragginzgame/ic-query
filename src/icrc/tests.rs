use super::{
    commands::test_support::{
        allowance_usage, balance_usage, parse_allowance_options, parse_balance_options,
        parse_token_options, root_usage, token_usage, try_parse_allowance_options,
        try_parse_balance_options, try_parse_token_options,
    },
    live::{
        IcrcSource, build_icrc_allowance_report_with_source, build_icrc_balance_report_with_source,
        build_icrc_token_report_with_source,
    },
    model::{
        IcrcAllowanceData, IcrcAllowanceRequest, IcrcBalanceData, IcrcBalanceRequest, IcrcError,
        IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenRequest, IcrcTokenStandardRow,
    },
    text::{icrc_allowance_report_text, icrc_balance_report_text, icrc_token_report_text},
};
use crate::cli::common::OutputFormat;
use serde_json::{Value as JsonValue, json};

const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const ACCOUNT_OWNER: &str = "aaaaa-aa";
const SOURCE_ENDPOINT: &str = "https://icp-api.io";
const FETCHED_AT_UNIX_SECS: u64 = 1_700_000_000;
const UPPER_SUBACCOUNT_HEX: &str =
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const LOWER_SUBACCOUNT_HEX: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

struct FixtureIcrcSource;

impl IcrcSource for FixtureIcrcSource {
    fn fetch_token(&self, request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);

        Ok(IcrcTokenData {
            token_name: "Fixture Token".to_string(),
            token_symbol: "FIX".to_string(),
            decimals: 8,
            transfer_fee: "123456789".to_string(),
            total_supply: "100000000000".to_string(),
            minting_account_owner: Some(ACCOUNT_OWNER.to_string()),
            minting_account_subaccount_hex: Some(LOWER_SUBACCOUNT_HEX.to_string()),
            supported_standards: vec![
                IcrcTokenStandardRow {
                    name: "ICRC-1".to_string(),
                    url: "https://github.com/dfinity/ICRC-1".to_string(),
                },
                IcrcTokenStandardRow {
                    name: "ICRC-2".to_string(),
                    url: "https://github.com/dfinity/ICRC-2".to_string(),
                },
            ],
            metadata: vec![
                IcrcTokenMetadataRow {
                    key: "icrc1:name".to_string(),
                    value_type: "text".to_string(),
                    value: JsonValue::String("Fixture Token".to_string()),
                },
                IcrcTokenMetadataRow {
                    key: "icrc1:fee".to_string(),
                    value_type: "nat".to_string(),
                    value: JsonValue::String("123456789".to_string()),
                },
                IcrcTokenMetadataRow {
                    key: "icrc1:logo".to_string(),
                    value_type: "bool".to_string(),
                    value: JsonValue::Bool(true),
                },
            ],
        })
    }

    fn fetch_balance(&self, request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.account_owner, ACCOUNT_OWNER);
        assert_eq!(
            request.subaccount_hex.as_deref(),
            Some(LOWER_SUBACCOUNT_HEX)
        );

        Ok(IcrcBalanceData {
            token_symbol: "FIX".to_string(),
            decimals: 8,
            balance: "123456789".to_string(),
        })
    }

    fn fetch_allowance(
        &self,
        request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.account_owner, ACCOUNT_OWNER);
        assert_eq!(
            request.account_subaccount_hex.as_deref(),
            Some(LOWER_SUBACCOUNT_HEX)
        );
        assert_eq!(request.spender_owner, ACCOUNT_OWNER);
        assert_eq!(
            request.spender_subaccount_hex.as_deref(),
            Some(LOWER_SUBACCOUNT_HEX)
        );

        Ok(IcrcAllowanceData {
            token_symbol: "FIX".to_string(),
            decimals: 8,
            allowance: "123456789".to_string(),
            expires_at_unix_nanos: Some("1700000000123456789".to_string()),
        })
    }
}

struct PanickingIcrcSource;

impl IcrcSource for PanickingIcrcSource {
    fn fetch_token(&self, _request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError> {
        panic!("token source should not be called")
    }

    fn fetch_balance(&self, _request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
        panic!("balance source should not be called")
    }

    fn fetch_allowance(
        &self,
        _request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError> {
        panic!("allowance source should not be called")
    }
}

#[test]
fn token_options_parse_through_clap() {
    let options = parse_token_options(&[
        LEDGER_CANISTER_ID,
        "--format",
        "json",
        "--source-endpoint",
        SOURCE_ENDPOINT,
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
    assert!(try_parse_token_options(&["not-a-principal"]).is_err());
}

#[test]
fn balance_options_parse_through_clap_and_normalize_subaccount() {
    let options = parse_balance_options(&[
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        "--subaccount",
        UPPER_SUBACCOUNT_HEX,
        "--format",
        "json",
        "--source-endpoint",
        SOURCE_ENDPOINT,
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(
        options.subaccount_hex.as_deref(),
        Some(LOWER_SUBACCOUNT_HEX)
    );
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
    assert!(try_parse_balance_options(&[LEDGER_CANISTER_ID, "not-a-principal"]).is_err());
    assert!(
        try_parse_balance_options(&[LEDGER_CANISTER_ID, ACCOUNT_OWNER, "--subaccount", "abc"])
            .is_err()
    );
}

#[test]
fn allowance_options_parse_through_clap_and_normalize_subaccounts() {
    let options = parse_allowance_options(&[
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        ACCOUNT_OWNER,
        "--owner-subaccount",
        UPPER_SUBACCOUNT_HEX,
        "--spender-subaccount",
        UPPER_SUBACCOUNT_HEX,
        "--format",
        "json",
        "--source-endpoint",
        SOURCE_ENDPOINT,
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.account_owner, ACCOUNT_OWNER);
    assert_eq!(
        options.account_subaccount_hex.as_deref(),
        Some(LOWER_SUBACCOUNT_HEX)
    );
    assert_eq!(options.spender_owner, ACCOUNT_OWNER);
    assert_eq!(
        options.spender_subaccount_hex.as_deref(),
        Some(LOWER_SUBACCOUNT_HEX)
    );
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
    assert!(
        try_parse_allowance_options(&[LEDGER_CANISTER_ID, "not-a-principal", ACCOUNT_OWNER])
            .is_err()
    );
    assert!(
        try_parse_allowance_options(&[
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            ACCOUNT_OWNER,
            "--owner-subaccount",
            "abc",
        ])
        .is_err()
    );
}

#[test]
fn token_report_builds_text_and_json_friendly_fields() {
    let request = IcrcTokenRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = build_icrc_token_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC token report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(report.token_name, "Fixture Token");
    assert_eq!(report.token_symbol, "FIX");
    assert_eq!(report.transfer_fee, "123456789");
    assert_eq!(report.total_supply, "100000000000");
    assert_eq!(
        report.minting_account_subaccount_hex.as_deref(),
        Some(LOWER_SUBACCOUNT_HEX)
    );

    let text = icrc_token_report_text(&report);
    assert!(text.contains("transfer_fee: 1.23"));
    assert!(text.contains("total_supply: 1000.00"));
    assert!(text.contains("ICRC-1"));
    assert!(text.contains("icrc1:logo"));

    let json = serde_json::to_value(&report).expect("serialize ICRC token report");
    assert_eq!(json["transfer_fee"], json!("123456789"));
    assert_eq!(json["metadata"][2]["value"], json!(true));
}

#[test]
fn balance_report_builds_text_and_json_friendly_fields() {
    let request = IcrcBalanceRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        account_owner: ACCOUNT_OWNER.to_string(),
        subaccount_hex: Some(UPPER_SUBACCOUNT_HEX.to_string()),
    };

    let report = build_icrc_balance_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC balance report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.subaccount_hex.as_deref(), Some(LOWER_SUBACCOUNT_HEX));
    assert_eq!(report.balance, "123456789");

    let text = icrc_balance_report_text(&report);
    assert!(text.contains("balance: 1.23 FIX"));
    assert!(text.contains("balance_base_units: 123456789"));

    let json = serde_json::to_value(&report).expect("serialize ICRC balance report");
    assert_eq!(json["balance"], json!("123456789"));
    assert_eq!(json["subaccount_hex"], json!(LOWER_SUBACCOUNT_HEX));
}

#[test]
fn allowance_report_builds_text_and_json_friendly_fields() {
    let request = IcrcAllowanceRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        account_owner: ACCOUNT_OWNER.to_string(),
        account_subaccount_hex: Some(UPPER_SUBACCOUNT_HEX.to_string()),
        spender_owner: ACCOUNT_OWNER.to_string(),
        spender_subaccount_hex: Some(UPPER_SUBACCOUNT_HEX.to_string()),
    };

    let report = build_icrc_allowance_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC allowance report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(
        report.account_subaccount_hex.as_deref(),
        Some(LOWER_SUBACCOUNT_HEX)
    );
    assert_eq!(
        report.spender_subaccount_hex.as_deref(),
        Some(LOWER_SUBACCOUNT_HEX)
    );
    assert_eq!(report.allowance, "123456789");
    assert_eq!(
        report.expires_at_unix_nanos.as_deref(),
        Some("1700000000123456789")
    );

    let text = icrc_allowance_report_text(&report);
    assert!(text.contains("allowance: 1.23 FIX"));
    assert!(text.contains("allowance_base_units: 123456789"));
    assert!(text.contains("expires_at_unix_nanos: 1700000000123456789"));

    let json = serde_json::to_value(&report).expect("serialize ICRC allowance report");
    assert_eq!(json["allowance"], json!("123456789"));
    assert_eq!(json["account_subaccount_hex"], json!(LOWER_SUBACCOUNT_HEX));
    assert_eq!(json["spender_subaccount_hex"], json!(LOWER_SUBACCOUNT_HEX));
    assert_eq!(json["expires_at_unix_nanos"], json!("1700000000123456789"));
}

#[test]
fn invalid_subaccount_is_rejected_before_source_fetch() {
    let request = IcrcBalanceRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        account_owner: ACCOUNT_OWNER.to_string(),
        subaccount_hex: Some("abc".to_string()),
    };

    let err = build_icrc_balance_report_with_source(&request, &PanickingIcrcSource)
        .expect_err("invalid subaccount should fail before source fetch");

    assert!(matches!(err, IcrcError::InvalidSubaccountHex { .. }));
}

#[test]
fn invalid_allowance_subaccount_is_rejected_before_source_fetch() {
    let request = IcrcAllowanceRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        account_owner: ACCOUNT_OWNER.to_string(),
        account_subaccount_hex: Some("abc".to_string()),
        spender_owner: ACCOUNT_OWNER.to_string(),
        spender_subaccount_hex: None,
    };

    let err = build_icrc_allowance_report_with_source(&request, &PanickingIcrcSource)
        .expect_err("invalid subaccount should fail before source fetch");

    assert!(matches!(err, IcrcError::InvalidSubaccountHex { .. }));
}

#[test]
fn usage_mentions_icrc_command_surface() {
    let root = root_usage();
    assert!(root.contains("Usage: icq icrc [COMMAND]"));
    assert!(root.contains("token"));
    assert!(root.contains("balance"));
    assert!(root.contains("allowance"));

    let token = token_usage();
    assert!(token.contains("Usage: icq icrc token [OPTIONS] <ledger-canister-id>"));
    assert!(token.contains("--source-endpoint <url>"));
    assert!(token.contains("--format <text|json>"));

    let balance = balance_usage();
    assert!(balance.contains("Usage: icq icrc balance [OPTIONS] <ledger-canister-id> <principal>"));
    assert!(balance.contains("--subaccount <hex>"));

    let allowance = allowance_usage();
    assert!(allowance.contains(
        "Usage: icq icrc allowance [OPTIONS] <ledger-canister-id> <owner-principal> <spender-principal>"
    ));
    assert!(allowance.contains("--owner-subaccount <hex>"));
    assert!(allowance.contains("--spender-subaccount <hex>"));
}
