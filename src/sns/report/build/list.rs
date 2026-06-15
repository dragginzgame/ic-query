use super::super::{
    SnsHostError, SnsListReport, SnsListRequest,
    assemble::sns_list_report_from_list,
    live::LiveSnsSource,
    lookup::{assign_sns_ids_in_current_order, sns_list_fetch_request, sort_mainnet_sns_instances},
    source::SnsListSource,
};

pub fn build_sns_list_report(request: &SnsListRequest) -> Result<SnsListReport, SnsHostError> {
    build_sns_list_report_with_source(request, &LiveSnsSource)
}

pub(in crate::sns::report) fn build_sns_list_report_with_source(
    request: &SnsListRequest,
    source: &dyn SnsListSource,
) -> Result<SnsListReport, SnsHostError> {
    let fetch_request = sns_list_fetch_request(request)?;
    let mut list = source.fetch_deployed_snses(&fetch_request)?;
    assign_sns_ids_in_current_order(&mut list.sns_instances);
    sort_mainnet_sns_instances(&mut list.sns_instances, request.sort);
    Ok(sns_list_report_from_list(
        list,
        request.verbose,
        request.sort,
    ))
}
