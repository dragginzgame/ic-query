use super::{
    ledger::{
        Icrc3ArchiveCallback, Icrc3ArchiveInfo, Icrc3ArchivedBlocks, Icrc3BlockWithId,
        Icrc3DataCertificate, Icrc3GetArchivesArgs, Icrc3GetBlocksRequest, Icrc3GetBlocksResult,
        Icrc3SupportedBlockType, Icrc3Value,
    },
    live::{
        IcrcAccountTransactionPageSource, IcrcAllowanceSource, IcrcArchivesSource,
        IcrcBalanceSource, IcrcBlockTypesSource, IcrcCapabilitiesSource, IcrcIndexSource,
        IcrcTipCertificateSource, IcrcTokenSource, IcrcTransactionsSource,
        build_icrc_account_transaction_page_report_with_source,
        build_icrc_allowance_report_with_source, build_icrc_archives_report_with_source,
        build_icrc_balance_report_with_source, build_icrc_block_types_report_with_source,
        build_icrc_capabilities_report_with_source, build_icrc_index_report_with_source,
        build_icrc_tip_certificate_report_with_source, build_icrc_token_report_with_source,
        build_icrc_transactions_report_with_source,
    },
    model::{
        IcrcAccountRow, IcrcAccountTransactionError, IcrcAccountTransactionPageData,
        IcrcAccountTransactionPageRequest, IcrcAccountTransactionRow, IcrcAllowanceData,
        IcrcAllowanceRequest, IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow,
        IcrcArchivedRangeRow, IcrcArchivesData, IcrcArchivesRequest, IcrcBalanceData,
        IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesData, IcrcCapabilitiesData,
        IcrcCapabilityRow, IcrcError, IcrcFollowedArchiveBlockRow, IcrcIndexData,
        IcrcLedgerRequest, IcrcTipCertificateData, IcrcTokenData, IcrcTokenMetadataRow,
        IcrcTokenStandardRow, IcrcTransactionBlockRow, IcrcTransactionsData,
        IcrcTransactionsRequest,
    },
    text::{
        icrc_account_transaction_page_report_text, icrc_allowance_report_text,
        icrc_archives_report_text, icrc_balance_report_text, icrc_block_types_report_text,
        icrc_capabilities_report_text, icrc_index_report_text, icrc_tip_certificate_report_text,
        icrc_token_report_text, icrc_transactions_report_text,
    },
};
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

impl IcrcTokenSource for FixtureIcrcSource {
    fn fetch_token(&self, request: &IcrcLedgerRequest) -> Result<IcrcTokenData, IcrcError> {
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
                    value_type: "text".to_string(),
                    value: JsonValue::String("data:image/png;base64,fixture".to_string()),
                },
            ],
        })
    }
}

impl IcrcBalanceSource for FixtureIcrcSource {
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
}

impl IcrcAllowanceSource for FixtureIcrcSource {
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

impl IcrcIndexSource for FixtureIcrcSource {
    fn fetch_index(&self, request: &IcrcLedgerRequest) -> Result<IcrcIndexData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);

        Ok(IcrcIndexData {
            index_canister_id: Some(INDEX_CANISTER_ID.to_string()),
            index_error: None,
        })
    }
}

impl IcrcTransactionsSource for FixtureIcrcSource {
    fn fetch_transactions(
        &self,
        request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);
        assert_eq!(request.start, 100);
        assert_eq!(request.limit, 2);
        assert!(request.follow_archives);

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
            followed_archive_blocks: vec![IcrcFollowedArchiveBlockRow {
                archive_canister_id: ARCHIVE_CANISTER_ID.to_string(),
                callback_method: "icrc3_get_blocks".to_string(),
                index: "0".to_string(),
                block_type: Some("1mint".to_string()),
                transaction_kind: Some("1mint".to_string()),
                timestamp_unix_nanos: Some("1699999999123456789".to_string()),
                amount_base_units: Some("987654321".to_string()),
                raw_block: json!({
                    "Map": {
                        "btype": { "Text": "1mint" },
                        "ts": { "Nat": "1699999999123456789" },
                        "tx": {
                            "Map": {
                                "amt": { "Nat": "987654321" }
                            }
                        }
                    }
                }),
            }],
            archive_follow_errors: vec![IcrcArchiveFollowErrorRow {
                callback_canister_id: ARCHIVE_CANISTER_ID.to_string(),
                callback_method: "icrc3_get_blocks".to_string(),
                ranges: vec![IcrcArchivedRangeRow {
                    start: "200".to_string(),
                    length: "10".to_string(),
                }],
                error: "archive query failed".to_string(),
            }],
        })
    }
}

impl IcrcBlockTypesSource for FixtureIcrcSource {
    fn fetch_block_types(
        &self,
        request: &IcrcLedgerRequest,
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
}

impl IcrcArchivesSource for FixtureIcrcSource {
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

impl IcrcTipCertificateSource for FixtureIcrcSource {
    fn fetch_tip_certificate(
        &self,
        request: &IcrcLedgerRequest,
    ) -> Result<IcrcTipCertificateData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);

        Ok(IcrcTipCertificateData {
            certificate_hex: Some("010203".to_string()),
            certificate_bytes: Some(3),
            hash_tree_hex: Some("aabb".to_string()),
            hash_tree_bytes: Some(2),
        })
    }
}

impl IcrcCapabilitiesSource for FixtureIcrcSource {
    fn fetch_capabilities(
        &self,
        request: &IcrcLedgerRequest,
    ) -> Result<IcrcCapabilitiesData, IcrcError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(request.source_endpoint, SOURCE_ENDPOINT);

        Ok(IcrcCapabilitiesData {
            supported_standards: vec![
                IcrcTokenStandardRow {
                    name: "ICRC-1".to_string(),
                    url: "https://github.com/dfinity/ICRC-1".to_string(),
                },
                IcrcTokenStandardRow {
                    name: "ICRC-3".to_string(),
                    url: "https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-3".to_string(),
                },
            ],
            capabilities: vec![
                IcrcCapabilityRow {
                    capability: "ICRC-1 supported standards".to_string(),
                    method: "icrc1_supported_standards".to_string(),
                    status: "available".to_string(),
                    details: Some("2 standard(s)".to_string()),
                    error: None,
                },
                IcrcCapabilityRow {
                    capability: "ICRC-3 tip certificate".to_string(),
                    method: "icrc3_get_tip_certificate".to_string(),
                    status: "unsupported".to_string(),
                    details: Some("method not exported by target canister".to_string()),
                    error: Some("Canister has no query method".to_string()),
                },
            ],
        })
    }
}

struct FixtureAccountTransactionsSource;

impl IcrcAccountTransactionPageSource for FixtureAccountTransactionsSource {
    fn fetch_account_transaction_page(
        &self,
        request: &IcrcAccountTransactionPageRequest,
    ) -> Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError> {
        assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
        assert_eq!(
            request.index_canister_id.as_deref(),
            Some(INDEX_CANISTER_ID)
        );
        assert_eq!(request.account_owner, ACCOUNT_OWNER);
        assert_eq!(
            request.subaccount_hex.as_deref(),
            Some(LOWER_SUBACCOUNT_HEX)
        );
        assert_eq!(request.start.as_deref(), Some("42"));
        assert_eq!(request.limit, 2);

        Ok(fixture_account_transaction_page())
    }
}

struct AccountTransactionsDataSource(IcrcAccountTransactionPageData);

impl IcrcAccountTransactionPageSource for AccountTransactionsDataSource {
    fn fetch_account_transaction_page(
        &self,
        _request: &IcrcAccountTransactionPageRequest,
    ) -> Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError> {
        Ok(self.0.clone())
    }
}

fn fixture_account_transaction_page() -> IcrcAccountTransactionPageData {
    let account = IcrcAccountRow {
        owner: Some(ACCOUNT_OWNER.to_string()),
        subaccount_hex: Some(LOWER_SUBACCOUNT_HEX.to_string()),
        account_identifier: None,
    };
    IcrcAccountTransactionPageData {
        index_canister_id: INDEX_CANISTER_ID.to_string(),
        balance: "250000000".to_string(),
        oldest_transaction_id: Some("7".to_string()),
        next_start: Some("40".to_string()),
        token_symbol: "FIX".to_string(),
        decimals: 8,
        transactions: vec![IcrcAccountTransactionRow {
            id: "40".to_string(),
            kind: "transfer".to_string(),
            timestamp_unix_nanos: Some("1700000000123456789".to_string()),
            amount_base_units: Some("125000000".to_string()),
            fee_base_units: Some("10000".to_string()),
            from: Some(account),
            to: Some(IcrcAccountRow {
                owner: Some(INDEX_CANISTER_ID.to_string()),
                subaccount_hex: None,
                account_identifier: None,
            }),
            spender: None,
            memo_hex: Some("aabb".to_string()),
            created_at_time_unix_nanos: Some("1700000000000000000".to_string()),
            expires_at_unix_nanos: None,
            expected_allowance_base_units: None,
            raw_transaction: json!({
                "kind": "transfer",
                "timestamp_unix_nanos": "1700000000123456789",
                "transfer": {
                    "amount_base_units": "125000000",
                    "fee_base_units": "10000",
                    "from": {
                        "owner": ACCOUNT_OWNER,
                        "subaccount_hex": LOWER_SUBACCOUNT_HEX
                    },
                    "to": {
                        "owner": INDEX_CANISTER_ID,
                        "subaccount_hex": null
                    }
                }
            }),
        }],
    }
}

struct PanickingAccountTransactionsSource;

impl IcrcAccountTransactionPageSource for PanickingAccountTransactionsSource {
    fn fetch_account_transaction_page(
        &self,
        _request: &IcrcAccountTransactionPageRequest,
    ) -> Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError> {
        panic!("account transactions source should not be called")
    }
}

struct PanickingIcrcSource;

impl IcrcBalanceSource for PanickingIcrcSource {
    fn fetch_balance(&self, _request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
        panic!("balance source should not be called")
    }
}

impl IcrcAllowanceSource for PanickingIcrcSource {
    fn fetch_allowance(
        &self,
        _request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError> {
        panic!("allowance source should not be called")
    }
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
fn icrc3_tip_certificate_shape_round_trips_through_candid() {
    let certificate = Some(Icrc3DataCertificate {
        certificate: vec![1, 2, 3],
        hash_tree: vec![0xaa, 0xbb],
    });

    let bytes = candid::encode_one(&certificate).expect("encode ICRC-3 data certificate");
    let decoded: Option<Icrc3DataCertificate> =
        candid::decode_one(&bytes).expect("decode ICRC-3 data certificate");

    assert_eq!(decoded, certificate);

    let bytes = candid::encode_one(None::<Icrc3DataCertificate>)
        .expect("encode absent ICRC-3 data certificate");
    let decoded: Option<Icrc3DataCertificate> =
        candid::decode_one(&bytes).expect("decode absent ICRC-3 data certificate");

    assert_eq!(decoded, None);
}

#[test]
fn token_report_builds_text_and_json_friendly_fields() {
    let request = IcrcLedgerRequest {
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
    assert_eq!(
        json["metadata"][2]["value"],
        json!("data:image/png;base64,fixture")
    );
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
    let request = IcrcLedgerRequest {
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
fn account_transaction_page_report_preserves_index_provenance_and_cursor() {
    let request = IcrcAccountTransactionPageRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        2,
    )
    .with_index_canister_id(INDEX_CANISTER_ID)
    .with_subaccount_hex(UPPER_SUBACCOUNT_HEX)
    .with_start("42");

    let report = build_icrc_account_transaction_page_report_with_source(
        &request,
        &FixtureAccountTransactionsSource,
    )
    .expect("build ICRC account transaction page report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(report.index_canister_id, INDEX_CANISTER_ID);
    assert_eq!(report.subaccount_hex.as_deref(), Some(LOWER_SUBACCOUNT_HEX));
    assert_eq!(report.requested_start.as_deref(), Some("42"));
    assert_eq!(report.next_start.as_deref(), Some("40"));
    assert_eq!(report.oldest_transaction_id.as_deref(), Some("7"));
    assert_eq!(report.balance, "250000000");
    assert_eq!(report.transactions[0].memo_hex.as_deref(), Some("aabb"));

    let text = icrc_account_transaction_page_report_text(&report);
    assert!(text.contains(&format!("index_canister_id: {INDEX_CANISTER_ID}")));
    assert!(text.contains("requested_start: 42"));
    assert!(text.contains("next_start: 40"));
    assert!(text.contains("balance: 2.50 FIX"));
    assert!(text.contains("transfer"));

    let json = serde_json::to_value(&report).expect("serialize account transaction page report");
    assert_eq!(json["index_canister_id"], json!(INDEX_CANISTER_ID));
    assert_eq!(json["requested_start"], json!("42"));
    assert_eq!(json["next_start"], json!("40"));
    assert_eq!(
        json["transactions"][0]["raw_transaction"]["transfer"]["amount_base_units"],
        json!("125000000")
    );
}

#[test]
fn account_transaction_page_report_rejects_zero_limit_before_source_call() {
    let request = IcrcAccountTransactionPageRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        0,
    );

    let error = build_icrc_account_transaction_page_report_with_source(
        &request,
        &PanickingAccountTransactionsSource,
    )
    .expect_err("zero limit must fail");

    assert!(matches!(
        error,
        IcrcAccountTransactionError::InvalidPageSize {
            page_size: 0,
            max_page_size: 100,
        }
    ));
}

#[test]
fn account_transaction_page_report_rejects_explicit_index_mismatch() {
    let request = IcrcAccountTransactionPageRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        2,
    )
    .with_index_canister_id(INDEX_CANISTER_ID);
    let mut page = fixture_account_transaction_page();
    page.index_canister_id = ARCHIVE_CANISTER_ID.to_string();

    let error = build_icrc_account_transaction_page_report_with_source(
        &request,
        &AccountTransactionsDataSource(page),
    )
    .expect_err("source index mismatch must fail");

    assert!(matches!(
        error,
        IcrcAccountTransactionError::CollectionIndexMismatch {
            expected_index_canister_id,
            actual_index_canister_id,
        } if expected_index_canister_id == INDEX_CANISTER_ID
            && actual_index_canister_id == ARCHIVE_CANISTER_ID
    ));
}

#[test]
fn account_transaction_page_report_rejects_noncanonical_source_cursor() {
    let request = IcrcAccountTransactionPageRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        2,
    );
    let mut page = fixture_account_transaction_page();
    page.next_start = Some("040".to_string());

    let error = build_icrc_account_transaction_page_report_with_source(
        &request,
        &AccountTransactionsDataSource(page),
    )
    .expect_err("noncanonical source cursor must fail");

    assert!(matches!(
        error,
        IcrcAccountTransactionError::InvalidPage { reason }
            if reason.contains("next_start") && reason.contains("not canonical")
    ));
}

#[test]
fn account_transaction_page_report_rejects_excess_or_unordered_rows() {
    let request = IcrcAccountTransactionPageRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        1,
    );
    let mut excessive = fixture_account_transaction_page();
    excessive
        .transactions
        .push(excessive.transactions[0].clone());

    let error = build_icrc_account_transaction_page_report_with_source(
        &request,
        &AccountTransactionsDataSource(excessive),
    )
    .expect_err("excessive page must fail");
    assert!(matches!(
        error,
        IcrcAccountTransactionError::InvalidPage { .. }
    ));

    let mut unordered = fixture_account_transaction_page();
    let mut older = unordered.transactions[0].clone();
    older.id = "41".to_string();
    unordered.transactions.push(older);
    unordered.next_start = Some("41".to_string());
    let request = IcrcAccountTransactionPageRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        2,
    );

    let error = build_icrc_account_transaction_page_report_with_source(
        &request,
        &AccountTransactionsDataSource(unordered),
    )
    .expect_err("unordered page must fail");
    assert!(matches!(
        error,
        IcrcAccountTransactionError::InvalidPage { .. }
    ));
}

#[test]
fn transactions_report_builds_text_and_json_friendly_fields() {
    let request = IcrcTransactionsRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        start: 100,
        limit: 2,
        follow_archives: true,
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
    assert_eq!(report.followed_archive_blocks.len(), 1);
    assert_eq!(report.archive_follow_errors.len(), 1);

    let text = icrc_transactions_report_text(&report);
    assert!(text.contains("requested_start: 100"));
    assert!(text.contains("follow_archives: true"));
    assert!(text.contains("returned_blocks: 2"));
    assert!(text.contains("followed_archive_blocks: 1"));
    assert!(text.contains("archive_follow_errors: 1"));
    assert!(text.contains("1xfer"));
    assert!(text.contains("1mint"));
    assert!(text.contains("1700000000123456789"));
    assert!(text.contains("qaa6y-5yaaa-aaaaa-aaafa-cai"));
    assert!(text.contains("archive query failed"));

    let json = serde_json::to_value(&report).expect("serialize ICRC transactions report");
    assert_eq!(json["requested_start"], json!("100"));
    assert_eq!(json["requested_limit"], json!(2));
    assert_eq!(json["follow_archives"], json!(true));
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
    assert_eq!(json["followed_archive_blocks"][0]["index"], json!("0"));
    assert_eq!(
        json["followed_archive_blocks"][0]["archive_canister_id"],
        json!(ARCHIVE_CANISTER_ID)
    );
    assert_eq!(
        json["followed_archive_blocks"][0]["amount_base_units"],
        json!("987654321")
    );
    assert_eq!(
        json["archive_follow_errors"][0]["error"],
        json!("archive query failed")
    );
}

#[test]
fn block_types_report_builds_text_and_json_friendly_fields() {
    let request = IcrcLedgerRequest {
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
fn tip_certificate_report_builds_text_and_json_friendly_fields() {
    let request = IcrcLedgerRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = build_icrc_tip_certificate_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC tip certificate report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert!(report.certificate_present);
    assert_eq!(report.certificate_hex.as_deref(), Some("010203"));
    assert_eq!(report.certificate_bytes, Some(3));
    assert_eq!(report.hash_tree_hex.as_deref(), Some("aabb"));
    assert_eq!(report.hash_tree_bytes, Some(2));

    let text = icrc_tip_certificate_report_text(&report);
    assert!(text.contains("certificate_present: true"));
    assert!(text.contains("certificate_bytes: 3"));
    assert!(text.contains("hash_tree_bytes: 2"));
    assert!(text.contains("certificate_hex: 010203"));
    assert!(text.contains("hash_tree_hex: aabb"));

    let json = serde_json::to_value(&report).expect("serialize ICRC tip certificate report");
    assert_eq!(json["certificate_present"], json!(true));
    assert_eq!(json["certificate_hex"], json!("010203"));
    assert_eq!(json["certificate_bytes"], json!(3));
    assert_eq!(json["hash_tree_hex"], json!("aabb"));
    assert_eq!(json["hash_tree_bytes"], json!(2));
}

#[test]
fn capabilities_report_builds_text_and_json_friendly_fields() {
    let request = IcrcLedgerRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = build_icrc_capabilities_report_with_source(&request, &FixtureIcrcSource)
        .expect("build ICRC capabilities report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(report.supported_standards.len(), 2);
    assert_eq!(report.capabilities.len(), 2);
    assert_eq!(report.capabilities[0].status, "available");
    assert_eq!(report.capabilities[1].status, "unsupported");

    let text = icrc_capabilities_report_text(&report);
    assert!(text.contains("standard_count: 2"));
    assert!(text.contains("capability_count: 2"));
    assert!(text.contains("ICRC-1 supported standards"));
    assert!(text.contains("icrc3_get_tip_certificate"));
    assert!(text.contains("unsupported"));

    let json = serde_json::to_value(&report).expect("serialize ICRC capabilities report");
    assert_eq!(json["supported_standards"][0]["name"], json!("ICRC-1"));
    assert_eq!(json["capabilities"][0]["status"], json!("available"));
    assert_eq!(
        json["capabilities"][1]["details"],
        json!("method not exported by target canister")
    );
    assert_eq!(
        json["capabilities"][1]["error"],
        json!("Canister has no query method")
    );
}

#[test]
fn index_report_renders_index_error_when_not_set() {
    struct MissingIndexSource;

    impl IcrcIndexSource for MissingIndexSource {
        fn fetch_index(&self, _request: &IcrcLedgerRequest) -> Result<IcrcIndexData, IcrcError> {
            Ok(IcrcIndexData {
                index_canister_id: None,
                index_error: Some("index principal not set".to_string()),
            })
        }
    }

    let request = IcrcLedgerRequest {
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
