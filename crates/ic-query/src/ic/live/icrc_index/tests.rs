use super::*;
use crate::ic::{
    DEFAULT_ICRC_ACCOUNT_INFO_SOURCE_ENDPOINT, DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
    IcIcrcAccountSort, IcIcrcHolderSort,
};

const LEDGER: &str = "mxzaz-hqaaa-aaaar-qaada-cai";

#[test]
fn index_wire_records_preserve_raw_fields_and_optional_fee_defaults() {
    let value = serde_json::json!({
        "id": "opaque-account", "owner": "aaaaa-aa", "subaccount": "00",
        "balance": "18446744073709551616", "total_transactions": 7,
        "created_timestamp": u64::MAX, "ledger_canister_id": LEDGER,
        "latest_transaction_index": 8, "updated_at": "source-time"
    });
    let account: IcIcrcAccountSourceRow = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(account.account_id, "opaque-account");
    assert_eq!(account.balance_base_units, "18446744073709551616");
    assert_eq!(account.created_at_unix_nanos, u64::MAX);
    assert_eq!(account.dashboard_updated_at, "source-time");
    assert!(!account.active_fee_collector);
    assert_eq!(
        account.fee_collector_block_ranges,
        Vec::<Vec<serde_json::Value>>::new()
    );
    let mut missing = value;
    missing.as_object_mut().unwrap().remove("owner");
    assert!(serde_json::from_value::<IcIcrcAccountSourceRow>(missing).is_err());
    let holder: IcIcrcHolderSourceRow = serde_json::from_value(serde_json::json!({
        "principal": "aaaaa-aa", "balance": "123", "total_transactions": 7,
        "created_timestamp": u64::MAX, "ledger_canister_id": LEDGER,
        "latest_transaction_index": 8, "percentage": "12.5", "value_usd": null,
        "updated_at": "source-time"
    }))
    .unwrap();
    assert_eq!(holder.balance_base_units, "123");
    assert_eq!(holder.created_at_unix_nanos, u64::MAX);
    assert_eq!(holder.percentage, serde_json::json!("12.5"));
    assert!(holder.value_usd.is_null());
}

#[test]
fn account_list_url_preserves_owner_cursor_limit_and_sort() {
    let mut query = IcIcrcAccountListQuery::new(2, IcIcrcAccountSort::BalanceDescending);
    query.owner = Some("222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae".into());
    query.after = Some("1668734888.0,hkmli-faaaa-aaaar-qb4ba-cai".into());

    let url = account_list_url(DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT, LEDGER, &query)
        .expect("account-list URL");

    assert_eq!(
        url.path(),
        "/api/v2/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/accounts"
    );
    assert_eq!(
        url.query(),
        Some(
            "owner=222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae&after=1668734888.0&after=hkmli-faaaa-aaaar-qb4ba-cai&limit=2&sort_by=-balance"
        )
    );
}

#[test]
fn holder_list_and_exact_account_use_their_declared_api_versions() {
    let holder = holder_list_url(
        DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
        LEDGER,
        &IcIcrcHolderListQuery::new(20, IcIcrcHolderSort::Principal),
    )
    .expect("holder-list URL");
    let account = account_info_url(
        DEFAULT_ICRC_ACCOUNT_INFO_SOURCE_ENDPOINT,
        LEDGER,
        "222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae",
    )
    .expect("account-info URL");

    assert_eq!(
        holder.path(),
        "/api/v2/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/holders"
    );
    assert_eq!(
        account.path(),
        "/api/v1/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/accounts/222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae"
    );
}
