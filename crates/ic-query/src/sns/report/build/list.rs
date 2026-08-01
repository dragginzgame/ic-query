//! Module: sns::report::build::list
//!
//! Responsibility: build deployed SNS list reports.
//! Does not own: command parsing, SNS-W transport internals, report DTO mapping, or rendering.
//! Boundary: fetches source rows, assigns stable ids, applies view sorting, and assembles output.

use crate::sns::report::{
    SnsHostError, SnsListReport, SnsListRequest,
    assemble::sns_list_report_from_list,
    live::LiveSnsSource,
    lookup::{assign_sns_ids_in_current_order, sns_list_fetch_request},
    source::{SnsDiscoverySource, join_mainnet_sns_inventory, validate_mainnet_sns_inventory},
    view::sort_mainnet_sns_instances,
};

pub fn build_sns_list_report(request: &SnsListRequest) -> Result<SnsListReport, SnsHostError> {
    build_sns_list_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_list_report_with_source(
    request: &SnsListRequest,
    source: &dyn SnsDiscoverySource,
) -> Result<SnsListReport, SnsHostError> {
    let fetch_request = sns_list_fetch_request(request)?;
    let inventory = source.fetch_sns_inventory(&fetch_request)?;
    validate_mainnet_sns_inventory(&fetch_request, &inventory)?;
    let metadata = source.fetch_sns_metadata(&fetch_request, &inventory.sns_instances)?;
    let mut list = join_mainnet_sns_inventory(inventory, metadata)?;
    assign_sns_ids_in_current_order(&mut list.sns_instances);
    sort_mainnet_sns_instances(&mut list.sns_instances, request.sort);
    Ok(sns_list_report_from_list(
        list,
        request.verbose,
        request.sort,
    ))
}
