use super::*;
use crate::ic::DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT;

#[test]
fn total_supply_url_preserves_ledger_and_explicit_bounds() {
    let query = IcIcrcTotalSupplyQuery::new(1_785_542_400, 1_785_801_600, 86_400);
    let url = total_supply_url(
        DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
        "mxzaz-hqaaa-aaaar-qaada-cai",
        &query,
    )
    .expect("total-supply URL");

    assert_eq!(
        url.path(),
        "/api/v2/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/total-supply"
    );
    assert_eq!(
        url.query(),
        Some("start=1785542400&end=1785801600&step=86400")
    );
}

#[test]
fn token_value_url_preserves_ledger_and_explicit_bounds() {
    let query = IcIcrcTokenValueQuery::new(1_785_542_400, 1_785_628_800, 100);
    let url = token_value_url(
        DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
        "mxzaz-hqaaa-aaaar-qaada-cai",
        &query,
    )
    .expect("token-value URL");

    assert_eq!(
        url.as_str(),
        "https://icrc-api.internetcomputer.org/api/v2/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/token-values?start=1785542400&end=1785628800&limit=100"
    );
}

#[test]
fn total_supply_wire_preserves_raw_values_and_ignores_additive_fields() {
    let wire: TotalSupplySeries = serde_json::from_value(serde_json::json!({
        "data": [[1_785_542_400_u64, "23326766272"]],
        "future_field": true
    }))
    .expect("current total-supply payload");

    assert_eq!(wire.data, [(1_785_542_400, "23326766272".to_string())]);
}

#[test]
fn token_value_wire_preserves_nullable_raw_fields_and_ignores_additions() {
    let wire: TokenValueSeries = serde_json::from_value(serde_json::json!({
        "ledger_canister_id": "mxzaz-hqaaa-aaaar-qaada-cai",
        "data": [{
            "price": "63710.86993032754",
            "volume_24h": null,
            "price_usd": "63710.86993032754",
            "volume_24h_usd": "23337.881075287027",
            "source": "ICPSwap-API",
            "source_url": "https://app.icpswap.com/info-tokens/details/mxzaz-hqaaa-aaaar-qaada-cai",
            "timestamp": 1_785_542_517_u64,
            "future_field": true
        }],
        "future_field": true
    }))
    .expect("current token-value payload");

    assert_eq!(wire.data.len(), 1);
    assert_eq!(wire.data[0].volume_24h, None);
    assert_eq!(wire.data[0].source.as_deref(), Some("ICPSwap-API"));
    assert_eq!(wire.data[0].timestamp_unix_secs, Some(1_785_542_517));
}

#[test]
fn indexed_count_urls_select_one_non_paginated_resource() {
    for (kind, resource) in [
        (IcIcrcIndexedCountKind::Account, "accounts"),
        (IcIcrcIndexedCountKind::Holder, "holders"),
        (IcIcrcIndexedCountKind::Transaction, "transactions"),
    ] {
        let url = indexed_count_url(
            DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
            "mxzaz-hqaaa-aaaar-qaada-cai",
            kind,
        )
        .expect("indexed-count URL");

        assert_eq!(
            url.as_str(),
            format!(
                "https://icrc-api.internetcomputer.org/api/v2/ledgers/mxzaz-hqaaa-aaaar-qaada-cai/{resource}/count"
            )
        );
    }
}

#[test]
fn indexed_count_wire_preserves_total_and_ignores_additive_fields() {
    let wire: IndexedCount = serde_json::from_value(serde_json::json!({
        "total": 78_272_u64,
        "future_field": true
    }))
    .expect("current indexed-count payload");

    assert_eq!(wire.total, 78_272);
}
