use super::*;
use crate::ic::{IcIcrcAccountSort, IcIcrcHolderSort};

#[test]
fn page_queries_reject_conflicting_cursors_and_invalid_limits() {
    let mut account = IcIcrcAccountListQuery::new(20, IcIcrcAccountSort::Id);
    account.after = Some("next".into());
    account.before = Some("previous".into());

    assert!(normalized_account_list_query(&account).is_err());

    let holder = IcIcrcHolderListQuery::new(0, IcIcrcHolderSort::Principal);
    assert!(validate_holder_list_query(&holder).is_err());
}

#[test]
fn nanosecond_timestamps_normalize_without_losing_subsecond_precision() {
    assert_eq!(
        split_unix_nanos(1_731_833_720_056_622_711),
        (1_731_833_720, 56_622_711)
    );
}

#[test]
fn holder_projection_rejects_structured_or_boolean_value_evidence() {
    let row = IcIcrcHolderSourceRow {
        principal: "aaaaa-aa".to_string(),
        balance_base_units: "1".to_string(),
        total_transactions: 1,
        created_at_unix_nanos: 1_000_000_000,
        ledger_canister_id: "mxzaz-hqaaa-aaaar-qaada-cai".to_string(),
        latest_transaction_index: 1,
        percentage: serde_json::json!(0.1),
        value_usd: serde_json::json!(true),
        dashboard_updated_at: "2026-08-09T10:00:00Z".to_string(),
    };

    assert!(project_holder_row(0, "mxzaz-hqaaa-aaaar-qaada-cai", row).is_err());
}
