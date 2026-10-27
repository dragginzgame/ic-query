//! Module: ic::source::metric
//!
//! Responsibility: Dashboard metric source contract, bounds, and canonical projection.
//! Does not own: HTTP transport, shared provenance, canisters, or network resources.
//! Boundary: validates one explicitly bounded metric response before report construction.

use super::{
    inclusive_observation_count, invalid_request, invalid_source, report_provenance,
    validate_collection_end, validate_observation_timestamps, validate_provenance,
};
use crate::ic::{
    IcHostError, IcMetricQuery, IcMetricReport, IcMetricSeries, IcMetricSourceData,
    IcSourceRequest, MAX_IC_METRIC_OBSERVATIONS_PER_SERIES, MAX_IC_METRIC_STEP_SECS,
    MIN_IC_METRIC_TIMESTAMP,
};

///
/// IcMetricSource
///
/// Source contract for one bounded official Dashboard network metric query.
///

pub trait IcMetricSource: Sync {
    /// Fetch one metric window without pagination or automatic follow-up calls.
    fn fetch_metric<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcMetricQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcMetricSourceData, IcHostError>>;
}

pub(in crate::ic) fn validate_metric_request(
    now_unix_secs: u64,
    query: &IcMetricQuery,
) -> Result<(), IcHostError> {
    validate_metric_query(query)?;
    validate_collection_end(now_unix_secs, query.end_unix_secs)
}

pub(in crate::ic) fn validate_metric_query(query: &IcMetricQuery) -> Result<(), IcHostError> {
    if query.start_unix_secs < MIN_IC_METRIC_TIMESTAMP {
        return invalid_request(
            "query.start_unix_secs",
            format!("must be at least {MIN_IC_METRIC_TIMESTAMP}"),
        );
    }
    super::validate_query_time_order(query.start_unix_secs, query.end_unix_secs)?;
    if !(1..=MAX_IC_METRIC_STEP_SECS).contains(&query.step_secs) {
        return invalid_request(
            "query.step_secs",
            format!("must be between 1 and {MAX_IC_METRIC_STEP_SECS}"),
        );
    }

    let requested_observations =
        inclusive_observation_count(query.start_unix_secs, query.end_unix_secs, query.step_secs);
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

pub(in crate::ic) fn metric_report_from_source(
    request: &IcSourceRequest,
    query: &IcMetricQuery,
    mut source: IcMetricSourceData,
) -> Result<IcMetricReport, IcHostError> {
    validate_provenance(request, &source.source)?;
    super::validate_requested_query("metric", query, &source.query)?;

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

    let requested_observation_limit = usize::try_from(inclusive_observation_count(
        query.start_unix_secs,
        query.end_unix_secs,
        query.step_secs,
    ))
    .expect("metric observation limit fits usize");
    for series in series {
        if series.observations.len() > requested_observation_limit {
            return invalid_source(format!(
                "series {:?} returned {} observations for a request bounded to {requested_observation_limit}",
                series.name,
                series.observations.len()
            ));
        }
        validate_observation_timestamps(
            series
                .observations
                .iter()
                .map(|observation| observation.timestamp_unix_secs),
            query.start_unix_secs..=query.end_unix_secs,
            &format!("series {:?}", series.name),
        )?;
        for observation in &series.observations {
            if observation.value.is_empty() {
                return invalid_source(format!(
                    "series {:?} contains an empty raw value",
                    series.name
                ));
            }
        }
    }
    Ok(())
}
