//! Module: sns::report::build::params
//!
//! Responsibility: build SNS governance parameter reports.
//! Does not own: command parsing, governance transport internals, DTO assembly, or rendering.
//! Boundary: resolves SNS identity, fetches parameters, and delegates report assembly.

use crate::sns::report::{
    SnsHostError, SnsLookupRequest, SnsParamsReport, assemble::sns_params_report_from_parts,
    live::LiveSnsSource, lookup::resolve_sns_lookup, source::SnsParamsSource,
};

pub fn build_sns_params_report(
    request: &SnsLookupRequest,
) -> Result<SnsParamsReport, SnsHostError> {
    build_sns_params_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_params_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsParamsSource,
) -> Result<SnsParamsReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    let parameters = source.fetch_sns_params(&lookup.fetch_request, &lookup.sns)?;
    Ok(sns_params_report_from_parts(
        lookup.list,
        lookup.id,
        lookup.sns,
        parameters,
    ))
}
