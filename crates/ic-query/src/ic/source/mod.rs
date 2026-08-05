//! Module: ic::source
//!
//! Responsibility: shared Dashboard source provenance and capability facade.
//! Does not own: HTTP transport, capability-specific validation, command parsing, or rendering.
//! Boundary: treats live and custom source provenance as untrusted authority data.

mod canister;
mod icrc_analytics;
mod metric;
mod network;
mod node_status;

use crate::ic::{
    IC_DASHBOARD_AUTHORITY, IC_DASHBOARD_NETWORK, IC_DASHBOARD_REPORT_SCHEMA_VERSION,
    IcDashboardReportProvenance, IcHostError, IcSourceRequest,
};
use candid::Principal;

pub use canister::{IcCanisterCollectionSource, IcCanisterSource};
pub use icrc_analytics::IcIcrcAnalyticsSource;
pub use metric::IcMetricSource;
pub use network::IcNetworkSource;
pub use node_status::IcNodeStatusSource;

pub(super) use canister::{
    canonical_canister_id, canonical_page_cursors, count_report_from_source, normalized_filters,
    page_report_from_source, report_from_source, validate_page_cursor_exclusivity,
    validate_page_limit,
};
pub(super) use icrc_analytics::{
    icrc_indexed_count_report_from_source, icrc_token_value_report_from_source,
    icrc_total_supply_report_from_source, validate_icrc_token_value_query,
    validate_icrc_token_value_request, validate_icrc_total_supply_query,
    validate_icrc_total_supply_request,
};
pub(super) use metric::{
    metric_report_from_source, validate_metric_query, validate_metric_request,
};
pub(super) use network::{
    boundary_node_data_centers_report_from_source, daily_stats_report_from_source,
    validate_daily_stats_query, validate_daily_stats_request,
};
pub(super) use node_status::node_status_snapshot_from_source;

fn validate_provenance(
    expected: &IcSourceRequest,
    actual: &IcSourceRequest,
) -> Result<(), IcHostError> {
    for (field, expected, actual) in [
        (
            "source_endpoint",
            expected.endpoint.as_str(),
            actual.endpoint.as_str(),
        ),
        (
            "fetched_at",
            expected.fetched_at.as_str(),
            actual.fetched_at.as_str(),
        ),
        (
            "fetched_by",
            expected.fetched_by.as_str(),
            actual.fetched_by.as_str(),
        ),
    ] {
        if actual != expected {
            return invalid_source(format!(
                "{field} is {actual:?}, expected requested value {expected:?}"
            ));
        }
    }
    Ok(())
}

pub(in crate::ic) fn canonical_request_principal(
    field: &'static str,
    value: &str,
) -> Result<String, IcHostError> {
    Principal::from_text(value)
        .map(|principal| principal.to_text())
        .map_err(|error| IcHostError::InvalidPrincipal {
            field,
            reason: error.to_string(),
        })
}

fn validate_canonical_principal(field: &'static str, value: &str) -> Result<(), IcHostError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_source_value(format!("{field} {value:?}: {error}")))?;
    let canonical = principal.to_text();
    if canonical != value {
        return invalid_source(format!(
            "{field} {value:?} is not canonical principal text; expected {canonical:?}"
        ));
    }
    Ok(())
}

fn validate_principal_match(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), IcHostError> {
    validate_canonical_principal(field, actual)?;
    if actual == expected {
        return Ok(());
    }
    invalid_source(format!(
        "{field} is {actual:?}, expected requested principal {expected:?}"
    ))
}

fn validate_collection_end(now_unix_secs: u64, end_unix_secs: u64) -> Result<(), IcHostError> {
    if end_unix_secs > now_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must not be later than the collection time",
        );
    }
    Ok(())
}

fn inclusive_observation_count(start_unix_secs: u64, end_unix_secs: u64, step_secs: u32) -> u64 {
    (end_unix_secs - start_unix_secs) / u64::from(step_secs) + 1
}

fn report_provenance(source: IcSourceRequest) -> IcDashboardReportProvenance {
    IcDashboardReportProvenance {
        schema_version: IC_DASHBOARD_REPORT_SCHEMA_VERSION,
        network: IC_DASHBOARD_NETWORK.to_string(),
        authority: IC_DASHBOARD_AUTHORITY.to_string(),
        source_endpoint: source.endpoint,
        fetched_at: source.fetched_at,
        fetched_by: source.fetched_by,
        certified: false,
        point_in_time_guaranteed: false,
    }
}

fn invalid_source<T>(reason: impl Into<String>) -> Result<T, IcHostError> {
    Err(invalid_source_value(reason))
}

fn invalid_source_value(reason: impl Into<String>) -> IcHostError {
    IcHostError::InvalidSourceData {
        reason: reason.into(),
    }
}

fn invalid_request<T>(field: &'static str, reason: impl Into<String>) -> Result<T, IcHostError> {
    Err(IcHostError::InvalidRequest {
        field,
        reason: reason.into(),
    })
}
