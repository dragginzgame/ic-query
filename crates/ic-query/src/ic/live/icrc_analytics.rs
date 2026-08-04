//! Module: ic::live::icrc_analytics
//!
//! Responsibility: live official ICRC analytics URL construction and wire decoding.
//! Does not own: native ledger calls, report projection, bounds policy, or rendering.
//! Boundary: fetches one selected ledger resource without pagination or follow-up calls.

use super::{LiveIcSource, append_path_segments, dashboard_base_url, fetch_live};
use crate::ic::{
    IcHostError, IcIcrcAnalyticsSource, IcIcrcIndexedCountKind, IcIcrcIndexedCountSourceData,
    IcIcrcTokenValueQuery, IcIcrcTokenValueSourceData, IcIcrcTokenValueSourceRow,
    IcIcrcTotalSupplyObservation, IcIcrcTotalSupplyQuery, IcIcrcTotalSupplySourceData,
    IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcIcrcAnalyticsSource for LiveIcSource {
    fn fetch_indexed_count(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        kind: IcIcrcIndexedCountKind,
    ) -> Result<IcIcrcIndexedCountSourceData, IcHostError> {
        let ledger_canister_id =
            source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
        let wire: IndexedCount = fetch_live(indexed_count_url(
            &request.endpoint,
            &ledger_canister_id,
            kind,
        )?)?;
        Ok(IcIcrcIndexedCountSourceData {
            source: request.clone(),
            ledger_canister_id,
            kind,
            total: wire.total,
        })
    }

    fn fetch_token_value_series(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcTokenValueQuery,
    ) -> Result<IcIcrcTokenValueSourceData, IcHostError> {
        source::validate_icrc_token_value_query(query)?;
        let ledger_canister_id =
            source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
        let wire: TokenValueSeries = fetch_live(token_value_url(
            &request.endpoint,
            &ledger_canister_id,
            query,
        )?)?;
        Ok(IcIcrcTokenValueSourceData {
            source: request.clone(),
            ledger_canister_id: wire.ledger_canister_id,
            query: query.clone(),
            rows: wire
                .data
                .into_iter()
                .map(|row| IcIcrcTokenValueSourceRow {
                    price: row.price,
                    volume_24h: row.volume_24h,
                    price_usd: row.price_usd,
                    volume_24h_usd: row.volume_24h_usd,
                    source: row.source,
                    source_url: row.source_url,
                    timestamp_unix_secs: row.timestamp,
                })
                .collect(),
        })
    }

    fn fetch_total_supply_series(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcTotalSupplyQuery,
    ) -> Result<IcIcrcTotalSupplySourceData, IcHostError> {
        source::validate_icrc_total_supply_query(query)?;
        let ledger_canister_id =
            source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
        let url = total_supply_url(&request.endpoint, &ledger_canister_id, query)?;
        let wire: TotalSupplySeries = fetch_live(url)?;
        Ok(IcIcrcTotalSupplySourceData {
            source: request.clone(),
            ledger_canister_id,
            query: query.clone(),
            observations: wire
                .data
                .into_iter()
                .map(|(timestamp_unix_secs, total_supply_base_units)| {
                    IcIcrcTotalSupplyObservation {
                        timestamp_unix_secs,
                        total_supply_base_units,
                    }
                })
                .collect(),
        })
    }
}

fn token_value_url(
    endpoint: &str,
    ledger_canister_id: &str,
    query: &IcIcrcTokenValueQuery,
) -> Result<Url, IcHostError> {
    let mut url = ledger_resource_url(endpoint, ledger_canister_id, &["token-values"])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("start", &query.start_unix_secs.to_string());
        pairs.append_pair("end", &query.end_unix_secs.to_string());
        pairs.append_pair("limit", &query.limit.to_string());
    }
    Ok(url)
}

fn total_supply_url(
    endpoint: &str,
    ledger_canister_id: &str,
    query: &IcIcrcTotalSupplyQuery,
) -> Result<Url, IcHostError> {
    let mut url = ledger_resource_url(endpoint, ledger_canister_id, &["total-supply"])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("start", &query.start_unix_secs.to_string());
        pairs.append_pair("end", &query.end_unix_secs.to_string());
        pairs.append_pair("step", &query.step_secs.to_string());
    }
    Ok(url)
}

fn indexed_count_url(
    endpoint: &str,
    ledger_canister_id: &str,
    kind: IcIcrcIndexedCountKind,
) -> Result<Url, IcHostError> {
    ledger_resource_url(
        endpoint,
        ledger_canister_id,
        &[kind.resource_path_segment(), "count"],
    )
}

fn ledger_resource_url(
    endpoint: &str,
    ledger_canister_id: &str,
    resource_path: &[&str],
) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["ledgers", ledger_canister_id])?;
    append_path_segments(endpoint, &mut url, resource_path)?;
    Ok(url)
}

#[derive(SerdeDeserialize)]
struct TotalSupplySeries {
    data: Vec<(u64, String)>,
}

#[derive(SerdeDeserialize)]
struct IndexedCount {
    total: u64,
}

#[derive(SerdeDeserialize)]
struct TokenValueSeries {
    ledger_canister_id: String,
    data: Vec<TokenValue>,
}

#[derive(SerdeDeserialize)]
struct TokenValue {
    price: Option<String>,
    volume_24h: Option<String>,
    price_usd: Option<String>,
    volume_24h_usd: Option<String>,
    source: Option<String>,
    source_url: Option<String>,
    timestamp: Option<u64>,
}

#[cfg(test)]
mod tests {
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
        assert_eq!(wire.data[0].timestamp, Some(1_785_542_517));
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
}
