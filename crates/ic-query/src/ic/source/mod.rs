//! Module: ic::source
//!
//! Responsibility: shared Dashboard source provenance and capability facade.
//! Does not own: HTTP transport, capability-specific validation, command parsing, or rendering.
//! Boundary: treats live and custom source provenance as untrusted authority data.

mod canister;
mod icrc_analytics;
mod metric;
mod network;

use crate::ic::{
    IC_DASHBOARD_AUTHORITY, IC_DASHBOARD_NETWORK, IC_DASHBOARD_REPORT_SCHEMA_VERSION,
    IcDashboardReportProvenance, IcHostError, IcSourceRequest,
};

pub use canister::{IcCanisterCollectionSource, IcCanisterSource};
pub use icrc_analytics::IcIcrcAnalyticsSource;
pub use metric::IcMetricSource;
pub use network::IcNetworkSource;

pub(super) use canister::{
    canonical_canister_id, canonical_page_cursor, canonical_request_principal,
    count_report_from_source, normalized_filters, page_report_from_source, report_from_source,
    validate_page_limit, validate_principal_match,
};
pub(super) use icrc_analytics::{
    icrc_total_supply_report_from_source, validate_icrc_total_supply_query,
    validate_icrc_total_supply_request,
};
pub(super) use metric::{
    metric_report_from_source, validate_metric_query, validate_metric_request,
};
pub(super) use network::{
    boundary_node_data_centers_report_from_source, daily_stats_report_from_source,
    validate_daily_stats_query, validate_daily_stats_request,
};

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
