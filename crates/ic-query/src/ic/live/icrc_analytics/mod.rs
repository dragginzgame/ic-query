//! Module: ic::live::icrc_analytics
//!
//! Responsibility: live official ICRC analytics URL construction and wire decoding.
//! Does not own: native ledger calls, report projection, bounds policy, or rendering.
//! Boundary: fetches one selected ledger resource without pagination or follow-up calls.

use super::{LiveIcSource, ledger_resource_url};
use crate::ic::{
    IcHostError, IcIcrcAnalyticsSource, IcIcrcIndexedCountKind, IcIcrcIndexedCountSourceData,
    IcIcrcTokenValueQuery, IcIcrcTokenValueSourceData, IcIcrcTokenValueSourceRow,
    IcIcrcTotalSupplyObservation, IcIcrcTotalSupplyQuery, IcIcrcTotalSupplySourceData,
    IcSourceRequest, source,
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl IcIcrcAnalyticsSource for LiveIcSource {
    fn fetch_indexed_count<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        ledger_canister_id: &'a str,
        kind: IcIcrcIndexedCountKind,
    ) -> crate::report::SourceFuture<'a, Result<IcIcrcIndexedCountSourceData, IcHostError>> {
        Box::pin(async move {
            let ledger_canister_id =
                source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
            let wire: IndexedCount = self
                .fetch_live(indexed_count_url(
                    &request.endpoint,
                    &ledger_canister_id,
                    kind,
                )?)
                .await?;
            Ok(IcIcrcIndexedCountSourceData {
                source: request.clone(),
                ledger_canister_id,
                kind,
                total: wire.total,
            })
        })
    }

    fn fetch_token_value_series<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        ledger_canister_id: &'a str,
        query: &'a IcIcrcTokenValueQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcIcrcTokenValueSourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_icrc_token_value_query(query)?;
            let ledger_canister_id =
                source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
            let wire: TokenValueSeries = self
                .fetch_live(token_value_url(
                    &request.endpoint,
                    &ledger_canister_id,
                    query,
                )?)
                .await?;
            Ok(IcIcrcTokenValueSourceData {
                source: request.clone(),
                ledger_canister_id: wire.ledger_canister_id,
                query: query.clone(),
                rows: wire.data,
            })
        })
    }

    fn fetch_total_supply_series<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        ledger_canister_id: &'a str,
        query: &'a IcIcrcTotalSupplyQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcIcrcTotalSupplySourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_icrc_total_supply_query(query)?;
            let ledger_canister_id =
                source::canonical_request_principal("ledger_canister_id", ledger_canister_id)?;
            let url = total_supply_url(&request.endpoint, &ledger_canister_id, query)?;
            let wire: TotalSupplySeries = self.fetch_live(url).await?;
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
        super::append_time_window(&mut pairs, query.start_unix_secs, query.end_unix_secs);
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
        super::append_time_series(
            &mut pairs,
            query.start_unix_secs,
            query.end_unix_secs,
            query.step_secs,
        );
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
    data: Vec<IcIcrcTokenValueSourceRow>,
}

#[cfg(test)]
mod tests;
