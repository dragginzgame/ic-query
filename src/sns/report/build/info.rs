use super::super::{
    SnsHostError, SnsInfoReport, SnsInfoRequest, assemble::sns_info_report_from_list,
    live::LiveSnsSource, lookup::resolve_sns_lookup, source::SnsListSource,
};

pub fn build_sns_info_report(request: &SnsInfoRequest) -> Result<SnsInfoReport, SnsHostError> {
    build_sns_info_report_with_source(request, &LiveSnsSource)
}

pub(in crate::sns::report) fn build_sns_info_report_with_source(
    request: &SnsInfoRequest,
    source: &dyn SnsListSource,
) -> Result<SnsInfoReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    Ok(sns_info_report_from_list(
        lookup.list,
        lookup.id,
        lookup.sns,
    ))
}
