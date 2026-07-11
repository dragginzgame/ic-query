//! Module: sns::report::build::info
//!
//! Responsibility: build one deployed SNS info report.
//! Does not own: command parsing, SNS-W transport internals, report DTO mapping, or rendering.
//! Boundary: resolves lookup input through a source and delegates DTO assembly.

use crate::sns::report::{
    SnsHostError, SnsInfoReport, SnsLookupRequest, assemble::sns_info_report_from_list,
    live::LiveSnsSource, lookup::resolve_sns_lookup, source::SnsListSource,
};

pub fn build_sns_info_report(request: &SnsLookupRequest) -> Result<SnsInfoReport, SnsHostError> {
    build_sns_info_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_info_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsListSource,
) -> Result<SnsInfoReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    Ok(sns_info_report_from_list(
        lookup.list,
        lookup.id,
        lookup.sns,
    ))
}
