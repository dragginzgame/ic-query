use super::{
    commands::test_support::{
        allowance_usage, archives_usage, balance_usage, block_types_usage, index_usage,
        parse_allowance_options, parse_archives_options, parse_balance_options,
        parse_block_types_options, parse_index_options, parse_token_options,
        parse_transactions_options, root_usage, token_usage, transactions_usage,
        try_parse_allowance_options, try_parse_archives_options, try_parse_balance_options,
        try_parse_block_types_options, try_parse_index_options, try_parse_token_options,
        try_parse_transactions_options,
    },
    ledger::{
        Icrc3ArchiveCallback, Icrc3ArchiveInfo, Icrc3ArchivedBlocks, Icrc3BlockWithId,
        Icrc3GetArchivesArgs, Icrc3GetBlocksRequest, Icrc3GetBlocksResult, Icrc3SupportedBlockType,
        Icrc3Value,
    },
    live::{
        IcrcSource, build_icrc_allowance_report_with_source,
        build_icrc_archives_report_with_source, build_icrc_balance_report_with_source,
        build_icrc_block_types_report_with_source, build_icrc_index_report_with_source,
        build_icrc_token_report_with_source, build_icrc_transactions_report_with_source,
    },
    model::{
        IcrcAllowanceData, IcrcAllowanceRequest, IcrcArchiveRow, IcrcArchivedBlocksRow,
        IcrcArchivedRangeRow, IcrcArchivesData, IcrcArchivesRequest, IcrcBalanceData,
        IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesData, IcrcBlockTypesRequest, IcrcError,
        IcrcIndexData, IcrcIndexRequest, IcrcTokenData, IcrcTokenMetadataRow, IcrcTokenRequest,
        IcrcTokenStandardRow, IcrcTransactionBlockRow, IcrcTransactionsData,
        IcrcTransactionsRequest,
    },
    text::{
        icrc_allowance_report_text, icrc_archives_report_text, icrc_balance_report_text,
        icrc_block_types_report_text, icrc_index_report_text, icrc_token_report_text,
        icrc_transactions_report_text,
    },
};
use crate::cli::common::OutputFormat;
use candid::{Nat, Principal, types::reference::Func};
use serde_json::{Value as JsonValue, json};
use std::collections::BTreeMap;

const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const INDEX_CANISTER_ID: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const ARCHIVE_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";
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

    fn fetch_index(&self, request: &IcrcIndexRequest) -> Result<IcrcIndexData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);

        Ok(IcrcIndexData {
            index_canister_id: Some(INDEX_CANISTER_ID.to_string()),
            index_error: None,
        })
    }

    fn fetch_transactions(
        &self,
        request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);
        assert_eq!(request.start, 100);
        assert_eq!(request.limit, 2);

        Ok(IcrcTransactionsData {
            log_length: Some("1000".to_string()),
            blocks: vec![
                IcrcTransactionBlockRow {
                    index: "100".to_string(),
                    block_type: Some("1xfer".to_string()),
                    transaction_kind: Some("1xfer".to_string()),
                    timestamp_unix_nanos: Some("1700000000123456789".to_string()),
                    amount_base_units: Some("123456789".to_string()),
                    raw_block: json!({
                        "Map": {
                            "btype": { "Text": "1xfer" },
                            "ts": { "Nat": "1700000000123456789" },
                            "tx": {
                                "Map": {
                                    "amt": { "Nat": "123456789" }
                                }
                            }
                        }
                    }),
                },
                IcrcTransactionBlockRow {
                    index: "101".to_string(),
                    block_type: None,
                    transaction_kind: Some("mint".to_string()),
                    timestamp_unix_nanos: Some("1700000001123456789".to_string()),
                    amount_base_units: Some("42".to_string()),
                    raw_block: json!({
                        "Map": {
                            "ts": { "Nat": "1700000001123456789" },
                            "tx": {
                                "Map": {
                                    "op": { "Text": "mint" },
                                    "amt": { "Nat": "42" }
                                }
                            }
                        }
                    }),
                },
            ],
            archived_blocks: vec![IcrcArchivedBlocksRow {
                callback_canister_id: ARCHIVE_CANISTER_ID.to_string(),
                callback_method: "icrc3_get_blocks".to_string(),
                ranges: vec![IcrcArchivedRangeRow {
                    start: "0".to_string(),
                    length: "100".to_string(),
                }],
            }],
        })
    }

    fn fetch_block_types(
        &self,
        request: &IcrcBlockTypesRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);

        Ok(IcrcBlockTypesData {
            block_types: vec![
                IcrcBlockTypeRow {
                    block_type: "1xfer".to_string(),
                    url: "https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-3".to_string(),
                },
                IcrcBlockTypeRow {
                    block_type: "2approve".to_string(),
                    url: "https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-3".to_string(),
                },
            ],
        })
    }

    fn fetch_archives(&self, request: &IcrcArchivesRequest) -> Result<IcrcArchivesData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);
        assert_eq!(
            request.from_canister_id.as_deref(),
            Some(ARCHIVE_CANISTER_ID)
        );

        Ok(IcrcArchivesData {
            archives: vec![IcrcArchiveRow {
                canister_id: ARCHIVE_CANISTER_ID.to_string(),
                start: "0".to_string(),
                end: "999".to_string(),
            }],
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

    fn fetch_index(&self, _request: &IcrcIndexRequest) -> Result<IcrcIndexData, IcrcError> {
        panic!("index source should not be called")
    }

    fn fetch_transactions(
        &self,
        _request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError> {
        panic!("transactions source should not be called")
    }

    fn fetch_block_types(
        &self,
        _request: &IcrcBlockTypesRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError> {
        panic!("block types source should not be called")
    }

    fn fetch_archives(
        &self,
        _request: &IcrcArchivesRequest,
    ) -> Result<IcrcArchivesData, IcrcError> {
        panic!("archives source should not be called")
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
fn index_options_parse_through_clap() {
    let options = parse_index_options(&[
        LEDGER_CANISTER_ID,
        "--format",
        "json",
        "--source-endpoint",
        SOURCE_ENDPOINT,
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
    assert!(try_parse_index_options(&["not-a-principal"]).is_err());
}

#[test]
fn transactions_options_parse_through_clap() {
    let options = parse_transactions_options(&[
        LEDGER_CANISTER_ID,
        "--start",
        "100",
        "--limit",
        "2",
        "--format",
        "json",
        "--source-endpoint",
        SOURCE_ENDPOINT,
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.start, 100);
    assert_eq!(options.limit, 2);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
    assert!(try_parse_transactions_options(&["not-a-principal"]).is_err());
    assert!(try_parse_transactions_options(&[LEDGER_CANISTER_ID, "--limit", "0"]).is_err());
    assert!(try_parse_transactions_options(&[LEDGER_CANISTER_ID, "--limit", "101"]).is_err());
    assert!(try_parse_transactions_options(&[LEDGER_CANISTER_ID, "--start", "-1"]).is_err());
}

#[test]
fn block_types_options_parse_through_clap() {
    let options = parse_block_types_options(&[
        LEDGER_CANISTER_ID,
        "--format",
        "json",
        "--source-endpoint",
        SOURCE_ENDPOINT,
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
    assert!(try_parse_block_types_options(&["not-a-principal"]).is_err());
}

#[test]
fn archives_options_parse_through_clap() {
    let options = parse_archives_options(&[
        LEDGER_CANISTER_ID,
        "--from",
        ARCHIVE_CANISTER_ID,
        "--format",
        "json",
        "--source-endpoint",
        SOURCE_ENDPOINT,
    ]);

    assert_eq!(options.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(
        options.from_canister_id.as_deref(),
        Some(ARCHIVE_CANISTER_ID)
    );
    assert_eq!(options.format, OutputFormat::Json);
    assert_eq!(options.source_endpoint, SOURCE_ENDPOINT);
    assert!(try_parse_archives_options(&["not-a-principal"]).is_err());
    assert!(
        try_parse_archives_options(&[LEDGER_CANISTER_ID, "--from", "not-a-principal"]).is_err()
    );
}

#[test]
fn icrc3_blocks_result_round_trips_through_candid() {
    let mut tx = BTreeMap::new();
    tx.insert("op".to_string(), Icrc3Value::Text("mint".to_string()));
    tx.insert("amt".to_string(), Icrc3Value::Nat(Nat::from(42_u64)));

    let mut block = BTreeMap::new();
    block.insert("btype".to_string(), Icrc3Value::Text("1mint".to_string()));
    block.insert(
        "ts".to_string(),
        Icrc3Value::Nat(Nat::from(1_700_000_001_000_000_000_u64)),
    );
    block.insert("tx".to_string(), Icrc3Value::Map(tx));

    let result = Icrc3GetBlocksResult {
        log_length: Nat::from(1_000_u64),
        blocks: vec![Icrc3BlockWithId {
            id: Nat::from(100_u64),
            block: Icrc3Value::Map(block),
        }],
        archived_blocks: vec![Icrc3ArchivedBlocks {
            args: vec![Icrc3GetBlocksRequest {
                start: Nat::from(0_u64),
                length: Nat::from(100_u64),
            }],
            callback: Icrc3ArchiveCallback(Func {
                principal: Principal::from_text("qaa6y-5yaaa-aaaaa-aaafa-cai")
                    .expect("archive callback principal"),
                method: "icrc3_get_blocks".to_string(),
            }),
        }],
    };

    let bytes = candid::encode_one(&result).expect("encode ICRC-3 blocks result");
    let decoded: Icrc3GetBlocksResult =
        candid::decode_one(&bytes).expect("decode ICRC-3 blocks result");

    assert_eq!(decoded, result);
}

#[test]
fn icrc3_archive_and_block_type_shapes_round_trip_through_candid() {
    let archives_args = Icrc3GetArchivesArgs {
        from: Some(
            Principal::from_text(ARCHIVE_CANISTER_ID).expect("archive pagination principal"),
        ),
    };
    let archives = vec![Icrc3ArchiveInfo {
        canister_id: Principal::from_text(ARCHIVE_CANISTER_ID).expect("archive canister principal"),
        start: Nat::from(0_u64),
        end: Nat::from(999_u64),
    }];
    let block_types = vec![Icrc3SupportedBlockType {
        block_type: "1xfer".to_string(),
        url: "https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-3".to_string(),
    }];

    let archives_args_bytes = candid::encode_one(&archives_args).expect("encode archives args");
    let archives_bytes = candid::encode_one(&archives).expect("encode archives result");
    let block_types_bytes = candid::encode_one(&block_types).expect("encode block types result");

    let decoded_archives_args: Icrc3GetArchivesArgs =
        candid::decode_one(&archives_args_bytes).expect("decode archives args");
    let decoded_archives: Vec<Icrc3ArchiveInfo> =
        candid::decode_one(&archives_bytes).expect("decode archives result");
    let decoded_block_types: Vec<Icrc3SupportedBlockType> =
        candid::decode_one(&block_types_bytes).expect("decode block types result");

    assert_eq!(decoded_archives_args, archives_args);
    assert_eq!(decoded_archives, archives);
    assert_eq!(decoded_block_types, block_types);
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
fn index_report_builds_text_and_json_friendly_fields() {
    let request = IcrcIndexRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = build_icrc_index_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC index report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(report.index_canister_id.as_deref(), Some(INDEX_CANISTER_ID));
    assert_eq!(report.index_error, None);

    let text = icrc_index_report_text(&report);
    assert!(text.contains("index_canister_id: bw4dl-smaaa-aaaaa-qaacq-cai"));
    assert!(!text.contains("index_error"));

    let json = serde_json::to_value(&report).expect("serialize ICRC index report");
    assert_eq!(json["index_canister_id"], json!(INDEX_CANISTER_ID));
    assert_eq!(json["index_error"], JsonValue::Null);
}

#[test]
fn transactions_report_builds_text_and_json_friendly_fields() {
    let request = IcrcTransactionsRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        start: 100,
        limit: 2,
    };

    let report = build_icrc_transactions_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC transactions report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(report.requested_start, "100");
    assert_eq!(report.requested_limit, 2);
    assert_eq!(report.log_length.as_deref(), Some("1000"));
    assert_eq!(report.blocks.len(), 2);
    assert_eq!(report.archived_blocks.len(), 1);

    let text = icrc_transactions_report_text(&report);
    assert!(text.contains("requested_start: 100"));
    assert!(text.contains("returned_blocks: 2"));
    assert!(text.contains("1xfer"));
    assert!(text.contains("1700000000123456789"));
    assert!(text.contains("qaa6y-5yaaa-aaaaa-aaafa-cai"));

    let json = serde_json::to_value(&report).expect("serialize ICRC transactions report");
    assert_eq!(json["requested_start"], json!("100"));
    assert_eq!(json["requested_limit"], json!(2));
    assert_eq!(json["log_length"], json!("1000"));
    assert_eq!(json["blocks"][0]["index"], json!("100"));
    assert_eq!(json["blocks"][0]["amount_base_units"], json!("123456789"));
    assert_eq!(
        json["blocks"][0]["raw_block"]["Map"]["btype"]["Text"],
        json!("1xfer")
    );
    assert_eq!(
        json["archived_blocks"][0]["ranges"][0]["length"],
        json!("100")
    );
}

#[test]
fn block_types_report_builds_text_and_json_friendly_fields() {
    let request = IcrcBlockTypesRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = build_icrc_block_types_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC block types report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(report.block_types.len(), 2);
    assert_eq!(report.block_types[0].block_type, "1xfer");

    let text = icrc_block_types_report_text(&report);
    assert!(text.contains("block_type_count: 2"));
    assert!(text.contains("1xfer"));
    assert!(text.contains("2approve"));

    let json = serde_json::to_value(&report).expect("serialize ICRC block types report");
    assert_eq!(json["block_types"][0]["block_type"], json!("1xfer"));
    assert_eq!(json["block_types"][1]["block_type"], json!("2approve"));
}

#[test]
fn archives_report_builds_text_and_json_friendly_fields() {
    let request = IcrcArchivesRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        from_canister_id: Some(ARCHIVE_CANISTER_ID.to_string()),
    };

    let report = build_icrc_archives_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC archives report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(
        report.from_canister_id.as_deref(),
        Some(ARCHIVE_CANISTER_ID)
    );
    assert_eq!(report.archives.len(), 1);
    assert_eq!(report.archives[0].canister_id, ARCHIVE_CANISTER_ID);
    assert_eq!(report.archives[0].start, "0");
    assert_eq!(report.archives[0].end, "999");

    let text = icrc_archives_report_text(&report);
    assert!(text.contains("archive_count: 1"));
    assert!(text.contains("from_canister_id: qaa6y-5yaaa-aaaaa-aaafa-cai"));
    assert!(text.contains("qaa6y-5yaaa-aaaaa-aaafa-cai"));
    assert!(text.contains("999"));

    let json = serde_json::to_value(&report).expect("serialize ICRC archives report");
    assert_eq!(json["from_canister_id"], json!(ARCHIVE_CANISTER_ID));
    assert_eq!(
        json["archives"][0]["canister_id"],
        json!(ARCHIVE_CANISTER_ID)
    );
    assert_eq!(json["archives"][0]["start"], json!("0"));
    assert_eq!(json["archives"][0]["end"], json!("999"));
}

#[test]
fn index_report_renders_index_error_when_not_set() {
    struct MissingIndexSource;

    impl IcrcSource for MissingIndexSource {
        fn fetch_token(&self, _request: &IcrcTokenRequest) -> Result<IcrcTokenData, IcrcError> {
            panic!("token source should not be called")
        }

        fn fetch_balance(
            &self,
            _request: &IcrcBalanceRequest,
        ) -> Result<IcrcBalanceData, IcrcError> {
            panic!("balance source should not be called")
        }

        fn fetch_allowance(
            &self,
            _request: &IcrcAllowanceRequest,
        ) -> Result<IcrcAllowanceData, IcrcError> {
            panic!("allowance source should not be called")
        }

        fn fetch_index(&self, _request: &IcrcIndexRequest) -> Result<IcrcIndexData, IcrcError> {
            Ok(IcrcIndexData {
                index_canister_id: None,
                index_error: Some("index principal not set".to_string()),
            })
        }

        fn fetch_transactions(
            &self,
            _request: &IcrcTransactionsRequest,
        ) -> Result<IcrcTransactionsData, IcrcError> {
            Ok(IcrcTransactionsData {
                log_length: None,
                blocks: Vec::new(),
                archived_blocks: Vec::new(),
            })
        }

        fn fetch_block_types(
            &self,
            _request: &IcrcBlockTypesRequest,
        ) -> Result<IcrcBlockTypesData, IcrcError> {
            panic!("block types source should not be called")
        }

        fn fetch_archives(
            &self,
            _request: &IcrcArchivesRequest,
        ) -> Result<IcrcArchivesData, IcrcError> {
            panic!("archives source should not be called")
        }
    }

    let request = IcrcIndexRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = build_icrc_index_report_with_source(&request, &MissingIndexSource)
        .expect("build ICRC index report");

    let text = icrc_index_report_text(&report);
    assert!(text.contains("index_canister_id: -"));
    assert!(text.contains("index_error: index principal not set"));

    let json = serde_json::to_value(&report).expect("serialize ICRC index report");
    assert_eq!(json["index_canister_id"], JsonValue::Null);
    assert_eq!(json["index_error"], json!("index principal not set"));
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
    assert!(root.contains("index"));
    assert!(root.contains("transactions"));
    assert!(root.contains("block-types"));
    assert!(root.contains("archives"));

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

    let index = index_usage();
    assert!(index.contains("Usage: icq icrc index [OPTIONS] <ledger-canister-id>"));
    assert!(index.contains("--source-endpoint <url>"));
    assert!(index.contains("--format <text|json>"));

    let transactions = transactions_usage();
    assert!(transactions.contains("Usage: icq icrc transactions [OPTIONS] <ledger-canister-id>"));
    assert!(transactions.contains("--start <index>"));
    assert!(transactions.contains("--limit <count>"));
    assert!(transactions.contains("--source-endpoint <url>"));
    assert!(transactions.contains("--format <text|json>"));

    let block_types = block_types_usage();
    assert!(block_types.contains("Usage: icq icrc block-types [OPTIONS] <ledger-canister-id>"));
    assert!(block_types.contains("--source-endpoint <url>"));
    assert!(block_types.contains("--format <text|json>"));

    let archives = archives_usage();
    assert!(archives.contains("Usage: icq icrc archives [OPTIONS] <ledger-canister-id>"));
    assert!(archives.contains("--from <canister-id>"));
    assert!(archives.contains("--source-endpoint <url>"));
    assert!(archives.contains("--format <text|json>"));
}
