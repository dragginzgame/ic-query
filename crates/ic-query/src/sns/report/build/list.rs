//! Module: sns::report::build::list
//!
//! Responsibility: build deployed SNS list reports.
//! Does not own: command parsing, SNS-W transport internals, report DTO mapping, or rendering.
//! Boundary: fetches source rows, assigns stable ids, applies view sorting, and assembles output.

use crate::sns::report::{
    SnsHostError, SnsListReport, SnsListRequest,
    assemble::{SnsReportProvenance, sns_list_report_from_list},
    live::LiveSnsSource,
    lookup::{assign_sns_ids_in_current_order, sns_list_fetch_request},
    source::{
        SnsCatalogSource, join_mainnet_sns_inventory, join_mainnet_sns_lifecycles,
        validate_joined_mainnet_sns_catalog, validate_mainnet_sns_inventory,
    },
    view::{filter_mainnet_sns_instances, sort_mainnet_sns_instances},
};

pub fn build_sns_list_report(request: &SnsListRequest) -> Result<SnsListReport, SnsHostError> {
    build_sns_list_report_with_source(request, &LiveSnsSource)
}

pub fn build_sns_list_report_with_source(
    request: &SnsListRequest,
    source: &dyn SnsCatalogSource,
) -> Result<SnsListReport, SnsHostError> {
    let mut list = fetch_joined_sns_catalog(request, source)?;
    let catalog_sns_count = list.sns_instances.len();
    filter_mainnet_sns_instances(&mut list.sns_instances, request.all_lifecycles);
    sort_mainnet_sns_instances(&mut list.sns_instances, request.sort);
    Ok(sns_list_report_from_list(
        list,
        catalog_sns_count,
        request.all_lifecycles,
        request.verbose,
        request.sort,
        SnsReportProvenance::live(),
    ))
}

pub(in crate::sns::report) fn fetch_joined_sns_catalog(
    request: &SnsListRequest,
    source: &dyn SnsCatalogSource,
) -> Result<crate::sns::report::JoinedMainnetSnsInventory, SnsHostError> {
    let fetch_request = sns_list_fetch_request(request)?;
    let inventory = source.fetch_sns_inventory(&fetch_request)?;
    validate_mainnet_sns_inventory(&fetch_request, &inventory)?;
    let metadata = source.fetch_sns_metadata(&fetch_request, &inventory.sns_instances)?;
    let lifecycles = source.fetch_sns_lifecycles(&fetch_request, &inventory.sns_instances)?;
    let mut list = join_mainnet_sns_inventory(inventory, metadata)?;
    join_mainnet_sns_lifecycles(&mut list, lifecycles)?;
    assign_sns_ids_in_current_order(&mut list.sns_instances);
    validate_joined_mainnet_sns_catalog(&list)?;
    Ok(list)
}
