#[cfg(feature = "host")]
use ic_query::QueryProgress;
#[cfg(feature = "host")]
use ic_query::icrc::{
    CachedIcrcAccountTransactionSnapshot,
    DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS,
    ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE, IcrcAccountTransactionCacheStatusReport,
    IcrcAccountTransactionCollectionData, IcrcAccountTransactionCollectionSource,
    IcrcAccountTransactionError, IcrcAccountTransactionListReport, IcrcAccountTransactionPageData,
    IcrcAccountTransactionPageSource, IcrcAccountTransactionRefreshReport, IcrcAllowanceData,
    IcrcAllowanceSource, IcrcArchivesData, IcrcArchivesSource, IcrcBalanceData, IcrcBalanceSource,
    IcrcBlockTypesData, IcrcBlockTypesSource, IcrcCapabilitiesData, IcrcCapabilitiesSource,
    IcrcError, IcrcIndexData, IcrcIndexSource, IcrcTipCertificateData, IcrcTipCertificateSource,
    IcrcTokenData, IcrcTokenSource, IcrcTransactionsData, IcrcTransactionsSource,
    build_icrc_account_transaction_cache_status_report, build_icrc_account_transaction_list_report,
    build_icrc_account_transaction_page_report,
    build_icrc_account_transaction_page_report_with_source, build_icrc_allowance_report,
    build_icrc_allowance_report_with_source, build_icrc_archives_report,
    build_icrc_archives_report_with_source, build_icrc_balance_report,
    build_icrc_balance_report_with_source, build_icrc_block_types_report,
    build_icrc_block_types_report_with_source, build_icrc_capabilities_report,
    build_icrc_capabilities_report_with_source, build_icrc_index_report,
    build_icrc_index_report_with_source, build_icrc_tip_certificate_report,
    build_icrc_tip_certificate_report_with_source, build_icrc_token_report,
    build_icrc_token_report_with_source, build_icrc_transactions_report,
    build_icrc_transactions_report_with_source, icrc_account_transaction_cache_path,
    icrc_account_transaction_refresh_attempt_path, icrc_account_transaction_refresh_lock_path,
    load_cached_icrc_account_transactions, load_or_refresh_missing_icrc_account_transactions,
    load_or_refresh_missing_icrc_account_transactions_with_source,
    load_or_refresh_stale_icrc_account_transactions,
    load_or_refresh_stale_icrc_account_transactions_with_source,
    refresh_icrc_account_transaction_cache, refresh_icrc_account_transaction_cache_with_progress,
    refresh_icrc_account_transaction_cache_with_source,
};
use ic_query::icrc::{
    DEFAULT_ICRC_SOURCE_ENDPOINT, IcrcAccountRow, IcrcAccountTransactionCacheRequest,
    IcrcAccountTransactionListRequest, IcrcAccountTransactionPageReport,
    IcrcAccountTransactionPageRequest, IcrcAccountTransactionRefreshRequest,
    IcrcAccountTransactionRow, IcrcAccountTransactionSort, IcrcAllowanceReport,
    IcrcAllowanceRequest, IcrcArchiveFollowErrorRow, IcrcArchiveRow, IcrcArchivedBlocksRow,
    IcrcArchivedRangeRow, IcrcArchivesReport, IcrcArchivesRequest, IcrcBalanceReport,
    IcrcBalanceRequest, IcrcBlockTypeRow, IcrcBlockTypesReport, IcrcCapabilitiesReport,
    IcrcCapabilityRow, IcrcCapabilityStatus, IcrcFollowedArchiveBlockRow, IcrcIndexReport,
    IcrcLedgerRequest, IcrcMetadataValueKind, IcrcTipCertificateReport, IcrcTokenMetadataRow,
    IcrcTokenReport, IcrcTokenStandardRow, IcrcTransactionBlockRow, IcrcTransactionsReport,
    IcrcTransactionsRequest, icrc_account_transaction_page_report_text, icrc_allowance_report_text,
    icrc_archives_report_text, icrc_balance_report_text, icrc_block_types_report_text,
    icrc_capabilities_report_text, icrc_index_report_text, icrc_tip_certificate_report_text,
    icrc_token_report_text, icrc_transactions_report_text, normalize_subaccount_hex,
};
use serde_json::json;
#[cfg(feature = "host")]
use std::path::PathBuf;

const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const ACCOUNT_OWNER: &str = "aaaaa-aa";
const ARCHIVE_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";
const SOURCE_ENDPOINT: &str = "https://icp-api.io";
const FETCHED_AT: &str = "2023-11-14T22:13:20Z";
const FETCHED_AT_UNIX_SECS: u64 = 1_700_000_000;
const FETCHED_BY: &str = "ic-query";
const SUBACCOUNT_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";

#[cfg(feature = "host")]
type IcrcTokenBuilder = fn(&IcrcLedgerRequest) -> Result<IcrcTokenReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcBalanceBuilder = fn(&IcrcBalanceRequest) -> Result<IcrcBalanceReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcAllowanceBuilder = fn(&IcrcAllowanceRequest) -> Result<IcrcAllowanceReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionPageBuilder =
    fn(
        &IcrcAccountTransactionPageRequest,
    ) -> Result<IcrcAccountTransactionPageReport, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionListBuilder =
    fn(
        &IcrcAccountTransactionListRequest,
    ) -> Result<IcrcAccountTransactionListReport, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionCacheLoader =
    fn(
        &IcrcAccountTransactionCacheRequest,
    ) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionRefreshWithSource =
    fn(
        &IcrcAccountTransactionRefreshRequest,
        &dyn IcrcAccountTransactionCollectionSource,
    ) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionLoadMissingWithSource =
    fn(
        &IcrcAccountTransactionRefreshRequest,
        &dyn IcrcAccountTransactionCollectionSource,
    ) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionLoadStaleWithSource =
    fn(
        &IcrcAccountTransactionRefreshRequest,
        u64,
        &dyn IcrcAccountTransactionCollectionSource,
    ) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionCachePath =
    fn(&IcrcAccountTransactionCacheRequest) -> Result<PathBuf, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionCacheStatusBuilder =
    fn(
        &IcrcAccountTransactionCacheRequest,
    ) -> Result<IcrcAccountTransactionCacheStatusReport, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionRefresh =
    fn(
        &IcrcAccountTransactionRefreshRequest,
    ) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionRefreshWithProgress =
    fn(
        &IcrcAccountTransactionRefreshRequest,
        &mut (dyn QueryProgress + Send),
    ) -> Result<IcrcAccountTransactionRefreshReport, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionLoadMissing =
    fn(
        &IcrcAccountTransactionRefreshRequest,
    ) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcAccountTransactionLoadStale =
    fn(
        &IcrcAccountTransactionRefreshRequest,
        u64,
    ) -> Result<CachedIcrcAccountTransactionSnapshot, IcrcAccountTransactionError>;
#[cfg(feature = "host")]
type IcrcIndexBuilder = fn(&IcrcLedgerRequest) -> Result<IcrcIndexReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcTransactionsBuilder =
    fn(&IcrcTransactionsRequest) -> Result<IcrcTransactionsReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcBlockTypesBuilder = fn(&IcrcLedgerRequest) -> Result<IcrcBlockTypesReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcArchivesBuilder = fn(&IcrcArchivesRequest) -> Result<IcrcArchivesReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcTipCertificateBuilder =
    fn(&IcrcLedgerRequest) -> Result<IcrcTipCertificateReport, IcrcError>;
#[cfg(feature = "host")]
type IcrcCapabilitiesBuilder = fn(&IcrcLedgerRequest) -> Result<IcrcCapabilitiesReport, IcrcError>;

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

    let account_transactions = IcrcAccountTransactionPageRequest::new(
        SOURCE_ENDPOINT,
        FETCHED_AT_UNIX_SECS,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
        25,
    )
    .with_index_canister_id(ARCHIVE_CANISTER_ID)
    .with_subaccount_hex(SUBACCOUNT_HEX)
    .with_start("100");
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

    let cache = IcrcAccountTransactionCacheRequest::new(
        "/tmp/ic-query-cache",
        SOURCE_ENDPOINT,
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
    )
    .with_subaccount_hex(SUBACCOUNT_HEX);
    let refresh =
        IcrcAccountTransactionRefreshRequest::new(cache.clone(), FETCHED_AT_UNIX_SECS, 100, 1_800)
            .with_index_canister_id(ARCHIVE_CANISTER_ID)
            .with_max_pages(Some(50));
    let list = IcrcAccountTransactionListRequest::new(cache, 25)
        .with_sort(IcrcAccountTransactionSort::Oldest);
    assert_eq!(refresh.page_size, 100);
    assert_eq!(refresh.max_pages, Some(50));
    assert_eq!(list.sort, IcrcAccountTransactionSort::Oldest);

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

    let archives =
        IcrcArchivesRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID)
            .with_from_canister_id(ARCHIVE_CANISTER_ID);
    assert_eq!(
        archives.from_canister_id.as_deref(),
        Some(ARCHIVE_CANISTER_ID)
    );
}

#[cfg(feature = "host")]
#[test]
fn public_icrc_host_api_exposes_live_builder_entry_points() {
    assert_eq!(ICRC_ACCOUNT_TRANSACTION_MAX_PAGE_SIZE, 100);
    assert_eq!(
        DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS,
        1_800
    );
    accepts_public_function::<IcrcTokenBuilder>(build_icrc_token_report);
    accepts_public_function::<IcrcBalanceBuilder>(build_icrc_balance_report);
    accepts_public_function::<IcrcAllowanceBuilder>(build_icrc_allowance_report);
    accepts_public_function::<IcrcAccountTransactionPageBuilder>(
        build_icrc_account_transaction_page_report,
    );
    accepts_public_function::<IcrcAccountTransactionListBuilder>(
        build_icrc_account_transaction_list_report,
    );
    accepts_public_function::<IcrcAccountTransactionCacheLoader>(
        load_cached_icrc_account_transactions,
    );
    accepts_public_function::<IcrcAccountTransactionRefreshWithSource>(
        refresh_icrc_account_transaction_cache_with_source,
    );
    accepts_public_function::<IcrcAccountTransactionLoadMissingWithSource>(
        load_or_refresh_missing_icrc_account_transactions_with_source,
    );
    accepts_public_function::<IcrcAccountTransactionLoadStaleWithSource>(
        load_or_refresh_stale_icrc_account_transactions_with_source,
    );
    for path_builder in [
        icrc_account_transaction_cache_path,
        icrc_account_transaction_refresh_attempt_path,
        icrc_account_transaction_refresh_lock_path,
    ] {
        accepts_public_function::<IcrcAccountTransactionCachePath>(path_builder);
    }
    accepts_public_function::<IcrcAccountTransactionCacheStatusBuilder>(
        build_icrc_account_transaction_cache_status_report,
    );
    accepts_public_function::<IcrcAccountTransactionRefresh>(
        refresh_icrc_account_transaction_cache,
    );
    accepts_public_function::<IcrcAccountTransactionRefreshWithProgress>(
        refresh_icrc_account_transaction_cache_with_progress,
    );
    accepts_public_function::<IcrcAccountTransactionLoadMissing>(
        load_or_refresh_missing_icrc_account_transactions,
    );
    accepts_public_function::<IcrcAccountTransactionLoadStale>(
        load_or_refresh_stale_icrc_account_transactions,
    );
    accepts_public_function::<IcrcIndexBuilder>(build_icrc_index_report);
    accepts_public_function::<IcrcTransactionsBuilder>(build_icrc_transactions_report);
    accepts_public_function::<IcrcBlockTypesBuilder>(build_icrc_block_types_report);
    accepts_public_function::<IcrcArchivesBuilder>(build_icrc_archives_report);
    accepts_public_function::<IcrcTipCertificateBuilder>(build_icrc_tip_certificate_report);
    accepts_public_function::<IcrcCapabilitiesBuilder>(build_icrc_capabilities_report);
}

#[cfg(feature = "host")]
fn accepts_public_function<T>(_function: T) {}

#[cfg(feature = "host")]
#[test]
fn public_icrc_host_api_accepts_custom_source_adapters() {
    let source = FixtureIcrcSource;
    let token_request =
        IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID);
    let report =
        build_icrc_token_report_with_source(&token_request, &source).expect("token report");

    assert_eq!(report.ledger_canister_id, LEDGER_CANISTER_ID);
    assert_eq!(report.token_symbol, "FIX");
    assert_eq!(report.supported_standards, vec![standard_row("ICRC-1")]);

    build_icrc_balance_report_with_source(
        &IcrcBalanceRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
        ),
        &source,
    )
    .expect("balance report");
    build_icrc_allowance_report_with_source(
        &IcrcAllowanceRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            ACCOUNT_OWNER,
        ),
        &source,
    )
    .expect("allowance report");
    build_icrc_account_transaction_page_report_with_source(
        &IcrcAccountTransactionPageRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
            ACCOUNT_OWNER,
            1,
        ),
        &source,
    )
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
                1_800,
            ),
            &mut progress,
        )
        .expect("complete account transaction source");
    build_icrc_index_report_with_source(
        &IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .expect("index report");
    build_icrc_transactions_report_with_source(
        &IcrcTransactionsRequest::new(
            SOURCE_ENDPOINT,
            FETCHED_AT_UNIX_SECS,
            LEDGER_CANISTER_ID,
            0,
            1,
        ),
        &source,
    )
    .expect("transactions report");
    build_icrc_block_types_report_with_source(
        &IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .expect("block types report");
    build_icrc_archives_report_with_source(
        &IcrcArchivesRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .expect("archives report");
    build_icrc_tip_certificate_report_with_source(
        &IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .expect("tip certificate report");
    build_icrc_capabilities_report_with_source(
        &IcrcLedgerRequest::new(SOURCE_ENDPOINT, FETCHED_AT_UNIX_SECS, LEDGER_CANISTER_ID),
        &source,
    )
    .expect("capabilities report");
}

#[cfg(feature = "host")]
struct FixtureIcrcSource;

#[cfg(feature = "host")]
impl IcrcTokenSource for FixtureIcrcSource {
    fn fetch_token(&self, request: &IcrcLedgerRequest) -> Result<IcrcTokenData, IcrcError> {
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
    }
}

#[cfg(feature = "host")]
impl IcrcBalanceSource for FixtureIcrcSource {
    fn fetch_balance(&self, _request: &IcrcBalanceRequest) -> Result<IcrcBalanceData, IcrcError> {
        Ok(IcrcBalanceData {
            token_symbol: "FIX".to_string(),
            decimals: 8,
            balance: "0".to_string(),
        })
    }
}

#[cfg(feature = "host")]
impl IcrcAllowanceSource for FixtureIcrcSource {
    fn fetch_allowance(
        &self,
        _request: &IcrcAllowanceRequest,
    ) -> Result<IcrcAllowanceData, IcrcError> {
        Ok(IcrcAllowanceData {
            token_symbol: "FIX".to_string(),
            decimals: 8,
            allowance: "0".to_string(),
            expires_at_unix_nanos: None,
        })
    }
}

#[cfg(feature = "host")]
impl IcrcIndexSource for FixtureIcrcSource {
    fn fetch_index(&self, _request: &IcrcLedgerRequest) -> Result<IcrcIndexData, IcrcError> {
        Ok(IcrcIndexData {
            index_canister_id: None,
            index_error: None,
        })
    }
}

#[cfg(feature = "host")]
impl IcrcTransactionsSource for FixtureIcrcSource {
    fn fetch_transactions(
        &self,
        _request: &IcrcTransactionsRequest,
    ) -> Result<IcrcTransactionsData, IcrcError> {
        Ok(IcrcTransactionsData {
            log_length: Some("0".to_string()),
            blocks: Vec::new(),
            archived_blocks: Vec::new(),
            followed_archive_blocks: Vec::new(),
            archive_follow_errors: Vec::new(),
        })
    }
}

#[cfg(feature = "host")]
impl IcrcBlockTypesSource for FixtureIcrcSource {
    fn fetch_block_types(
        &self,
        _request: &IcrcLedgerRequest,
    ) -> Result<IcrcBlockTypesData, IcrcError> {
        Ok(IcrcBlockTypesData {
            block_types: Vec::new(),
        })
    }
}

#[cfg(feature = "host")]
impl IcrcArchivesSource for FixtureIcrcSource {
    fn fetch_archives(
        &self,
        _request: &IcrcArchivesRequest,
    ) -> Result<IcrcArchivesData, IcrcError> {
        Ok(IcrcArchivesData {
            archives: Vec::new(),
        })
    }
}

#[cfg(feature = "host")]
impl IcrcTipCertificateSource for FixtureIcrcSource {
    fn fetch_tip_certificate(
        &self,
        _request: &IcrcLedgerRequest,
    ) -> Result<IcrcTipCertificateData, IcrcError> {
        Ok(IcrcTipCertificateData {
            certificate_hex: None,
            certificate_bytes: None,
            hash_tree_hex: None,
            hash_tree_bytes: None,
        })
    }
}

#[cfg(feature = "host")]
impl IcrcCapabilitiesSource for FixtureIcrcSource {
    fn fetch_capabilities(
        &self,
        _request: &IcrcLedgerRequest,
    ) -> Result<IcrcCapabilitiesData, IcrcError> {
        Ok(IcrcCapabilitiesData {
            supported_standards: vec![standard_row("ICRC-1")],
            capabilities: Vec::new(),
        })
    }
}

#[cfg(feature = "host")]
impl IcrcAccountTransactionPageSource for FixtureIcrcSource {
    fn fetch_account_transaction_page(
        &self,
        _request: &IcrcAccountTransactionPageRequest,
    ) -> Result<IcrcAccountTransactionPageData, IcrcAccountTransactionError> {
        Ok(IcrcAccountTransactionPageData {
            index_canister_id: ARCHIVE_CANISTER_ID.to_string(),
            balance: "0".to_string(),
            oldest_transaction_id: None,
            next_start: None,
            token_symbol: "FIX".to_string(),
            decimals: 8,
            transactions: Vec::new(),
        })
    }
}

#[cfg(feature = "host")]
impl IcrcAccountTransactionCollectionSource for FixtureIcrcSource {
    fn fetch_complete_account_transactions(
        &self,
        _request: &IcrcAccountTransactionRefreshRequest,
        _progress: &mut (dyn QueryProgress + Send),
    ) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError> {
        Ok(IcrcAccountTransactionCollectionData {
            index_canister_id: ARCHIVE_CANISTER_ID.to_string(),
            balance: "0".to_string(),
            token_symbol: "FIX".to_string(),
            decimals: 8,
            transactions: Vec::new(),
            page_count: 1,
            last_cursor: None,
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
            value_type: IcrcMetadataValueKind::Text,
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
fn public_icrc_account_transaction_page_api_is_constructible_without_host() {
    let account = IcrcAccountRow {
        owner: Some(ACCOUNT_OWNER.to_string()),
        subaccount_hex: Some(SUBACCOUNT_HEX.to_string()),
        account_identifier: None,
    };
    let report = IcrcAccountTransactionPageReport {
        schema_version: 1,
        ledger_canister_id: LEDGER_CANISTER_ID.to_string(),
        index_canister_id: ARCHIVE_CANISTER_ID.to_string(),
        account_owner: ACCOUNT_OWNER.to_string(),
        subaccount_hex: Some(SUBACCOUNT_HEX.to_string()),
        requested_start: Some("100".to_string()),
        requested_limit: 25,
        next_start: Some("75".to_string()),
        oldest_transaction_id: Some("7".to_string()),
        balance: "100000000".to_string(),
        token_symbol: "ICP".to_string(),
        decimals: 8,
        fetched_at: FETCHED_AT.to_string(),
        source_endpoint: SOURCE_ENDPOINT.to_string(),
        fetched_by: FETCHED_BY.to_string(),
        transactions: vec![IcrcAccountTransactionRow {
            id: "75".to_string(),
            kind: "transfer".to_string(),
            timestamp_unix_nanos: Some("1700000000123456789".to_string()),
            amount_base_units: Some("50000000".to_string()),
            fee_base_units: Some("10000".to_string()),
            from: Some(account),
            to: Some(IcrcAccountRow {
                owner: Some(ARCHIVE_CANISTER_ID.to_string()),
                subaccount_hex: None,
                account_identifier: None,
            }),
            spender: None,
            memo_hex: None,
            created_at_time_unix_nanos: None,
            expires_at_unix_nanos: None,
            expected_allowance_base_units: None,
            raw_transaction: json!({"kind": "transfer"}),
        }],
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
    let request = IcrcLedgerRequest {
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
    let request = IcrcLedgerRequest {
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
    assert!(text.contains("certificate_bytes: 3 B"));
    assert!(text.contains("hash_tree_hex: aabb"));
}

#[test]
fn public_icrc_capabilities_api_is_constructible_and_renderable_without_host() {
    let request = IcrcLedgerRequest {
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
