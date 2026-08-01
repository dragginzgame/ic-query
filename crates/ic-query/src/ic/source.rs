//! Module: ic::source
//!
//! Responsibility: IC Dashboard source contract, validation, and canonical projection.
//! Does not own: HTTP transport, command parsing, or text rendering.
//! Boundary: treats live and custom source results as untrusted authority data.

use crate::{
    ic::{
        IC_DASHBOARD_AUTHORITY, IC_DASHBOARD_NETWORK, IC_DASHBOARD_REPORT_SCHEMA_VERSION,
        IcBoundaryNodeDataCenterRow, IcBoundaryNodeDataCentersReport,
        IcBoundaryNodeDataCentersSourceData, IcCanisterCountReport, IcCanisterCountSourceData,
        IcCanisterFilters, IcCanisterPageReport, IcCanisterPageRow, IcCanisterPageSourceData,
        IcCanisterReport, IcCanisterSourceData, IcCanisterUpgrade, IcDailyStatsQuery,
        IcDailyStatsReport, IcDailyStatsRow, IcDailyStatsSourceData, IcDashboardReportProvenance,
        IcHostError, IcMetricQuery, IcMetricReport, IcMetricSeries, IcMetricSourceData,
        IcSourceRequest, MAX_IC_BOUNDARY_NODE_DATA_CENTERS, MAX_IC_CANISTER_PAGE_LIMIT,
        MAX_IC_DAILY_STATS_ROWS, MAX_IC_DAILY_STATS_WINDOW_SECS,
        MAX_IC_METRIC_OBSERVATIONS_PER_SERIES, MAX_IC_METRIC_STEP_SECS,
        MIN_IC_DAILY_STATS_TIMESTAMP, MIN_IC_METRIC_TIMESTAMP,
    },
    subnet_catalog::format_utc_timestamp_secs,
};
use candid::Principal;
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

///
/// IcMetricSource
///
/// Source contract for one bounded official Dashboard network metric query.
///

pub trait IcMetricSource {
    /// Fetch one metric window without pagination or automatic follow-up calls.
    fn fetch_metric(
        &self,
        request: &IcSourceRequest,
        query: &IcMetricQuery,
    ) -> Result<IcMetricSourceData, IcHostError>;
}

///
/// IcCanisterSource
///
/// Source contract for fetching one canister from an IC Dashboard-compatible API.
///

pub trait IcCanisterSource {
    /// Fetch one canister with explicit endpoint and collection provenance.
    fn fetch_canister(
        &self,
        request: &IcSourceRequest,
        canister_id: &str,
    ) -> Result<IcCanisterSourceData, IcHostError>;
}

///
/// IcCanisterCollectionSource
///
/// Source contract for bounded IC Dashboard canister discovery.
///

pub trait IcCanisterCollectionSource {
    /// Fetch one filtered canister count with explicit collection provenance.
    fn fetch_canister_count(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
    ) -> Result<IcCanisterCountSourceData, IcHostError>;

    /// Fetch at most `limit` rows without automatically following a cursor.
    fn fetch_canister_page(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
        limit: u16,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<IcCanisterPageSourceData, IcHostError>;
}

pub(super) fn validate_metric_request(
    now_unix_secs: u64,
    query: &IcMetricQuery,
) -> Result<(), IcHostError> {
    validate_metric_query(query)?;
    if query.end_unix_secs > now_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must not be later than the collection time",
        );
    }
    Ok(())
}

pub(super) fn validate_metric_query(query: &IcMetricQuery) -> Result<(), IcHostError> {
    if query.start_unix_secs < MIN_IC_METRIC_TIMESTAMP {
        return invalid_request(
            "query.start_unix_secs",
            format!("must be at least {MIN_IC_METRIC_TIMESTAMP}"),
        );
    }
    if query.end_unix_secs < query.start_unix_secs {
        return invalid_request(
            "query.end_unix_secs",
            "must be greater than or equal to query.start_unix_secs",
        );
    }
    if !(1..=MAX_IC_METRIC_STEP_SECS).contains(&query.step_secs) {
        return invalid_request(
            "query.step_secs",
            format!("must be between 1 and {MAX_IC_METRIC_STEP_SECS}"),
        );
    }

    let requested_observations = metric_observation_limit(query);
    if requested_observations > MAX_IC_METRIC_OBSERVATIONS_PER_SERIES {
        return invalid_request(
            "query",
            format!(
                "would request {requested_observations} observations per series; maximum is {MAX_IC_METRIC_OBSERVATIONS_PER_SERIES}"
            ),
        );
    }
    Ok(())
}

pub(super) fn validate_daily_stats_request(
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

pub(super) fn validate_daily_stats_query(query: &IcDailyStatsQuery) -> Result<(), IcHostError> {
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

pub(super) fn canonical_canister_id(value: &str) -> Result<String, IcHostError> {
    canonical_request_principal("canister_id", value)
}

pub(super) fn normalized_filters(
    filters: &IcCanisterFilters,
) -> Result<IcCanisterFilters, IcHostError> {
    let mut filters = filters.clone();
    filters.subnet_id = filters
        .subnet_id
        .as_deref()
        .map(|value| canonical_request_principal("filters.subnet_id", value))
        .transpose()?;
    filters.controller_id = filters
        .controller_id
        .as_deref()
        .map(|value| canonical_request_principal("filters.controller_id", value))
        .transpose()?;
    normalize_string_filters("filters.languages", &mut filters.languages)?;
    normalize_string_filters("filters.canister_types", &mut filters.canister_types)?;

    if let Some(query) = filters.query.as_deref() {
        let length = query.chars().count();
        if !(2..=100).contains(&length) {
            return invalid_request("filters.query", "must contain between 2 and 100 characters");
        }
    }
    Ok(filters)
}

pub(super) fn canonical_page_cursor(
    field: &'static str,
    cursor: Option<&str>,
) -> Result<Option<String>, IcHostError> {
    cursor
        .map(|value| canonical_request_principal(field, value))
        .transpose()
}

pub(super) fn validate_page_limit(limit: u16) -> Result<(), IcHostError> {
    if (1..=MAX_IC_CANISTER_PAGE_LIMIT).contains(&limit) {
        return Ok(());
    }
    invalid_request(
        "limit",
        format!("must be between 1 and {MAX_IC_CANISTER_PAGE_LIMIT}"),
    )
}

fn canonical_request_principal(field: &'static str, value: &str) -> Result<String, IcHostError> {
    Principal::from_text(value)
        .map(|principal| principal.to_text())
        .map_err(|error| IcHostError::InvalidPrincipal {
            field,
            reason: error.to_string(),
        })
}

fn normalize_string_filters(field: &'static str, values: &mut [String]) -> Result<(), IcHostError> {
    if values.iter().any(String::is_empty) {
        return invalid_request(field, "values must not be empty");
    }
    values.sort_unstable();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return invalid_request(field, "values must be unique");
    }
    Ok(())
}

pub(super) fn report_from_source(
    request: &IcSourceRequest,
    requested_canister_id: &str,
    mut source: IcCanisterSourceData,
) -> Result<IcCanisterReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_principal_match("canister_id", requested_canister_id, &source.canister_id)?;
    validate_canonical_principal("subnet_id", &source.subnet_id)?;

    let mut seen_controllers = HashSet::with_capacity(source.controllers.len());
    for controller in &source.controllers {
        validate_canonical_principal("controller", controller)?;
        if !seen_controllers.insert(controller.clone()) {
            return invalid_source(format!("duplicate controller principal {controller}"));
        }
    }
    source.controllers.sort_unstable();

    validate_optional_module_hash("module_hash", &source.module_hash)?;
    if source.dashboard_updated_at.is_empty() {
        return invalid_source("dashboard_updated_at must not be empty");
    }

    if let Some(upgrades) = source.upgrades.as_mut() {
        validate_upgrades(upgrades)?;
        upgrades.sort_unstable_by(|left, right| {
            right
                .executed_timestamp_seconds
                .cmp(&left.executed_timestamp_seconds)
                .then_with(|| right.proposal_id.cmp(&left.proposal_id))
                .then_with(|| left.module_hash.cmp(&right.module_hash))
        });
    }

    Ok(IcCanisterReport {
        provenance: report_provenance(source.source),
        canister_id: source.canister_id,
        dashboard_id: source.dashboard_id,
        canister_type: source.canister_type,
        name: source.name,
        subnet_id: source.subnet_id,
        controllers: source.controllers,
        language: source.language,
        module_hash: source.module_hash,
        dashboard_updated_at: source.dashboard_updated_at,
        upgrade_count: source.upgrades.as_ref().map(Vec::len),
        upgrades: source.upgrades,
    })
}

pub(super) fn boundary_node_data_centers_report_from_source(
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

pub(super) fn daily_stats_report_from_source(
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

pub(super) fn metric_report_from_source(
    request: &IcSourceRequest,
    query: &IcMetricQuery,
    mut source: IcMetricSourceData,
) -> Result<IcMetricReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    if source.query != *query {
        return invalid_source(format!(
            "metric query is {:?}, expected requested query {query:?}",
            source.query
        ));
    }

    validate_metric_series(query, &mut source.series)?;
    let returned_observation_count = source
        .series
        .iter()
        .map(|series| series.observations.len())
        .sum();

    Ok(IcMetricReport {
        provenance: report_provenance(source.source),
        query: source.query,
        returned_series_count: source.series.len(),
        returned_observation_count,
        series: source.series,
    })
}

pub(super) fn count_report_from_source(
    request: &IcSourceRequest,
    filters: &IcCanisterFilters,
    source: IcCanisterCountSourceData,
) -> Result<IcCanisterCountReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_filter_match(filters, &source.filters)?;

    Ok(IcCanisterCountReport {
        provenance: report_provenance(source.source),
        filters: source.filters,
        total: source.total,
    })
}

pub(super) fn page_report_from_source(
    request: &IcSourceRequest,
    filters: &IcCanisterFilters,
    limit: u16,
    after: Option<&str>,
    before: Option<&str>,
    mut source: IcCanisterPageSourceData,
) -> Result<IcCanisterPageReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    validate_filter_match(filters, &source.filters)?;
    if source.requested_limit != limit {
        return invalid_source(format!(
            "requested_limit is {}, expected requested value {limit}",
            source.requested_limit
        ));
    }
    validate_optional_match("after", after, source.after.as_deref())?;
    validate_optional_match("before", before, source.before.as_deref())?;
    if source.rows.len() > usize::from(limit) {
        return invalid_source(format!(
            "source returned {} rows for requested limit {limit}",
            source.rows.len()
        ));
    }

    validate_page_rows(&mut source.rows)?;
    validate_source_cursor("previous_cursor", source.previous_cursor.as_deref())?;
    validate_source_cursor("next_cursor", source.next_cursor.as_deref())?;
    validate_page_boundary_cursor(
        "previous_cursor",
        source.previous_cursor.as_deref(),
        source.rows.first(),
    )?;
    validate_page_boundary_cursor(
        "next_cursor",
        source.next_cursor.as_deref(),
        source.rows.last(),
    )?;

    Ok(IcCanisterPageReport {
        provenance: report_provenance(source.source),
        filters: source.filters,
        requested_limit: source.requested_limit,
        returned_count: source.rows.len(),
        after: source.after,
        before: source.before,
        previous_cursor: source.previous_cursor,
        next_cursor: source.next_cursor,
        rows: source.rows,
    })
}

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

fn validate_metric_series(
    query: &IcMetricQuery,
    series: &mut [IcMetricSeries],
) -> Result<(), IcHostError> {
    series.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    let expected_names = query.metric.series_names();
    if series.len() != expected_names.len()
        || expected_names
            .iter()
            .any(|name| !series.iter().any(|series| series.name == *name))
    {
        let actual_names = series
            .iter()
            .map(|series| series.name.as_str())
            .collect::<Vec<_>>();
        return invalid_source(format!(
            "metric series names are {actual_names:?}, expected {expected_names:?}"
        ));
    }

    let requested_observation_limit = usize::try_from(metric_observation_limit(query))
        .expect("metric observation limit fits usize");
    for series in series {
        if series.observations.len() > requested_observation_limit {
            return invalid_source(format!(
                "series {:?} returned {} observations for a request bounded to {requested_observation_limit}",
                series.name,
                series.observations.len()
            ));
        }
        let mut previous_timestamp = None;
        for observation in &series.observations {
            if !(query.start_unix_secs..=query.end_unix_secs)
                .contains(&observation.timestamp_unix_secs)
            {
                return invalid_source(format!(
                    "series {:?} observation timestamp {} is outside the requested window",
                    series.name, observation.timestamp_unix_secs
                ));
            }
            if previous_timestamp
                .is_some_and(|previous| previous >= observation.timestamp_unix_secs)
            {
                return invalid_source(format!(
                    "series {:?} observations must be strictly ordered by timestamp",
                    series.name
                ));
            }
            if observation.value.is_empty() {
                return invalid_source(format!(
                    "series {:?} contains an empty raw value",
                    series.name
                ));
            }
            previous_timestamp = Some(observation.timestamp_unix_secs);
        }
    }
    Ok(())
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

fn metric_observation_limit(query: &IcMetricQuery) -> u64 {
    (query.end_unix_secs - query.start_unix_secs) / u64::from(query.step_secs) + 1
}

fn validate_filter_match(
    expected: &IcCanisterFilters,
    actual: &IcCanisterFilters,
) -> Result<(), IcHostError> {
    if actual == expected {
        return Ok(());
    }
    invalid_source(format!(
        "filters are {actual:?}, expected requested filters {expected:?}"
    ))
}

fn validate_optional_match(
    field: &'static str,
    expected: Option<&str>,
    actual: Option<&str>,
) -> Result<(), IcHostError> {
    if actual == expected {
        return Ok(());
    }
    invalid_source(format!(
        "{field} is {actual:?}, expected requested value {expected:?}"
    ))
}

fn validate_page_rows(rows: &mut [IcCanisterPageRow]) -> Result<(), IcHostError> {
    let mut seen_canisters = HashSet::with_capacity(rows.len());
    let mut seen_dashboard_ids = HashSet::with_capacity(rows.len());
    let mut previous_canister_id: Option<&str> = None;

    for row in rows {
        validate_canonical_principal("row.canister_id", &row.canister_id)?;
        validate_canonical_principal("row.subnet_id", &row.subnet_id)?;
        if !seen_canisters.insert(row.canister_id.clone()) {
            return invalid_source(format!("duplicate canister_id {}", row.canister_id));
        }
        if !seen_dashboard_ids.insert(row.dashboard_id) {
            return invalid_source(format!("duplicate dashboard_id {}", row.dashboard_id));
        }
        if previous_canister_id.is_some_and(|previous| previous >= row.canister_id.as_str()) {
            return invalid_source("rows must be strictly ordered by canister_id");
        }
        previous_canister_id = Some(&row.canister_id);

        let mut seen_controllers = HashSet::with_capacity(row.controllers.len());
        for controller in &row.controllers {
            validate_canonical_principal("row.controller", &controller.principal_id)?;
            if !seen_controllers.insert(controller.principal_id.clone()) {
                return invalid_source(format!(
                    "duplicate controller principal {} for canister {}",
                    controller.principal_id, row.canister_id
                ));
            }
        }
        row.controllers.sort_unstable_by(|left, right| {
            left.principal_id
                .cmp(&right.principal_id)
                .then_with(|| left.raw_metadata.cmp(&right.raw_metadata))
        });
        validate_optional_module_hash("row.module_hash", &row.module_hash)?;
        if row.dashboard_updated_at.is_empty() {
            return invalid_source(format!(
                "dashboard_updated_at must not be empty for canister {}",
                row.canister_id
            ));
        }
    }
    Ok(())
}

fn validate_source_cursor(field: &'static str, cursor: Option<&str>) -> Result<(), IcHostError> {
    if let Some(cursor) = cursor {
        validate_canonical_principal(field, cursor)?;
    }
    Ok(())
}

fn validate_page_boundary_cursor(
    field: &'static str,
    cursor: Option<&str>,
    boundary: Option<&IcCanisterPageRow>,
) -> Result<(), IcHostError> {
    if let (Some(cursor), Some(boundary)) = (cursor, boundary)
        && cursor != boundary.canister_id
    {
        return invalid_source(format!(
            "{field} is {cursor:?}, expected page boundary {:?}",
            boundary.canister_id
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

fn validate_upgrades(upgrades: &[IcCanisterUpgrade]) -> Result<(), IcHostError> {
    let mut proposal_ids = HashSet::with_capacity(upgrades.len());
    for upgrade in upgrades {
        validate_module_hash("upgrade.module_hash", &upgrade.module_hash)?;
        if !proposal_ids.insert(upgrade.proposal_id) {
            return invalid_source(format!(
                "duplicate upgrade proposal_id {}",
                upgrade.proposal_id
            ));
        }
    }
    Ok(())
}

fn validate_optional_module_hash(field: &'static str, value: &str) -> Result<(), IcHostError> {
    if value.is_empty() {
        return Ok(());
    }
    validate_module_hash(field, value)
}

fn validate_module_hash(field: &'static str, value: &str) -> Result<(), IcHostError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    invalid_source(format!(
        "{field} must be 64-character lowercase hexadecimal text"
    ))
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
