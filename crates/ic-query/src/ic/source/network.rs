//! Module: ic::source::network
//!
//! Responsibility: Dashboard network-resource source contract, validation, and projection.
//! Does not own: HTTP transport, shared provenance, canisters, or metric series.
//! Boundary: validates bounded daily statistics and finite boundary-node resources.

use super::{
    invalid_request, invalid_source, invalid_source_value, report_provenance, validate_provenance,
};
use crate::{
    ic::{
        IcBoundaryNodeDataCenterRow, IcBoundaryNodeDataCentersReport,
        IcBoundaryNodeDataCentersSourceData, IcDailyStatsQuery, IcDailyStatsReport,
        IcDailyStatsRow, IcDailyStatsSourceData, IcHostError, IcSourceRequest,
        MAX_IC_BOUNDARY_NODE_DATA_CENTERS, MAX_IC_DAILY_STATS_ROWS, MAX_IC_DAILY_STATS_WINDOW_SECS,
        MIN_IC_DAILY_STATS_TIMESTAMP,
    },
    subnet_catalog::format_utc_timestamp_secs,
};
use std::collections::HashSet;

///
/// IcNetworkSource
///
/// Source contract for bounded official Dashboard network reports.
///

pub trait IcNetworkSource {
    /// Fetch the complete boundary-node data-center resource in one request.
    fn fetch_boundary_node_data_centers(
        &self,
        request: &IcSourceRequest,
    ) -> Result<IcBoundaryNodeDataCentersSourceData, IcHostError>;

    /// Fetch one explicitly bounded daily-statistics window in one request.
    fn fetch_daily_stats(
        &self,
        request: &IcSourceRequest,
        query: &IcDailyStatsQuery,
    ) -> Result<IcDailyStatsSourceData, IcHostError>;
}

pub(in crate::ic) fn validate_daily_stats_request(
    now_unix_secs: u64,
    query: &IcDailyStatsQuery,
) -> Result<(), IcHostError> {
    validate_daily_stats_query(query)?;
    if query.end_unix_secs > now_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must not be later than the collection time",
        );
    }
    Ok(())
}

pub(in crate::ic) fn validate_daily_stats_query(
    query: &IcDailyStatsQuery,
) -> Result<(), IcHostError> {
    if query.start_unix_secs < MIN_IC_DAILY_STATS_TIMESTAMP {
        return invalid_request(
            "query.start_unix_secs",
            format!("must be at least {MIN_IC_DAILY_STATS_TIMESTAMP}"),
        );
    }
    if query.end_unix_secs < query.start_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must be greater than or equal to query.start_unix_secs",
        );
    }
    let window_secs = query.end_unix_secs - query.start_unix_secs;
    if window_secs > MAX_IC_DAILY_STATS_WINDOW_SECS {
        return invalid_request(
            "query",
            format!("window is {window_secs} seconds; maximum is {MAX_IC_DAILY_STATS_WINDOW_SECS}"),
        );
    }
    Ok(())
}

pub(in crate::ic) fn boundary_node_data_centers_report_from_source(
    request: &IcSourceRequest,
    mut source: IcBoundaryNodeDataCentersSourceData,
) -> Result<IcBoundaryNodeDataCentersReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    let total_node_count = validate_boundary_node_data_centers(&mut source.rows)?;

    Ok(IcBoundaryNodeDataCentersReport {
        provenance: report_provenance(source.source),
        data_center_count: source.rows.len(),
        total_node_count,
        rows: source.rows,
    })
}

pub(in crate::ic) fn daily_stats_report_from_source(
    request: &IcSourceRequest,
    query: &IcDailyStatsQuery,
    mut source: IcDailyStatsSourceData,
) -> Result<IcDailyStatsReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    if source.query != *query {
        return invalid_source(format!(
            "daily-statistics query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }
    validate_daily_stats_rows(query, &mut source.rows)?;

    Ok(IcDailyStatsReport {
        provenance: report_provenance(source.source),
        query: source.query,
        returned_day_count: source.rows.len(),
        rows: source.rows,
    })
}

fn validate_boundary_node_data_centers(
    rows: &mut [IcBoundaryNodeDataCenterRow],
) -> Result<u64, IcHostError> {
    if rows.len() > MAX_IC_BOUNDARY_NODE_DATA_CENTERS {
        return invalid_source(format!(
            "source returned {} boundary-node data centers; maximum is {MAX_IC_BOUNDARY_NODE_DATA_CENTERS}",
            rows.len()
        ));
    }

    let mut seen_ids = HashSet::with_capacity(rows.len());
    let mut total_node_count = 0_u64;
    for row in rows.iter() {
        for (field, value) in [
            ("row.dc_id", row.dc_id.as_str()),
            ("row.name", row.name.as_str()),
            ("row.owner", row.owner.as_str()),
            ("row.region", row.region.as_str()),
        ] {
            if value.is_empty() {
                return invalid_source(format!("{field} must not be empty"));
            }
        }
        if !seen_ids.insert(row.dc_id.as_str()) {
            return invalid_source(format!("duplicate data-center id {:?}", row.dc_id));
        }
        validate_coordinate("row.latitude", &row.latitude, -90.0, 90.0)?;
        validate_coordinate("row.longitude", &row.longitude, -180.0, 180.0)?;

        let node_count = row.total_nodes.parse::<u64>().map_err(|error| {
            invalid_source_value(format!(
                "row.total_nodes {:?} is not an unsigned decimal count: {error}",
                row.total_nodes
            ))
        })?;
        if node_count.to_string() != row.total_nodes {
            return invalid_source(format!(
                "row.total_nodes {:?} is not canonical unsigned decimal text",
                row.total_nodes
            ));
        }
        total_node_count = total_node_count
            .checked_add(node_count)
            .ok_or_else(|| invalid_source_value("boundary-node total overflows u64"))?;
    }
    rows.sort_unstable_by(|left, right| left.dc_id.cmp(&right.dc_id));
    Ok(total_node_count)
}

fn validate_daily_stats_rows(
    query: &IcDailyStatsQuery,
    rows: &mut [IcDailyStatsRow],
) -> Result<(), IcHostError> {
    if rows.len() > MAX_IC_DAILY_STATS_ROWS {
        return invalid_source(format!(
            "source returned {} daily-statistics rows; maximum is {MAX_IC_DAILY_STATS_ROWS}",
            rows.len()
        ));
    }

    let mut seen_days = HashSet::with_capacity(rows.len());
    let mut seen_timestamps = HashSet::with_capacity(rows.len());
    for row in rows.iter() {
        if !(query.start_unix_secs..=query.end_unix_secs).contains(&row.timestamp_unix_secs) {
            return invalid_source(format!(
                "daily-statistics timestamp {} is outside the requested window",
                row.timestamp_unix_secs
            ));
        }
        if !seen_timestamps.insert(row.timestamp_unix_secs) {
            return invalid_source(format!(
                "duplicate daily-statistics timestamp {}",
                row.timestamp_unix_secs
            ));
        }
        if !seen_days.insert(row.day.as_str()) {
            return invalid_source(format!("duplicate daily-statistics day {:?}", row.day));
        }
        let timestamp = format_utc_timestamp_secs(row.timestamp_unix_secs);
        let expected_day = timestamp
            .split_once('T')
            .expect("formatted timestamp contains a date separator")
            .0;
        if row.day != expected_day {
            return invalid_source(format!(
                "daily-statistics day {:?} does not match timestamp date {expected_day:?}",
                row.day
            ));
        }
        for (field, value) in [
            (
                "row.average_query_transactions_per_second",
                row.average_query_transactions_per_second.as_str(),
            ),
            (
                "row.average_update_transactions_per_second",
                row.average_update_transactions_per_second.as_str(),
            ),
            (
                "row.average_transactions_per_second",
                row.average_transactions_per_second.as_str(),
            ),
            (
                "row.max_query_transactions_per_second",
                row.max_query_transactions_per_second.as_str(),
            ),
            (
                "row.max_update_transactions_per_second",
                row.max_update_transactions_per_second.as_str(),
            ),
            (
                "row.max_total_transactions_per_second",
                row.max_total_transactions_per_second.as_str(),
            ),
            (
                "row.blocks_per_second_average",
                row.blocks_per_second_average.as_str(),
            ),
        ] {
            validate_nonnegative_decimal(field, value)?;
        }
    }
    rows.sort_unstable_by_key(|row| row.timestamp_unix_secs);
    Ok(())
}

fn validate_nonnegative_decimal(field: &'static str, raw: &str) -> Result<(), IcHostError> {
    let value = raw.parse::<f64>().map_err(|error| {
        invalid_source_value(format!("{field} {raw:?} is not decimal text: {error}"))
    })?;
    if value.is_finite() && value >= 0.0 {
        return Ok(());
    }
    invalid_source(format!("{field} {raw:?} must be finite and nonnegative"))
}

fn validate_coordinate(
    field: &'static str,
    raw: &str,
    minimum: f64,
    maximum: f64,
) -> Result<(), IcHostError> {
    let value = raw.parse::<f64>().map_err(|error| {
        invalid_source_value(format!("{field} {raw:?} is not decimal text: {error}"))
    })?;
    if value.is_finite() && (minimum..=maximum).contains(&value) {
        return Ok(());
    }
    invalid_source(format!(
        "{field} {raw:?} is outside the inclusive range {minimum} through {maximum}"
    ))
}
