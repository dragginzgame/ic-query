//! Module: sns::report::build::list
//!
//! Responsibility: build deployed SNS list reports.
//! Does not own: command parsing, SNS-W transport internals, report DTO mapping, or rendering.
//! Boundary: fetches source rows, assigns stable ids, applies view sorting, and assembles output.

use crate::sns::report::{
    SnsHostError, SnsListReport, SnsListRequest,
    assemble::{SnsReportProvenance, sns_list_report_from_list},
    live::LiveSnsSource,
    lookup::sns_list_fetch_request,
    source::{
        SnsCatalogSource, join_mainnet_sns_inventory, join_mainnet_sns_lifecycles,
        validate_mainnet_sns_inventory,
    },
};

pub async fn build_sns_list_report(
    request: &SnsListRequest,
) -> Result<SnsListReport, SnsHostError> {
    build_sns_list_report_with_source(request, &LiveSnsSource::default()).await
}

pub async fn build_sns_list_report_with_source(
    request: &SnsListRequest,
    source: &dyn SnsCatalogSource,
) -> Result<SnsListReport, SnsHostError> {
    let list = fetch_joined_sns_catalog(request, source).await?;
    Ok(sns_list_report_from_list(
        list,
        request,
        SnsReportProvenance::live(),
    ))
}

pub(in crate::sns::report) async fn fetch_joined_sns_catalog(
    request: &SnsListRequest,
    source: &dyn SnsCatalogSource,
) -> Result<crate::sns::report::JoinedMainnetSnsInventory, SnsHostError> {
    let fetch_request = sns_list_fetch_request(request)?;
    let inventory = source.fetch_sns_inventory(&fetch_request).await?;
    validate_mainnet_sns_inventory(&fetch_request, &inventory)?;
    let metadata = source
        .fetch_sns_metadata(&fetch_request, &inventory.sns_instances)
        .await?;
    let lifecycles = source
        .fetch_sns_lifecycles(&fetch_request, &inventory.sns_instances)
        .await?;
    let mut list = join_mainnet_sns_inventory(inventory, metadata)?;
    join_mainnet_sns_lifecycles(&mut list, lifecycles)?;
    for (index, sns) in list.sns_instances.iter_mut().enumerate() {
        sns.id = index + 1;
    }
    Ok(list)
}
