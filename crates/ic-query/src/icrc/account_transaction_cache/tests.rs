use super::*;
use crate::{
    QueryProgress,
    icrc::{
        IcrcAccountTransactionCollectionSource, IcrcAccountTransactionRow,
        IcrcAccountTransactionSort,
    },
    test_support::temp_dir,
};
use serde_json::json;
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const INDEX_CANISTER_ID: &str = "qhbym-qaaaa-aaaaa-aaafq-cai";
const ACCOUNT_OWNER: &str = "aaaaa-aa";

#[test]
fn cache_entity_changes_with_collection_identity_not_view_options() {
    let first = IcrcAccountTransactionCacheRequest::new(
        "/tmp/project",
        "https://icp-api.io",
        "ryjl3-tyaaa-aaaaa-aaaba-cai",
        "aaaaa-aa",
    );
    let same = first.clone();
    let different_endpoint = IcrcAccountTransactionCacheRequest {
        source_endpoint: "https://example.com".to_string(),
        ..first.clone()
    };

    assert_eq!(cache_entity(&first), cache_entity(&same));
    assert_ne!(cache_entity(&first), cache_entity(&different_endpoint));
}

#[test]
fn complete_refresh_publishes_canonical_cache_and_cache_only_views() {
    let root = temp_dir("ic-query-icrc-account-refresh");
    let cache = cache_request(&root);
    let request = refresh_request(cache.clone(), 1_700_000_000);
    let source = SuccessSource::new(vec![row("12"), row("10"), row("2")]);

    let refresh = refresh_icrc_account_transaction_cache_with_source(&request, &source)
        .expect("refresh complete cache");
    let cached = load_cached_icrc_account_transactions(&cache).expect("load complete cache");
    let oldest = build_icrc_account_transaction_list_report(
        &IcrcAccountTransactionListRequest::new(cache.clone(), 2)
            .with_sort(IcrcAccountTransactionSort::Oldest),
    )
    .expect("list cache oldest first");
    let status =
        build_icrc_account_transaction_cache_status_report(&cache).expect("cache status report");

    assert!(!refresh.point_in_time_guaranteed);
    assert_eq!(refresh.transaction_count, 3);
    assert_eq!(cached.snapshot.completeness.status, "api_exhausted");
    assert_eq!(cached.snapshot.completeness.row_count, 3);
    assert_eq!(
        oldest
            .transactions
            .iter()
            .map(|transaction| transaction.id.as_str())
            .collect::<Vec<_>>(),
        vec!["2", "10"]
    );
    assert_eq!(oldest.total_transaction_count, 3);
    assert_eq!(
        status.cache.as_ref().expect("cache summary").cache_status,
        "ok"
    );
    assert_eq!(
        status
            .latest_attempt
            .as_ref()
            .expect("refresh attempt")
            .status,
        "complete"
    );
    assert_eq!(source.calls.load(Ordering::Relaxed), 1);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_refresh_preserves_last_complete_cache_and_records_failure() {
    let root = temp_dir("ic-query-icrc-account-refresh-failure");
    let cache = cache_request(&root);
    let first_request = refresh_request(cache.clone(), 1_700_000_000);
    refresh_icrc_account_transaction_cache_with_source(
        &first_request,
        &SuccessSource::new(vec![row("7")]),
    )
    .expect("seed complete cache");
    let path = icrc_account_transaction_cache_path(&cache).expect("cache path");
    let before = fs::read(&path).expect("read seeded cache");
    let failed_request = refresh_request(cache.clone(), 1_700_000_001);

    let error =
        refresh_icrc_account_transaction_cache_with_source(&failed_request, &IncompleteSource)
            .expect_err("incomplete refresh must fail");
    let after = fs::read(&path).expect("read preserved cache");
    let status =
        build_icrc_account_transaction_cache_status_report(&cache).expect("cache status report");

    assert!(matches!(
        error,
        IcrcAccountTransactionError::IncompleteCollection {
            pages_fetched: 2,
            rows_fetched: 100,
            ..
        }
    ));
    assert_eq!(after, before);
    assert_eq!(
        status.cache.as_ref().expect("preserved cache").cache_status,
        "ok"
    );
    let attempt = status.latest_attempt.expect("failed attempt");
    assert_eq!(attempt.status, "failed");
    assert_eq!(attempt.pages_fetched, 2);
    assert_eq!(attempt.rows_fetched, 100);
    assert_eq!(attempt.last_cursor.as_deref(), Some("50"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn source_claiming_completion_with_wrong_final_cursor_is_not_published() {
    let root = temp_dir("ic-query-icrc-account-invalid-completion");
    let cache = cache_request(&root);
    let request = refresh_request(cache.clone(), 1_700_000_000);

    let error = refresh_icrc_account_transaction_cache_with_source(&request, &WrongCursorSource)
        .expect_err("invalid completion evidence must fail");

    assert!(matches!(
        error,
        IcrcAccountTransactionError::IncompleteCollection {
            reason,
            ..
        } if reason.contains("final cursor")
    ));
    assert!(
        !icrc_account_transaction_cache_path(&cache)
            .expect("cache path")
            .exists()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_and_stale_refresh_policies_do_not_masquerade_as_each_other() {
    let root = temp_dir("ic-query-icrc-account-refresh-policy");
    let cache = cache_request(&root);
    let source = SuccessSource::new(vec![row("1")]);
    let request = refresh_request(cache.clone(), 1_700_000_000);

    load_or_refresh_missing_icrc_account_transactions_with_source(&request, &source)
        .expect("refresh missing cache");
    load_or_refresh_missing_icrc_account_transactions_with_source(&request, &source)
        .expect("reuse present cache");
    assert_eq!(source.calls.load(Ordering::Relaxed), 1);

    let stale_request = refresh_request(cache, 2_000_000_000);
    load_or_refresh_stale_icrc_account_transactions_with_source(&stale_request, 60, &source)
        .expect("refresh stale cache");
    assert_eq!(source.calls.load(Ordering::Relaxed), 2);

    let _ = fs::remove_dir_all(root);
}

fn cache_request(root: &Path) -> IcrcAccountTransactionCacheRequest {
    IcrcAccountTransactionCacheRequest::new(
        root,
        "https://icp-api.io",
        LEDGER_CANISTER_ID,
        ACCOUNT_OWNER,
    )
}

fn refresh_request(
    cache: IcrcAccountTransactionCacheRequest,
    now_unix_secs: u64,
) -> IcrcAccountTransactionRefreshRequest {
    IcrcAccountTransactionRefreshRequest::new(cache, now_unix_secs, 100, 1_800)
}

fn row(id: &str) -> IcrcAccountTransactionRow {
    IcrcAccountTransactionRow {
        id: id.to_string(),
        kind: "transfer".to_string(),
        timestamp_unix_nanos: None,
        amount_base_units: None,
        fee_base_units: None,
        from: None,
        to: None,
        spender: None,
        memo_hex: None,
        created_at_time_unix_nanos: None,
        expires_at_unix_nanos: None,
        expected_allowance_base_units: None,
        raw_transaction: json!({"kind": "transfer"}),
    }
}

struct SuccessSource {
    calls: AtomicUsize,
    transactions: Vec<IcrcAccountTransactionRow>,
}

impl SuccessSource {
    fn new(transactions: Vec<IcrcAccountTransactionRow>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            transactions,
        }
    }
}

impl IcrcAccountTransactionCollectionSource for SuccessSource {
    fn fetch_complete_account_transactions(
        &self,
        _request: &IcrcAccountTransactionRefreshRequest,
        _progress: &mut (dyn QueryProgress + Send),
    ) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(IcrcAccountTransactionCollectionData {
            index_canister_id: INDEX_CANISTER_ID.to_string(),
            balance: "42".to_string(),
            token_symbol: "ICP".to_string(),
            decimals: 8,
            transactions: self.transactions.clone(),
            page_count: 1,
            last_cursor: self
                .transactions
                .last()
                .map(|transaction| transaction.id.clone()),
        })
    }
}

struct IncompleteSource;

impl IcrcAccountTransactionCollectionSource for IncompleteSource {
    fn fetch_complete_account_transactions(
        &self,
        _request: &IcrcAccountTransactionRefreshRequest,
        _progress: &mut (dyn QueryProgress + Send),
    ) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError> {
        Err(IcrcAccountTransactionError::IncompleteCollection {
            pages_fetched: 2,
            rows_fetched: 100,
            last_cursor: Some("50".to_string()),
            reason: "fixture stopped before API exhaustion".to_string(),
        })
    }
}

struct WrongCursorSource;

impl IcrcAccountTransactionCollectionSource for WrongCursorSource {
    fn fetch_complete_account_transactions(
        &self,
        _request: &IcrcAccountTransactionRefreshRequest,
        _progress: &mut (dyn QueryProgress + Send),
    ) -> Result<IcrcAccountTransactionCollectionData, IcrcAccountTransactionError> {
        Ok(IcrcAccountTransactionCollectionData {
            index_canister_id: INDEX_CANISTER_ID.to_string(),
            balance: "42".to_string(),
            token_symbol: "ICP".to_string(),
            decimals: 8,
            transactions: vec![row("7")],
            page_count: 1,
            last_cursor: Some("6".to_string()),
        })
    }
}
