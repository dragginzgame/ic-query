use ic_query::icrc::{
    DEFAULT_ICRC_SOURCE_ENDPOINT, IcrcAllowanceReport, IcrcAllowanceRequest,
    IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow, IcrcArchivedRangeRow,
    IcrcArchivesReport, IcrcArchivesRequest, IcrcBalanceReport, IcrcBalanceRequest,
    IcrcBlockTypeRow, IcrcBlockTypesReport, IcrcBlockTypesRequest, IcrcCapabilitiesReport,
    IcrcCapabilitiesRequest, IcrcCapabilityRow, IcrcFollowedArchiveBlockRow, IcrcIndexReport,
    IcrcIndexRequest, IcrcTipCertificateReport, IcrcTipCertificateRequest, IcrcTokenMetadataRow,
    IcrcTokenReport, IcrcTokenRequest, IcrcTokenStandardRow, IcrcTransactionBlockRow,
    IcrcTransactionsReport, IcrcTransactionsRequest, icrc_allowance_report_text,
    icrc_archives_report_text, icrc_balance_report_text, icrc_block_types_report_text,
    icrc_capabilities_report_text, icrc_index_report_text, icrc_tip_certificate_report_text,
    icrc_token_report_text, icrc_transactions_report_text,
};
#[cfg(feature = "host")]
use ic_query::icrc::{
    IcrcError, build_icrc_allowance_report, build_icrc_archives_report, build_icrc_balance_report,
    build_icrc_block_types_report, build_icrc_capabilities_report, build_icrc_index_report,
    build_icrc_tip_certificate_report, build_icrc_token_report, build_icrc_transactions_report,
};
use serde_json::json;

const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const ACCOUNT_OWNER: &str = "aaaaa-aa";
const ARCHIVE_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";
const SOURCE_ENDPOINT: &str = "https://icp-api.io";
const FETCHED_AT: &str = "2023-11-14T22:13:20Z";
const FETCHED_AT_UNIX_SECS: u64 = 1_700_000_000;
const FETCHED_BY: &str = "ic-query";
const SUBACCOUNT_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";

#[cfg(feature = "host")]
type IcrcTokenBuilder = fn(&IcrcTokenRequest) -> Result<IcrcTokenReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcBalanceBuilder = fn(&IcrcBalanceRequest) -> Result<IcrcBalanceReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcAllowanceBuilder = fn(&IcrcAllowanceRequest) -> Result<IcrcAllowanceReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcIndexBuilder = fn(&IcrcIndexRequest) -> Result<IcrcIndexReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcTransactionsBuilder =
    fn(&IcrcTransactionsRequest) -> Result<IcrcTransactionsReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcBlockTypesBuilder = fn(&IcrcBlockTypesRequest) -> Result<IcrcBlockTypesReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcArchivesBuilder = fn(&IcrcArchivesRequest) -> Result<IcrcArchivesReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcTipCertificateBuilder =
    fn(&IcrcTipCertificateRequest) -> Result<IcrcTipCertificateReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcCapabilitiesBuilder =
    fn(&IcrcCapabilitiesRequest) -> Result<IcrcCapabilitiesReport, IcrcError>;

#[test]
fn public_icrc_request_constructors_set_expected_fields() {
    assert_eq!(DEFAULT_ICRC_SOURCE_ENDPOINT, SOURCE_ENDPOINT);

    let token = IcrcTokenRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID);
    assert_eq!(token.source_endpoint, SOURCE_ENDPOINT);
    assert_eq!(token.now_unix_secs, FETCHED_AT_UNIX_SECS);
    assert_eq!(token.ledger_canister_id, LEDGER_CANISTER_ID);

    let balance = IcrcBalanceRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
    )
    .with_subaccount_hex(SUBACCOUNT_HEX);
    assert_eq!(balance.account_owner, ACCOUNT_OWNER);
    assert_eq!(balance.subaccount_hex.as_deref(), Some(SUBACCOUNT_HEX));

    let allowance = IcrcAllowanceRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        ARCHIVE_CANISTER_ID,
    )
    .with_account_subaccount_hex(SUBACCOUNT_HEX)
    .with_spender_subaccount_hex(SUBACCOUNT_HEX);
    assert_eq!(allowance.account_owner, ACCOUNT_OWNER);
    assert_eq!(allowance.spender_owner, ARCHIVE_CANISTER_ID);
    assert_eq!(
        allowance.account_subaccount_hex.as_deref(),
        Some(SUBACCOUNT_HEX)
    );
    assert_eq!(
        allowance.spender_subaccount_hex.as_deref(),
        Some(SUBACCOUNT_HEX)
    );

    let index = IcrcIndexRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID);
    assert_eq!(index.ledger_canister_id, LEDGER_CANISTER_ID);

    let transactions = IcrcTransactionsRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        100,
        25,
    )
    .with_follow_archives(true);
    assert_eq!(transactions.start, 100);
    assert_eq!(transactions.limit, 25);
    assert!(transactions.follow_archives);

    let block_types =
        IcrcBlockTypesRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID);
    assert_eq!(block_types.ledger_canister_id, LEDGER_CANISTER_ID);

    let archives =
        IcrcArchivesRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID)
            .with_from_canister_id(ARCHIVE_CANISTER_ID);
    assert_eq!(
        archives.from_canister_id.as_deref(),
        Some(ARCHIVE_CANISTER_ID)
    );

    let tip_certificate =
        IcrcTipCertificateRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID);
    assert_eq!(tip_certificate.source_endpoint, SOURCE_ENDPOINT);

    let capabilities =
        IcrcCapabilitiesRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID);
    assert_eq!(capabilities.now_unix_secs, FETCHED_AT_UNIX_SECS);
}

#[cfg(feature = "host")]
#[test]
fn public_icrc_host_api_exposes_live_builder_entry_points() {
    accepts_public_function::<IcrcTokenBuilder>(build_icrc_token_report);
    accepts_public_function::<IcrcBalanceBuilder>(build_icrc_balance_report);
    accepts_public_function::<IcrcAllowanceBuilder>(build_icrc_allowance_report);
    accepts_public_function::<IcrcIndexBuilder>(build_icrc_index_report);
    accepts_public_function::<IcrcTransactionsBuilder>(build_icrc_transactions_report);
    accepts_public_function::<IcrcBlockTypesBuilder>(build_icrc_block_types_report);
    accepts_public_function::<IcrcArchivesBuilder>(build_icrc_archives_report);
    accepts_public_function::<IcrcTipCertificateBuilder>(build_icrc_tip_certificate_report);
    accepts_public_function::<IcrcCapabilitiesBuilder>(build_icrc_capabilities_report);
}

#[cfg(feature = "host")]
fn accepts_public_function<T>(_function: T) {}

#[test]
fn public_icrc_token_api_is_constructible_and_renderable_without_host() {
    let request = IcrcTokenRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcTokenReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        token_name: "Internet Computer".to_string(),
        token_symbol: "ICP".to_string(),
        decimals: 8,
        transfer_fee: "10000".to_string(),
        total_supply: "100000000".to_string(),
        minting_account_owner: None,
        minting_account_subaccount_hex: None,
        supported_standards: vec![standard_row("ICRC-1")],
        metadata: vec![IcrcTokenMetadataRow {
            key: "icrc1:symbol".to_string(),
            value_type: "Text".to_string(),
            value: json!("ICP"),
        }],
    };

    let text = icrc_token_report_text(&report);

    assert!(text.contains(&format!("ledger_canister_id: {LEDGER_CANISTER_ID}")));
    assert!(text.contains("token_symbol: ICP"));
    assert!(text.contains("ICRC-1"));
}

#[test]
fn public_icrc_balance_api_is_constructible_and_renderable_without_host() {
    let request = IcrcBalanceRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        account_owner: ACCOUNT_OWNER.to_string(),
        subaccount_hex: None,
    };

    let report = IcrcBalanceReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        token_symbol: "ICP".to_string(),
        decimals: 8,
        balance: "100000000".to_string(),
    };

    let text = icrc_balance_report_text(&report);

    assert!(text.contains("account_owner: aaaaa-aa"));
    assert!(text.contains("balance: 1.00 ICP"));
    assert!(text.contains("balance_base_units: 100000000"));
}

#[test]
fn public_icrc_allowance_api_is_constructible_and_renderable_without_host() {
    let request = IcrcAllowanceRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        account_owner: ACCOUNT_OWNER.to_string(),
        account_subaccount_hex: None,
        spender_owner: ACCOUNT_OWNER.to_string(),
        spender_subaccount_hex: None,
    };

    let report = IcrcAllowanceReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        account_owner: request.account_owner,
        account_subaccount_hex: request.account_subaccount_hex,
        spender_owner: request.spender_owner,
        spender_subaccount_hex: request.spender_subaccount_hex,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        token_symbol: "ICP".to_string(),
        decimals: 8,
        allowance: "50000000".to_string(),
        expires_at_unix_nanos: Some("1700000000123456789".to_string()),
    };

    let text = icrc_allowance_report_text(&report);

    assert!(text.contains("spender_owner: aaaaa-aa"));
    assert!(text.contains("allowance: 0.50 ICP"));
    assert!(text.contains("expires_at_unix_nanos: 1700000000123456789"));
}

#[test]
fn public_icrc_index_api_is_constructible_and_renderable_without_host() {
    let request = IcrcIndexRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcIndexReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        index_canister_id: None,
        index_error: Some("not configured".to_string()),
    };

    let text = icrc_index_report_text(&report);

    assert!(text.contains("index_canister_id: -"));
    assert!(text.contains("index_error: not configured"));
}

#[test]
fn public_icrc_transactions_api_is_constructible_and_renderable_without_host() {
    let request = IcrcTransactionsRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        start: 100,
        limit: 2,
        follow_archives: true,
    };

    let report = IcrcTransactionsReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        requested_start: request.start.to_string(),
        requested_limit: request.limit,
        follow_archives: request.follow_archives,
        log_length: Some("1000".to_string()),
        blocks: vec![IcrcTransactionBlockRow {
            index: "100".to_string(),
            block_type: Some("1xfer".to_string()),
            transaction_kind: Some("1xfer".to_string()),
            timestamp_unix_nanos: Some("1700000000123456789".to_string()),
            amount_base_units: Some("100000000".to_string()),
            raw_block: json!({"Map": {"btype": {"Text": "1xfer"}}}),
        }],
        archived_blocks: vec![IcrcArchivedBlocksRow {
            callback_canister_id: ARCHIVE_CANISTER_ID.to_string(),
            callback_method: "icrc3_get_blocks".to_string(),
            ranges: vec![archive_range_row()],
        }],
        followed_archive_blocks: vec![IcrcFollowedArchiveBlockRow {
            archive_canister_id: ARCHIVE_CANISTER_ID.to_string(),
            callback_method: "icrc3_get_blocks".to_string(),
            index: "0".to_string(),
            block_type: Some("1mint".to_string()),
            transaction_kind: Some("1mint".to_string()),
            timestamp_unix_nanos: Some("1699999999123456789".to_string()),
            amount_base_units: Some("50000000".to_string()),
            raw_block: json!({"Map": {"btype": {"Text": "1mint"}}}),
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
    };

    let text = icrc_transactions_report_text(&report);

    assert!(text.contains("requested_start: 100"));
    assert!(text.contains("follow_archives: true"));
    assert!(text.contains("archive_follow_errors: 1"));
    assert!(text.contains("archive query failed"));
}

#[test]
fn public_icrc_block_types_api_is_constructible_and_renderable_without_host() {
    let request = IcrcBlockTypesRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcBlockTypesReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        block_types: vec![IcrcBlockTypeRow {
            block_type: "1xfer".to_string(),
            url: "https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-3".to_string(),
        }],
    };

    let text = icrc_block_types_report_text(&report);

    assert!(text.contains("block_type_count: 1"));
    assert!(text.contains("1xfer"));
}

#[test]
fn public_icrc_archives_api_is_constructible_and_renderable_without_host() {
    let request = IcrcArchivesRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        from_canister_id: Some(ARCHIVE_CANISTER_ID.to_string()),
    };

    let report = IcrcArchivesReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        from_canister_id: request.from_canister_id,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        archives: vec![IcrcArchiveRow {
            canister_id: ARCHIVE_CANISTER_ID.to_string(),
            start: "0".to_string(),
            end: "999".to_string(),
        }],
    };

    let text = icrc_archives_report_text(&report);

    assert!(text.contains(&format!("from_canister_id: {ARCHIVE_CANISTER_ID}")));
    assert!(text.contains("archive_count: 1"));
    assert!(text.contains("999"));
}

#[test]
fn public_icrc_tip_certificate_api_is_constructible_and_renderable_without_host() {
    let request = IcrcTipCertificateRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcTipCertificateReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        certificate_present: true,
        certificate_hex: Some("010203".to_string()),
        certificate_bytes: Some(3),
        hash_tree_hex: Some("aabb".to_string()),
        hash_tree_bytes: Some(2),
    };

    let text = icrc_tip_certificate_report_text(&report);

    assert!(text.contains("certificate_present: true"));
    assert!(text.contains("certificate_bytes: 3"));
    assert!(text.contains("hash_tree_hex: aabb"));
}

#[test]
fn public_icrc_capabilities_api_is_constructible_and_renderable_without_host() {
    let request = IcrcCapabilitiesRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcCapabilitiesReport {
        schema_version: 1,
        ledger_canister_id: request.ledger_canister_id,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: FETCHED_BY.to_string(),
        supported_standards: vec![standard_row("ICRC-1")],
        capabilities: vec![IcrcCapabilityRow {
            capability: "ICRC-3 tip certificate".to_string(),
            method: "icrc3_get_tip_certificate".to_string(),
            status: "unsupported".to_string(),
            details: None,
            error: Some("Canister has no query method".to_string()),
        }],
    };

    let text = icrc_capabilities_report_text(&report);

    assert!(text.contains("standard_count: 1"));
    assert!(text.contains("capability_count: 1"));
    assert!(text.contains("Canister has no query method"));
}

fn standard_row(name: &str) -> IcrcTokenStandardRow {
    IcrcTokenStandardRow {
        name: name.to_string(),
        url: format!("https://example.com/{name}"),
    }
}

fn archive_range_row() -> IcrcArchivedRangeRow {
    IcrcArchivedRangeRow {
        start: "0".to_string(),
        length: "100".to_string(),
    }
}
