use super::super::{
    SnsHostError, SnsTokenReport, SnsTokenRequest, assemble::sns_token_report_from_parts,
    live::LiveSnsSource, lookup::resolve_sns_lookup, source::SnsTokenSource,
};

pub fn build_sns_token_report(request: &SnsTokenRequest) -> Result<SnsTokenReport, SnsHostError> {
    build_sns_token_report_with_source(request, &LiveSnsSource)
}

pub(in crate::sns::report) fn build_sns_token_report_with_source(
    request: &SnsTokenRequest,
    source: &dyn SnsTokenSource,
) -> Result<SnsTokenReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    let token = source.fetch_sns_token(&lookup.fetch_request, &lookup.sns)?;
    Ok(sns_token_report_from_parts(
        lookup.list,
        lookup.id,
        lookup.sns,
        token,
    ))
}
