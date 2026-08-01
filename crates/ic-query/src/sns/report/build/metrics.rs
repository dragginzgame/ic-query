//! Module: sns::report::build::metrics
//!
//! Responsibility: build bounded SNS Governance metrics reports.
//! Does not own: command parsing, transport internals, DTO assembly, or rendering.
//! Boundary: validates the request, resolves one SNS, and validates source evidence.

use crate::sns::report::{
    SnsHostError, SnsMetricsReport, SnsMetricsRequest, SnsMetricsSource,
    assemble::sns_metrics_report_from_parts,
    live::LiveSnsSource,
    lookup::resolve_sns_lookup,
    model::{sns_metrics_lookup_request, validate_sns_metrics_request},
    source::canonicalize_mainnet_sns_metrics,
};

/// Build a live bounded Governance metrics report for one deployed SNS.
pub fn build_sns_metrics_report(
    request: &SnsMetricsRequest,
) -> Result<SnsMetricsReport, SnsHostError> {
    build_sns_metrics_report_with_source(request, &LiveSnsSource)
}

/// Build a bounded Governance metrics report through a custom source capability.
pub fn build_sns_metrics_report_with_source(
    request: &SnsMetricsRequest,
    source: &dyn SnsMetricsSource,
) -> Result<SnsMetricsReport, SnsHostError> {
    validate_sns_metrics_request(request)?;
    let lookup = resolve_sns_lookup(&sns_metrics_lookup_request(request), source)?;
    let mut metrics = source.fetch_sns_metrics(
        &lookup.fetch_request,
        &lookup.sns,
        request.time_window_seconds,
    )?;
    canonicalize_mainnet_sns_metrics(
        &mut metrics,
        &lookup.sns.governance_canister_id,
        request.time_window_seconds,
    )?;
    Ok(sns_metrics_report_from_parts(
        lookup.list,
        lookup.id,
        lookup.sns,
        metrics,
    ))
}
