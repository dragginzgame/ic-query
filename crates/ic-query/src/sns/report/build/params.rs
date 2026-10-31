//! Module: sns::report::build::params
//!
//! Responsibility: build SNS governance parameter reports.
//! Does not own: command parsing, governance transport internals, or rendering.
//! Boundary: resolves SNS identity, fetches parameters, and assembles the report.

use crate::sns::report::SnsReportContext;
use crate::sns::report::{
    SNS_PARAMS_REPORT_SCHEMA_VERSION, SnsHostError, SnsLookupRequest, SnsParamsReport,
    live::LiveSnsSource, lookup::resolve_sns_lookup, source::SnsParamsSource,
};

pub async fn build_sns_params_report(
    request: &SnsLookupRequest,
) -> Result<SnsParamsReport, SnsHostError> {
    build_sns_params_report_with_source(request, &LiveSnsSource::default()).await
}

pub async fn build_sns_params_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsParamsSource,
) -> Result<SnsParamsReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source).await?;
    let parameters = source
        .fetch_sns_params(&lookup.fetch_request, &lookup.sns)
        .await?;
    Ok(SnsParamsReport {
        schema_version: SNS_PARAMS_REPORT_SCHEMA_VERSION,
        context: SnsReportContext::from_live(lookup.fetch_request, lookup.sns),
        parameters,
    })
}
