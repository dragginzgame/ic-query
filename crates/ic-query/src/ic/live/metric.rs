//! Module: ic::live::metric
//!
//! Responsibility: live Dashboard metric URL construction and wire decoding.
//! Does not own: shared HTTP transport, report projection, canisters, or network resources.
//! Boundary: fetches one bounded metric response and preserves raw observation values.

use super::{LiveIcSource, append_path_segments, dashboard_base_url, fetch_live};
use crate::ic::{
    IcHostError, IcMetricKind, IcMetricObservation, IcMetricQuery, IcMetricSeries, IcMetricSource,
    IcMetricSourceData, IcSourceRequest, source,
};
use url::Url;

impl IcMetricSource for LiveIcSource {
    fn fetch_metric(
        &self,
        request: &IcSourceRequest,
        query: &IcMetricQuery,
    ) -> Result<IcMetricSourceData, IcHostError> {
        source::validate_metric_query(query)?;
        let url = metric_url(&request.endpoint, query)?;
        let url_text = url.to_string();
        let wire: serde_json::Value = fetch_live(url)?;
        let series =
            decode_metric_series(wire, query.metric).map_err(|reason| IcHostError::JsonDecode {
                url: url_text,
                reason,
            })?;
        Ok(IcMetricSourceData {
            source: request.clone(),
            query: query.clone(),
            series,
        })
    }
}

fn metric_url(endpoint: &str, query: &IcMetricQuery) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &[query.metric.as_str()])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("format", "json");
        pairs.append_pair("start", &query.start_unix_secs.to_string());
        pairs.append_pair("end", &query.end_unix_secs.to_string());
        pairs.append_pair("step", &query.step_secs.to_string());
    }
    Ok(url)
}

fn decode_metric_series(
    value: serde_json::Value,
    metric: IcMetricKind,
) -> Result<Vec<IcMetricSeries>, String> {
    let serde_json::Value::Object(mut object) = value else {
        return Err("expected a JSON object".to_string());
    };
    metric
        .series_names()
        .iter()
        .map(|name| {
            let raw = object
                .remove(*name)
                .ok_or_else(|| format!("missing required series {name:?}"))?;
            let observations: Vec<(u64, String)> = serde_json::from_value(raw)
                .map_err(|error| format!("invalid series {name:?}: {error}"))?;
            Ok(IcMetricSeries {
                name: (*name).to_string(),
                observations: observations
                    .into_iter()
                    .map(|(timestamp_unix_secs, value)| IcMetricObservation {
                        timestamp_unix_secs,
                        value,
                    })
                    .collect(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_url_preserves_explicit_bound_and_step() {
        let query = IcMetricQuery::new(
            IcMetricKind::InstructionRate,
            1_700_000_000,
            1_700_003_600,
            300,
        );
        let url = metric_url("https://metrics-api.internetcomputer.org/api/v1/", &query)
            .expect("metric URL");

        assert_eq!(url.path(), "/api/v1/instruction-rate");
        assert_eq!(
            url.query(),
            Some("format=json&start=1700000000&end=1700003600&step=300")
        );
    }

    #[test]
    fn metric_decoder_preserves_raw_values_and_ignores_additive_fields() {
        let value = serde_json::json!({
            "total_nodes": [[1_700_000_000_u64, "559"]],
            "up_nodes": [[1_700_000_000_u64, "558"]],
            "future_field": true
        });

        let series =
            decode_metric_series(value, IcMetricKind::IcNodeCount).expect("current metric payload");

        assert_eq!(series.len(), 2);
        assert_eq!(series[0].name, "total_nodes");
        assert_eq!(series[0].observations[0].value, "559");
        assert_eq!(series[1].name, "up_nodes");
    }
}
