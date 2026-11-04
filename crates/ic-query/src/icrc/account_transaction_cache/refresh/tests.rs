use super::*;
use crate::{
    icrc::{IcrcAccountTransactionCacheRequest, load_cached_icrc_account_transactions},
    tests::temp_dir,
};
use ic_host_text::timestamp::parse_utc_timestamp_secs;

#[test]
fn publication_uses_the_callers_clock_and_elapsed_collection_duration() {
    let root = temp_dir("ic-query-icrc-account-collection-duration");
    let cache = IcrcAccountTransactionCacheRequest::new(
        &root,
        "https://icp-api.io",
        "ryjl3-tyaaa-aaaaa-aaaba-cai",
        "aaaaa-aa",
    );
    let request = IcrcAccountTransactionRefreshRequest::new(cache, 2_000_000_000, 100);
    let paths = cache_paths(&request.cache);
    let complete = IcrcAccountTransactionCollectionData {
        index_canister_id: "qhbym-qaaaa-aaaaa-aaafq-cai".to_string(),
        balance: "0".to_string(),
        token_symbol: "ICP".to_string(),
        decimals: 8,
        transactions: Vec::new(),
        page_count: 1,
        last_cursor: None,
    };
    let report =
        publish_complete_snapshot(&request, &paths, false, complete, Duration::from_secs(23))
            .unwrap();
    let cached = load_cached_icrc_account_transactions(&request.cache).unwrap();
    assert_eq!(
        parse_utc_timestamp_secs(&report.context.collection_started_at),
        Some(2_000_000_000)
    );
    assert_eq!(
        parse_utc_timestamp_secs(&report.context.collection_completed_at),
        Some(2_000_000_023)
    );
    assert_eq!(
        cached.snapshot.context.collection_completed_at,
        report.context.collection_completed_at
    );
    std::fs::remove_dir_all(root).unwrap();
}
