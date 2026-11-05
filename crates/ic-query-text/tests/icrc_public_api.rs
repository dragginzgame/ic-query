#[cfg(feature = "icrc-host")]
use ic_query::QueryProgress;
use ic_query::icrc::DEFAULT_ICRC_SOURCE_ENDPOINT;
use ic_query::icrc::IcrcAccountRow;
use ic_query::icrc::IcrcAccountTransactionCacheRequest;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcAccountTransactionCollectionData;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcAccountTransactionCollectionSource;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcAccountTransactionError;
use ic_query::icrc::IcrcAccountTransactionListRequest;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcAccountTransactionPageData;
use ic_query::icrc::IcrcAccountTransactionPageReport;
use ic_query::icrc::IcrcAccountTransactionPageRequest;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcAccountTransactionPageSource;
use ic_query::icrc::IcrcAccountTransactionRefreshRequest;
use ic_query::icrc::IcrcAccountTransactionRow;
use ic_query::icrc::IcrcAccountTransactionSort;
use ic_query::icrc::IcrcAllowanceData;
use ic_query::icrc::IcrcAllowanceReport;
use ic_query::icrc::IcrcAllowanceRequest;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcAllowanceSource;
use ic_query::icrc::IcrcArchiveFollowErrorRow;
use ic_query::icrc::IcrcArchiveRow;
use ic_query::icrc::IcrcArchivedBlocksRow;
use ic_query::icrc::IcrcArchivedRangeRow;
use ic_query::icrc::IcrcArchivesReport;
use ic_query::icrc::IcrcArchivesRequest;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcArchivesSource;
use ic_query::icrc::IcrcBalanceData;
use ic_query::icrc::IcrcBalanceReport;
use ic_query::icrc::IcrcBalanceRequest;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcBalanceSource;
use ic_query::icrc::IcrcBlockTypeRow;
use ic_query::icrc::IcrcBlockTypesReport;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcBlockTypesSource;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcCapabilitiesData;
use ic_query::icrc::IcrcCapabilitiesReport;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcCapabilitiesSource;
use ic_query::icrc::IcrcCapabilityRow;
use ic_query::icrc::IcrcCapabilityStatus;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcError;
use ic_query::icrc::IcrcFollowedArchiveBlockRow;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcIndexData;
use ic_query::icrc::IcrcIndexReport;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcIndexSource;
use ic_query::icrc::IcrcLedgerRequest;
use ic_query::icrc::IcrcMetadataValueKind;
use ic_query::icrc::IcrcTipCertificateReport;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcTipCertificateSource;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcTokenData;
use ic_query::icrc::IcrcTokenMetadataRow;
use ic_query::icrc::IcrcTokenReport;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcTokenSource;
use ic_query::icrc::IcrcTokenStandardRow;
use ic_query::icrc::IcrcTransactionBlockRow;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcTransactionsData;
use ic_query::icrc::IcrcTransactionsReport;
use ic_query::icrc::IcrcTransactionsRequest;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::IcrcTransactionsSource;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_account_transaction_page_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_allowance_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_archives_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_balance_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_block_types_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_capabilities_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_index_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_tip_certificate_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_token_report_with_source;
#[cfg(feature = "icrc-host")]
use ic_query::icrc::build_icrc_transactions_report_with_source;
use ic_query::icrc::normalize_subaccount_hex;
use ic_query_text::icrc::icrc_account_transaction_page_report_text;
use ic_query_text::icrc::icrc_allowance_report_text;
use ic_query_text::icrc::icrc_archives_report_text;
use ic_query_text::icrc::icrc_balance_report_text;
use ic_query_text::icrc::icrc_block_types_report_text;
use ic_query_text::icrc::icrc_capabilities_report_text;
use ic_query_text::icrc::icrc_index_report_text;
use ic_query_text::icrc::icrc_tip_certificate_report_text;
use ic_query_text::icrc::icrc_token_report_text;
use ic_query_text::icrc::icrc_transactions_report_text;
use serde_json::json;

const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const ACCOUNT_OWNER: &str = "aaaaa-aa";
const ARCHIVE_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";
const SOURCE_ENDPOINT: &str = "https://icp-api.io";
const FETCHED_AT: &str = "2023-11-14T22:13:20Z";
const FETCHED_AT_UNIX_SECS: u64 = 1_700_000_000;
const FETCHED_BY: &str = "ic-query";
const SUBACCOUNT_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";

#[test]
fn public_icrc_subaccount_normalization_is_available_without_host() {
    assert_eq!(
        normalize_subaccount_hex(&"AA".repeat(32)).expect("valid subaccount"),
        "aa".repeat(32)
    );
}

#[test]
fn public_icrc_request_constructors_set_expected_fields() {
    assert_eq!(DEFAULT_ICRC_SOURCE_ENDPOINT, SOURCE_ENDPOINT);

    let ledger = IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID);
    assert_eq!(ledger.source_endpoint, SOURCE_ENDPOINT);
    assert_eq!(ledger.now_unix_secs, FETCHED_AT_UNIX_SECS);
    assert_eq!(ledger.ledger_canister_id, LEDGER_CANISTER_ID);

    let mut balance = IcrcBalanceRequest::new(
        ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
        ),
        ACCOUNT_OWNER,
    );
    balance.subaccount_hex = Some(SUBACCOUNT_HEX.into());

    assert_eq!(balance.account_owner, ACCOUNT_OWNER);
    assert_eq!(balance.subaccount_hex.as_deref(), Some(SUBACCOUNT_HEX));

    let mut allowance = IcrcAllowanceRequest::new(
        ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
        ),
        ACCOUNT_OWNER,
        ARCHIVE_CANISTER_ID,
    );
    allowance.account_subaccount_hex = Some(SUBACCOUNT_HEX.into());
    allowance.spender_subaccount_hex = Some(SUBACCOUNT_HEX.into());

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

    let mut account_transactions = IcrcAccountTransactionPageRequest::new(
        ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
        ),
        ACCOUNT_OWNER,
        25,
    );
    account_transactions.index_canister_id = Some(ARCHIVE_CANISTER_ID.into());
    account_transactions.subaccount_hex = Some(SUBACCOUNT_HEX.into());
    account_transactions.start = Some("100".into());

    assert_eq!(
        account_transactions.index_canister_id.as_deref(),
        Some(ARCHIVE_CANISTER_ID)
    );
    assert_eq!(account_transactions.account_owner, ACCOUNT_OWNER);
    assert_eq!(
        account_transactions.subaccount_hex.as_deref(),
        Some(SUBACCOUNT_HEX)
    );
    assert_eq!(account_transactions.start.as_deref(), Some("100"));
    assert_eq!(account_transactions.limit, 25);

    let mut cache = IcrcAccountTransactionCacheRequest::new(
        "/tmp/ic-query-cache",
        SOURCE_ENDPOINT,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
    );
    cache.subaccount_hex = Some(SUBACCOUNT_HEX.into());

    let mut refresh =
        IcrcAccountTransactionRefreshRequest::new(cache.clone(), FETCHED_AT_UNIX_SECS, 100);
    refresh.index_canister_id = Some(ARCHIVE_CANISTER_ID.into());
    refresh.max_pages = Some(50);

    let mut list = IcrcAccountTransactionListRequest::new(cache, 25);
    list.sort = IcrcAccountTransactionSort::Oldest;

    assert_eq!(refresh.page_size, 100);
    assert_eq!(refresh.max_pages, Some(50));
    assert_eq!(list.sort, IcrcAccountTransactionSort::Oldest);

    let mut transactions = IcrcTransactionsRequest::new(
        ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
        ),
        100,
        25,
    );
    transactions.follow_archives = true;

    assert_eq!(transactions.start, 100);
    assert_eq!(transactions.limit, 25);
    assert!(transactions.follow_archives);

    let mut archives = IcrcArchivesRequest::new(ic_query::icrc::IcrcLedgerRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
    ));
    archives.from_canister_id = Some(ARCHIVE_CANISTER_ID.into());

    assert_eq!(
        archives.from_canister_id.as_deref(),
        Some(ARCHIVE_CANISTER_ID)
    );
}

#[cfg(feature = "icrc-host")]
#[tokio::test(flavor = "current_thread")]
#[expect(
    clippy::too_many_lines,
    reason = "One coherent public host API signature admission fixture."
)]
async fn public_icrc_host_api_accepts_custom_source_adapters() {
    let source = FixtureIcrcSource;
    let token_request =
        IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID);
    let report = build_icrc_token_report_with_source(&token_request, &source)
        .await
        .expect("token report");

    assert_eq!(report.provenance.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(report.token.token_symbol, "FIX");
    assert_eq!(
        report.token.supported_standards,
        vec![standard_row("ICRC-1")]
    );

    build_icrc_balance_report_with_source(
        &IcrcBalanceRequest::new(
            ic_query::icrc::IcrcLedgerRequest::new(
                SOURCE_ENDPOINT,
                FETCHED_AT_UNIX_SECS,
                LEDGER_CANISTER_ID,
            ),
            ACCOUNT_OWNER,
        ),
        &source,
    )
    .await
    .expect("balance report");
    build_icrc_allowance_report_with_source(
        &IcrcAllowanceRequest::new(
            ic_query::icrc::IcrcLedgerRequest::new(
                SOURCE_ENDPOINT,
                FETCHED_AT_UNIX_SECS,
                LEDGER_CANISTER_ID,
            ),
            ACCOUNT_OWNER,
            ACCOUNT_OWNER,
        ),
        &source,
    )
    .await
    .expect("allowance report");
    build_icrc_account_transaction_page_report_with_source(
        &IcrcAccountTransactionPageRequest::new(
            ic_query::icrc::IcrcLedgerRequest::new(
                SOURCE_ENDPOINT,
                FETCHED_AT_UNIX_SECS,
                LEDGER_CANISTER_ID,
            ),
            ACCOUNT_OWNER,
            1,
        ),
        &source,
    )
    .await
    .expect("account transaction page report");
    let mut progress = |_event| {};
    source
        .fetch_complete_account_transactions(
            &IcrcAccountTransactionRefreshRequest::new(
                IcrcAccountTransactionCacheRequest::new(
                    "/tmp/ic-query-cache",
                    SOURCE_ENDPOINT,
                    LEDGER_CANISTER_ID,
                    ACCOUNT_OWNER,
                ),
                FETCHED_AT_UNIX_SECS,
                100,
            ),
            &mut progress,
        )
        .await
        .expect("complete account transaction source");
    build_icrc_index_report_with_source(
        &IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .await
    .expect("index report");
    build_icrc_transactions_report_with_source(
        &IcrcTransactionsRequest::new(
            ic_query::icrc::IcrcLedgerRequest::new(
                SOURCE_ENDPOINT,
                FETCHED_AT_UNIX_SECS,
                LEDGER_CANISTER_ID,
            ),
            0,
            1,
        ),
        &source,
    )
    .await
    .expect("transactions report");
    build_icrc_block_types_report_with_source(
        &IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .await
    .expect("block types report");
    build_icrc_archives_report_with_source(
        &IcrcArchivesRequest::new(ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
        )),
        &source,
    )
    .await
    .expect("archives report");
    build_icrc_tip_certificate_report_with_source(
        &IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .await
    .expect("tip certificate report");
    build_icrc_capabilities_report_with_source(
        &IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .await
    .expect("capabilities report");
}

#[cfg(feature = "icrc-host")]
struct FixtureIcrcSource;

#[cfg(feature = "icrc-host")]
impl IcrcTokenSource for FixtureIcrcSource {
    fn fetch_token<'a>(
        &'a self,
        request: &'a IcrcLedgerRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<IcrcTokenData, IcrcError>> {
        Box::pin(async move {
            assert_eq!(request.ledger_canister_id, LEDGER_CANISTER_ID);
            Ok(IcrcTokenData {
                token_name: "Fixture Token".to_string(),
                token_symbol: "FIX".to_string(),
                decimals: 8,
                transfer_fee: "10000".to_string(),
                total_supply: "100000000".to_string(),
                minting_account_owner: None,
                minting_account_subaccount_hex: None,
                supported_standards: vec![standard_row("ICRC-1")],
                metadata: Vec::new(),
            })
        })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcBalanceSource for FixtureIcrcSource {
    fn fetch_balance<'a>(
        &'a self,
        _request: &'a IcrcBalanceRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<IcrcBalanceData, IcrcError>> {
        Box::pin(async move {
            Ok(IcrcBalanceData {
                token_symbol: "FIX".to_string(),
                decimals: 8,
                balance: "0".to_string(),
            })
        })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcAllowanceSource for FixtureIcrcSource {
    fn fetch_allowance<'a>(
        &'a self,
        _request: &'a IcrcAllowanceRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<IcrcAllowanceData, IcrcError>> {
        Box::pin(async move {
            Ok(IcrcAllowanceData {
                token_symbol: "FIX".to_string(),
                decimals: 8,
                allowance: "0".to_string(),
                expires_at_unix_nanos: None,
            })
        })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcIndexSource for FixtureIcrcSource {
    fn fetch_index<'a>(
        &'a self,
        _request: &'a IcrcLedgerRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<IcrcIndexData, IcrcError>> {
        Box::pin(async move {
            Ok(IcrcIndexData::Unavailable(
                "index principal not set".to_string(),
            ))
        })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcTransactionsSource for FixtureIcrcSource {
    fn fetch_transactions<'a>(
        &'a self,
        _request: &'a IcrcTransactionsRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<IcrcTransactionsData, IcrcError>> {
        Box::pin(async move {
            Ok(IcrcTransactionsData {
                log_length: Some("0".to_string()),
                blocks: Vec::new(),
                archived_blocks: Vec::new(),
                followed_archive_blocks: Vec::new(),
                archive_follow_errors: Vec::new(),
            })
        })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcBlockTypesSource for FixtureIcrcSource {
    fn fetch_block_types<'a>(
        &'a self,
        _request: &'a IcrcLedgerRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<Vec<IcrcBlockTypeRow>, IcrcError>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcArchivesSource for FixtureIcrcSource {
    fn fetch_archives<'a>(
        &'a self,
        _request: &'a IcrcArchivesRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<Vec<IcrcArchiveRow>, IcrcError>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcTipCertificateSource for FixtureIcrcSource {
    fn fetch_tip_certificate<'a>(
        &'a self,
        _request: &'a IcrcLedgerRequest,
    ) -> ic_query::report::SourceFuture<
        'a,
        Result<Option<ic_platform::icrc_ledger_types::ICRC3DataCertificate>, IcrcError>,
    > {
        Box::pin(async move { Ok(None) })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcCapabilitiesSource for FixtureIcrcSource {
    fn fetch_capabilities<'a>(
        &'a self,
        _request: &'a IcrcLedgerRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<IcrcCapabilitiesData, IcrcError>> {
        Box::pin(async move {
            Ok(IcrcCapabilitiesData {
                supported_standards: vec![standard_row("ICRC-1")],
                capabilities: Vec::new(),
            })
        })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcAccountTransactionPageSource for FixtureIcrcSource {
    fn fetch_account_transaction_page<'a>(
        &'a self,
        _request: &'a IcrcAccountTransactionPageRequest,
    ) -> ic_query::report::SourceFuture<
        'a,
        Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError>,
    > {
        Box::pin(async move {
            Ok(IcrcAccountTransactionPageData {
                index_canister_id: ARCHIVE_CANISTER_ID.to_string(),
                balance: "0".to_string(),
                oldest_transaction_id: None,
                next_start: None,
                token_symbol: "FIX".to_string(),
                decimals: 8,
                transactions: Vec::new(),
            })
        })
    }
}

#[cfg(feature = "icrc-host")]
impl IcrcAccountTransactionCollectionSource for FixtureIcrcSource {
    fn fetch_complete_account_transactions<'a>(
        &'a self,
        _request: &'a IcrcAccountTransactionRefreshRequest,
        _progress: &'a mut (dyn QueryProgress + Send),
    ) -> ic_query::report::SourceFuture<
        'a,
        Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError>,
    > {
        Box::pin(async move {
            Ok(IcrcAccountTransactionCollectionData {
                index_canister_id: ARCHIVE_CANISTER_ID.to_string(),
                balance: "0".to_string(),
                token_symbol: "FIX".to_string(),
                decimals: 8,
                transactions: Vec::new(),
                page_count: 1,
                last_cursor: None,
            })
        })
    }
}

#[test]
fn public_icrc_token_api_is_constructible_and_renderable_without_host() {
    let request = IcrcLedgerRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcTokenReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
        token: ic_query::icrc::IcrcTokenData {
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
                value_type: IcrcMetadataValueKind::Text,
                value: json!("ICP"),
            }],
        },
    };

    let text = icrc_token_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize ICRC token report");

    assert!(text.contains(&format!("ledger_canister_id: {LEDGER_CANISTER_ID}")));
    assert!(text.contains("token_symbol: ICP"));
    assert!(text.contains("transfer_fee: 0.00"));
    assert!(text.contains("total_supply: 1.00"));
    assert_eq!(json["transfer_fee"], "10000");
    assert_eq!(json["total_supply"], "100000000");
    assert!(text.contains("ICRC-1"));
}

#[test]
fn public_icrc_balance_api_is_constructible_and_renderable_without_host() {
    let request = IcrcBalanceRequest {
        ledger: ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT.to_string(),
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID.to_string(),
        ),
        account_owner: ACCOUNT_OWNER.to_string(),
        subaccount_hex: None,
    };

    let report = IcrcBalanceReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.ledger.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
        account_owner: request.account_owner,
        subaccount_hex: request.subaccount_hex,
        value: IcrcBalanceData {
            token_symbol: "ICP".to_string(),
            decimals: 8,
            balance: "100000000".to_string(),
        },
    };

    let json = serde_json::to_value(&report).expect("serialize flat report");
    assert_eq!(
        json,
        serde_json::json!({
            "schema_version": 1,
            "ledger_canister_id": LEDGER_CANISTER_ID,
            "account_owner": ACCOUNT_OWNER,
            "fetched_at": FETCHED_AT,
            "source_endpoint": SOURCE_ENDPOINT,
            "fetched_by": FETCHED_BY,
            "token_symbol": "ICP",
            "decimals": 8,
            "subaccount_hex": null,
            "balance": "100000000",
        })
    );

    let text = icrc_balance_report_text(&report);

    assert!(text.contains("account_owner: aaaaa-aa"));
    assert!(text.contains("balance: 1.00 ICP"));
    assert!(text.contains("balance_base_units: 100000000"));
}

#[test]
fn public_icrc_allowance_api_is_constructible_and_renderable_without_host() {
    let request = IcrcAllowanceRequest {
        ledger: ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT.to_string(),
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID.to_string(),
        ),
        account_owner: ACCOUNT_OWNER.to_string(),
        account_subaccount_hex: None,
        spender_owner: ACCOUNT_OWNER.to_string(),
        spender_subaccount_hex: None,
    };

    let report = IcrcAllowanceReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.ledger.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
        account_owner: request.account_owner,
        account_subaccount_hex: request.account_subaccount_hex,
        spender_owner: request.spender_owner,
        spender_subaccount_hex: request.spender_subaccount_hex,
        value: IcrcAllowanceData {
            token_symbol: "ICP".to_string(),
            decimals: 8,
            allowance: "50000000".to_string(),
            expires_at_unix_nanos: Some("1700000000123456789".to_string()),
        },
    };

    let json = serde_json::to_value(&report).expect("serialize flat report");
    assert_eq!(
        json,
        serde_json::json!({
            "schema_version": 1,
            "ledger_canister_id": LEDGER_CANISTER_ID,
            "account_owner": ACCOUNT_OWNER,
            "fetched_at": FETCHED_AT,
            "source_endpoint": SOURCE_ENDPOINT,
            "fetched_by": FETCHED_BY,
            "token_symbol": "ICP",
            "decimals": 8,
            "account_subaccount_hex": null,
            "spender_owner": ACCOUNT_OWNER,
            "spender_subaccount_hex": null,
            "allowance": "50000000",
            "expires_at_unix_nanos": "1700000000123456789",
        })
    );

    let text = icrc_allowance_report_text(&report);

    assert!(text.contains("spender_owner: aaaaa-aa"));
    assert!(text.contains("allowance: 0.50 ICP"));
    assert!(text.contains("expires_at_unix_nanos: 1700000000123456789"));
}

#[test]
fn public_icrc_account_transaction_page_api_is_constructible_without_host() {
    let account = IcrcAccountRow::Structured {
        owner: ACCOUNT_OWNER.to_string(),
        subaccount_hex: Some(SUBACCOUNT_HEX.to_string()),
    };
    let report = IcrcAccountTransactionPageReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: SOURCE_ENDPOINT.to_string(),
            fetched_by: FETCHED_BY.to_string(),
        },
        account_owner: ACCOUNT_OWNER.to_string(),
        subaccount_hex: Some(SUBACCOUNT_HEX.to_string()),
        requested_start: Some("100".to_string()),
        requested_limit: 25,

        page: ic_query::icrc::IcrcAccountTransactionPageData {
            index_canister_id: ARCHIVE_CANISTER_ID.to_string(),
            next_start: Some("75".to_string()),
            oldest_transaction_id: Some("7".to_string()),
            balance: "100000000".to_string(),
            token_symbol: "ICP".to_string(),
            decimals: 8,
            transactions: vec![IcrcAccountTransactionRow {
                id: "75".to_string(),
                kind: "transfer".to_string(),
                timestamp_unix_nanos: Some("1700000000123456789".to_string()),
                amount_base_units: Some("50000000".to_string()),
                fee_base_units: Some("10000".to_string()),
                from: Some(account),
                to: Some(IcrcAccountRow::Structured {
                    owner: ARCHIVE_CANISTER_ID.to_string(),
                    subaccount_hex: None,
                }),
                spender: None,
                memo_hex: None,
                created_at_time_unix_nanos: None,
                expires_at_unix_nanos: None,
                expected_allowance_base_units: None,
                raw_transaction: json!({"kind": "transfer"}),
            }],
        },
    };

    let text = icrc_account_transaction_page_report_text(&report);

    assert!(text.contains(&format!("index_canister_id: {ARCHIVE_CANISTER_ID}")));
    assert!(text.contains("next_start: 75"));
    assert!(text.contains("balance: 1.00 ICP"));
    assert!(text.contains("transfer"));
}

#[test]
fn public_icrc_index_api_is_constructible_and_renderable_without_host() {
    let request = IcrcLedgerRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcIndexReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
        index: ic_query::icrc::IcrcIndexData::Unavailable("not configured".to_string()),
    };

    let text = icrc_index_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize index outcome");
    assert_eq!(json["index"], json!({"unavailable": "not configured"}));

    assert!(text.contains("index_canister_id: -"));
    assert!(text.contains("index_error: not configured"));
}

#[test]
fn public_icrc_transactions_api_is_constructible_and_renderable_without_host() {
    let request = IcrcTransactionsRequest {
        ledger: ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT.to_string(),
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID.to_string(),
        ),
        start: 100,
        limit: 2,
        follow_archives: true,
    };

    let report = IcrcTransactionsReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.ledger.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
        requested_start: request.start.to_string(),
        requested_limit: request.limit,
        follow_archives: request.follow_archives,
        transactions: ic_query::icrc::IcrcTransactionsData {
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
                block: ic_query::icrc::IcrcTransactionBlockRow {
                    index: "0".to_string(),
                    block_type: Some("1mint".to_string()),
                    transaction_kind: Some("1mint".to_string()),
                    timestamp_unix_nanos: Some("1699999999123456789".to_string()),
                    amount_base_units: Some("50000000".to_string()),
                    raw_block: json!({"Map": {"btype": {"Text": "1mint"}}}),
                },
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
        },
    };

    let text = icrc_transactions_report_text(&report);

    assert!(text.contains("requested_start: 100"));
    assert!(text.contains("follow_archives: true"));
    assert!(text.contains("archive_follow_errors: 1"));
    assert!(text.contains("archive query failed"));
}

#[test]
fn public_icrc_block_types_api_is_constructible_and_renderable_without_host() {
    let request = IcrcLedgerRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcBlockTypesReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
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
        ledger: ic_query::icrc::IcrcLedgerRequest::new(
            SOURCE_ENDPOINT.to_string(),
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID.to_string(),
        ),
        from_canister_id: Some(ARCHIVE_CANISTER_ID.to_string()),
    };

    let report = IcrcArchivesReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.ledger.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
        from_canister_id: request.from_canister_id,
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
    let request = IcrcLedgerRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let mut report = IcrcTipCertificateReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
        certificate: Some(ic_query::icrc::IcrcTipCertificateEvidence {
            certificate_hex: "010203".to_string(),
            certificate_bytes: 3,
            hash_tree_hex: "aabb".to_string(),
            hash_tree_bytes: 2,
        }),
    };

    let text = icrc_tip_certificate_report_text(&report);

    assert!(text.contains("certificate_present: true"));
    assert!(text.contains("certificate_bytes: 3 B"));
    assert!(text.contains("hash_tree_hex: aabb"));
    let json = serde_json::to_value(&report).expect("serialize certificate evidence");
    assert_eq!(
        json["certificate"],
        json!({
            "certificate_hex": "010203",
            "certificate_bytes": 3,
            "hash_tree_hex": "aabb",
            "hash_tree_bytes": 2,
        })
    );

    report.certificate = None;
    let json = serde_json::to_value(&report).expect("serialize absent certificate");
    assert_eq!(json["certificate"], serde_json::Value::Null);
    let text = icrc_tip_certificate_report_text(&report);
    assert!(text.contains("certificate_present: false"));
    assert!(text.contains("certificate_bytes: -"));
    assert!(text.contains("hash_tree_hex: -"));
}

#[test]
fn public_icrc_capabilities_api_is_constructible_and_renderable_without_host() {
    let request = IcrcLedgerRequest {
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        now_unix_secs: FETCHED_AT_UNIX_SECS,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
    };

    let report = IcrcCapabilitiesReport {
        provenance: ic_query::icrc::IcrcReportProvenance {
            schema_version: 1,
            ledger_canister_id: request.ledger_canister_id,
            fetched_at: FETCHED_AT.to_string(),
            source_endpoint: request.source_endpoint,
            fetched_by: FETCHED_BY.to_string(),
        },
        supported_standards: vec![standard_row("ICRC-1")],
        capabilities: vec![IcrcCapabilityRow {
            capability: "ICRC-3 tip certificate".to_string(),
            method: "icrc3_get_tip_certificate".to_string(),
            status: IcrcCapabilityStatus::Unsupported,
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
