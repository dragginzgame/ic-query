//! Module: ic::source::icrc_analytics
//!
//! Responsibility: official ICRC analytics source contract, bounds, and projection.
//! Does not own: native ledger queries, HTTP transport, command parsing, or rendering.
//! Boundary: validates one ledger-scoped total-supply series before report construction.

use super::{
    invalid_request, invalid_source, report_provenance, validate_principal_match,
    validate_provenance,
};
use crate::ic::{
    IcHostError, IcIcrcTotalSupplyQuery, IcIcrcTotalSupplyReport, IcIcrcTotalSupplySourceData,
    IcSourceRequest, MAX_ICRC_ANALYTICS_OBSERVATIONS, MIN_ICRC_ANALYTICS_TIMESTAMP,
};

///
/// IcIcrcAnalyticsSource
///
/// Source contract for bounded official Dashboard ICRC analytics queries.
///

pub trait IcIcrcAnalyticsSource {
    /// Fetch one total-supply series without pagination or automatic follow-up calls.
    fn fetch_total_supply_series(
        &self,
        request: &IcSourceRequest,
        ledger_canister_id: &str,
        query: &IcIcrcTotalSupplyQuery,
    ) -> Result<IcIcrcTotalSupplySourceData, IcHostError>;
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
