//! Module: ic::source::icrc_analytics
//!
//! Responsibility: official ICRC analytics source contract, bounds, and projection.
//! Does not own: native ledger queries, HTTP transport, command parsing, or rendering.
//! Boundary: validates ledger-scoped scalar and bounded-series evidence before reports.

use super::{
    invalid_request, invalid_source, report_provenance, validate_principal_match,
    validate_provenance,
};
use crate::ic::{
    IcHostError, IcIcrcIndexedCountKind, IcIcrcIndexedCountReport, IcIcrcIndexedCountSourceData,
    IcIcrcTokenValueQuery, IcIcrcTokenValueReport, IcIcrcTokenValueRow, IcIcrcTokenValueSourceData,
    IcIcrcTotalSupplyQuery, IcIcrcTotalSupplyReport, IcIcrcTotalSupplySourceData, IcSourceRequest,
    MAX_ICRC_ANALYTICS_OBSERVATIONS, MAX_ICRC_TOKEN_VALUE_ROWS, MAX_ICRC_TOKEN_VALUE_WINDOW_SECS,
    MIN_ICRC_ANALYTICS_TIMESTAMP,
};

///
/// IcIcrcAnalyticsSource
///
/// Source contract for bounded official Dashboard ICRC analytics queries.
///

pub trait IcIcrcAnalyticsSource {
    /// Fetch one current scalar count without requesting indexed rows.
    fn fetch_indexed_count(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        kind: IcIcrcIndexedCountKind,
    ) -> Result<IcIcrcIndexedCountSourceData, IcHostError>;

    /// Fetch one bounded token-value series without pagination or follow-up calls.
    fn fetch_token_value_series(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcTokenValueQuery,
    ) -> Result<IcIcrcTokenValueSourceData, IcHostError>;

    /// Fetch one total-supply series without pagination or automatic follow-up calls.
    fn fetch_total_supply_series(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcTotalSupplyQuery,
    ) -> Result<IcIcrcTotalSupplySourceData, IcHostError>;
}

pub(in crate::ic) fn validate_icrc_token_value_request(
    now_unix_secs: u64,
    query: &IcIcrcTokenValueQuery,
) -> Result<(), IcHostError> {
    validate_icrc_token_value_query(query)?;
    if query.end_unix_secs > now_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must not be later than the collection time",
        );
    }
    Ok(())
}

pub(in crate::ic) fn validate_icrc_token_value_query(
    query: &IcIcrcTokenValueQuery,
) -> Result<(), IcHostError> {
    if query.end_unix_secs < query.start_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must be greater than or equal to query.start_unix_secs",
        );
    }
    if query.end_unix_secs - query.start_unix_secs > MAX_ICRC_TOKEN_VALUE_WINDOW_SECS {
        return invalid_request(
            "query",
            format!("window must not exceed {MAX_ICRC_TOKEN_VALUE_WINDOW_SECS} seconds"),
        );
    }
    if !(1..=MAX_ICRC_TOKEN_VALUE_ROWS).contains(&query.limit) {
        return invalid_request(
            "query.limit",
            format!("must be between 1 and {MAX_ICRC_TOKEN_VALUE_ROWS}"),
        );
    }
    Ok(())
}

pub(in crate::ic) fn icrc_token_value_report_from_source(
    request: &IcSourceRequest,
    ledger_canister_id: &str,
    query: &IcIcrcTokenValueQuery,
    source: IcIcrcTokenValueSourceData,
) -> Result<IcIcrcTokenValueReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_principal_match(
        "ledger_canister_id",
        ledger_canister_id,
        &source.ledger_canister_id,
    )?;
    if source.query != *query {
        return invalid_source(format!(
            "ICRC token-value query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }

    let returned_row_count = source.rows.len();
    if returned_row_count > usize::from(query.limit)
        || returned_row_count > usize::from(MAX_ICRC_TOKEN_VALUE_ROWS)
    {
        return invalid_source(format!(
            "token-value series returned {returned_row_count} rows for a request limited to {}",
            query.limit
        ));
    }

    let mut previous_timestamp = None;
    let mut rows = Vec::with_capacity(returned_row_count);
    for (index, row) in source.rows.into_iter().enumerate() {
        let Some(timestamp_unix_secs) = row.timestamp_unix_secs else {
            return invalid_source(format!("token-value row {index} is missing its timestamp"));
        };
        if !(query.start_unix_secs..=query.end_unix_secs).contains(&timestamp_unix_secs) {
            return invalid_source(format!(
                "token-value row {index} timestamp {timestamp_unix_secs} is outside the requested window"
            ));
        }
        if previous_timestamp.is_some_and(|previous| previous > timestamp_unix_secs) {
            return invalid_source("token-value rows must be ordered by nondecreasing timestamp");
        }
        previous_timestamp = Some(timestamp_unix_secs);
        rows.push(IcIcrcTokenValueRow {
            price: row.price,
            volume_24h: row.volume_24h,
            price_usd: row.price_usd,
            volume_24h_usd: row.volume_24h_usd,
            source: row.source,
            source_url: row.source_url,
            timestamp_unix_secs,
        });
    }

    Ok(IcIcrcTokenValueReport {
        provenance: report_provenance(source.source),
        ledger_canister_id: source.ledger_canister_id,
        query: source.query,
        returned_row_count,
        limit_reached: returned_row_count == usize::from(query.limit),
        rows,
    })
}

pub(in crate::ic) fn icrc_indexed_count_report_from_source(
    request: &IcSourceRequest,
    ledger_canister_id: &str,
    kind: IcIcrcIndexedCountKind,
    source: IcIcrcIndexedCountSourceData,
) -> Result<IcIcrcIndexedCountReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_principal_match(
        "ledger_canister_id",
        ledger_canister_id,
        &source.ledger_canister_id,
    )?;
    if source.kind != kind {
        return invalid_source(format!(
            "ICRC indexed-count kind is {:?}, expected requested kind {kind:?}",
            source.kind
        ));
    }
    Ok(IcIcrcIndexedCountReport {
        provenance: report_provenance(source.source),
        ledger_canister_id: source.ledger_canister_id,
        kind: source.kind,
        total: source.total,
    })
}

pub(in crate::ic) fn validate_icrc_total_supply_request(
    now_unix_secs: u64,
    query: &IcIcrcTotalSupplyQuery,
) -> Result<(), IcHostError> {
    validate_icrc_total_supply_query(query)?;
    if query.end_unix_secs > now_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must not be later than the collection time",
        );
    }
    Ok(())
}

pub(in crate::ic) fn validate_icrc_total_supply_query(
    query: &IcIcrcTotalSupplyQuery,
) -> Result<(), IcHostError> {
    if query.start_unix_secs < MIN_ICRC_ANALYTICS_TIMESTAMP {
        return invalid_request(
            "query.start_unix_secs",
            format!("must be at least {MIN_ICRC_ANALYTICS_TIMESTAMP}"),
        );
    }
    if query.end_unix_secs < query.start_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must be greater than or equal to query.start_unix_secs",
        );
    }
    if !matches!(query.step_secs, 3_600 | 86_400) {
        return invalid_request("query.step_secs", "must be either 3600 or 86400");
    }

    let requested_observations = icrc_total_supply_observation_limit(query);
    if requested_observations > MAX_ICRC_ANALYTICS_OBSERVATIONS {
        return invalid_request(
            "query",
            format!(
                "would request {requested_observations} observations; maximum is {MAX_ICRC_ANALYTICS_OBSERVATIONS}"
            ),
        );
    }
    Ok(())
}

pub(in crate::ic) fn icrc_total_supply_report_from_source(
    request: &IcSourceRequest,
    ledger_canister_id: &str,
    query: &IcIcrcTotalSupplyQuery,
    source: IcIcrcTotalSupplySourceData,
) -> Result<IcIcrcTotalSupplyReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_principal_match(
        "ledger_canister_id",
        ledger_canister_id,
        &source.ledger_canister_id,
    )?;
    if source.query != *query {
        return invalid_source(format!(
            "ICRC total-supply query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }

    let requested_observation_limit = icrc_total_supply_observation_limit(query);
    let returned_count = u64::try_from(source.observations.len()).unwrap_or(u64::MAX);
    if returned_count > requested_observation_limit
        || returned_count > MAX_ICRC_ANALYTICS_OBSERVATIONS
    {
        return invalid_source(format!(
            "total-supply series returned {returned_count} observations for a request bounded to {requested_observation_limit}"
        ));
    }

    let mut previous_timestamp = None;
    for observation in &source.observations {
        if !(query.start_unix_secs..=query.end_unix_secs).contains(&observation.timestamp_unix_secs)
        {
            return invalid_source(format!(
                "total-supply observation timestamp {} is outside the requested window",
                observation.timestamp_unix_secs
            ));
        }
        if previous_timestamp.is_some_and(|previous| previous >= observation.timestamp_unix_secs) {
            return invalid_source(
                "total-supply observations must be strictly ordered by timestamp",
            );
        }
        if !is_canonical_unsigned_decimal(&observation.total_supply_base_units) {
            return invalid_source(format!(
                "total-supply value {:?} is not canonical unsigned decimal text",
                observation.total_supply_base_units
            ));
        }
        previous_timestamp = Some(observation.timestamp_unix_secs);
    }

    Ok(IcIcrcTotalSupplyReport {
        provenance: report_provenance(source.source),
        ledger_canister_id: source.ledger_canister_id,
        query: source.query,
        requested_observation_limit,
        returned_observation_count: source.observations.len(),
        observations: source.observations,
    })
}

pub(in crate::ic) fn icrc_total_supply_observation_limit(query: &IcIcrcTotalSupplyQuery) -> u64 {
    (query.end_unix_secs - query.start_unix_secs) / u64::from(query.step_secs) + 1
}

fn is_canonical_unsigned_decimal(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}
