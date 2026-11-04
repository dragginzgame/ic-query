use super::*;

#[test]
fn metric_url_preserves_explicit_bound_and_step() {
    let query = IcMetricQuery::new(
        IcMetricKind::InstructionRate,
        1_700_000_000,
        1_700_003_600,
        300,
    );
    let url =
        metric_url("https://metrics-api.internetcomputer.org/api/v1/", &query).expect("metric URL");

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
