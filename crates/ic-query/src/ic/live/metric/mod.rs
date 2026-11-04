//! Module: ic::live::metric
//!
//! Responsibility: live Dashboard metric URL construction and wire decoding.
//! Does not own: shared HTTP transport, report projection, canisters, or network resources.
//! Boundary: fetches one bounded metric response and preserves raw observation values.

use super::{LiveIcSource, dashboard_resource_url};
use crate::ic::{
    IcHostError, IcMetricKind, IcMetricObservation, IcMetricQuery, IcMetricSeries, IcMetricSource,
    IcMetricSourceData, IcSourceRequest, source,
};
use url::Url;

impl IcMetricSource for LiveIcSource {
    fn fetch_metric<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        query: &'a IcMetricQuery,
    ) -> crate::report::SourceFuture<'a, Result<IcMetricSourceData, IcHostError>> {
        Box::pin(async move {
            source::validate_metric_query(query)?;
            let url = metric_url(&request.endpoint, query)?;
            let url_text = url.to_string();
            let wire: serde_json::Value = self.fetch_live(url).await?;
            let series = decode_metric_series(wire, query.metric).map_err(|reason| {
                IcHostError::JsonDecode {
                    url: url_text,
                    reason,
                }
            })?;
            Ok(IcMetricSourceData {
                source: request.clone(),
                query: query.clone(),
                series,
            })
        })
    }
}

fn metric_url(endpoint: &str, query: &IcMetricQuery) -> Result<Url, IcHostError> {
    let mut url = dashboard_resource_url(endpoint, &[query.metric.as_str()])?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("format", "json");
        super::append_time_series(
            &mut pairs,
            query.start_unix_secs,
            query.end_unix_secs,
            query.step_secs,
        );
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
mod tests;
