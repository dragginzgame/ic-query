use crate::nns::inventory::{NnsInventoryInputKind, resolve_nns_inventory_row};

use super::{
    NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION, NnsDataCenterInfoReport, NnsDataCenterListReport,
    NnsInventoryError, NnsInventoryInfoRequest, NnsInventoryListRequest,
    cache::{load_cached_nns_data_center_report, nns_data_center_cache_path},
    refresh::refresh_nns_data_center_report_with_source,
    source::NnsDataCenterSource,
};
use crate::nns::{LiveNnsSource, inventory::load_or_refresh_nns_inventory_report};
use crate::{QueryProgress, progress::IgnoreQueryProgress};

pub async fn build_nns_data_center_list_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsDataCenterListReport, NnsInventoryError> {
    build_nns_data_center_list_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        &mut IgnoreQueryProgress,
    )
    .await
}

pub async fn build_nns_data_center_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsDataCenterInfoReport, NnsInventoryError> {
    build_nns_data_center_info_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_data_center_list_report_with_source_and_progress(
    request: &NnsInventoryListRequest,
    source: &dyn NnsDataCenterSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsDataCenterListReport, NnsInventoryError> {
    load_or_refresh_nns_inventory_report(
        request,
        nns_data_center_cache_path(&request.cache.cache_root, &request.cache.network),
        progress,
        |cache| load_cached_nns_data_center_report(cache).map(|cached| cached.report),
        async |refresh_request| {
            refresh_nns_data_center_report_with_source(refresh_request, source)
                .await
                .map(|_| ())
        },
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_data_center_info_report_with_source_and_progress(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsDataCenterSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsDataCenterInfoReport, NnsInventoryError> {
    let list_request = request.to_list_request();
    let mut report =
        build_nns_data_center_list_report_with_source_and_progress(&list_request, source, progress)
            .await?;
    let (index, resolved_from) = resolve_data_center(&report, &request.input)?;
    let data_center = report.data_centers.swap_remove(index);
    Ok(NnsDataCenterInfoReport {
        schema_version: NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        provenance: report.provenance,
        data_center,
    })
}

pub(super) fn resolve_data_center(
    report: &NnsDataCenterListReport,
    input: &str,
) -> Result<(usize, String), NnsInventoryError> {
    resolve_nns_inventory_row(
        &report.data_centers,
        |row| row.data_center_id.as_str(),
        input,
        NnsInventoryInputKind::Text,
        "data_center_id",
    )
    .map_err(|error| error.for_collection(super::COLLECTION))
}
