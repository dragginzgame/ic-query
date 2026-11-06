use crate::nns::inventory::{NnsInventoryInputKind, resolve_nns_inventory_row};

use super::{
    NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION, NnsDataCenterInfoReport, NnsDataCenterListReport,
    NnsDataCenterRow, NnsInventoryError, NnsInventoryInfoRequest, NnsInventoryListRequest,
};
use crate::nns::{
    LiveNnsSource, NnsRegistryInventorySource, inventory::load_or_refresh_nns_inventory_report,
};
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
    source: &dyn NnsRegistryInventorySource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsDataCenterListReport, NnsInventoryError> {
    let snapshot = load_or_refresh_nns_inventory_report(request, source, progress).await?;
    let report = NnsDataCenterListReport {
        schema_version: super::NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
        provenance: snapshot.provenance,
        data_centers: snapshot.data_centers,
    };
    Ok(report)
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_data_center_info_report_with_source_and_progress(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsRegistryInventorySource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsDataCenterInfoReport, NnsInventoryError> {
    let mut snapshot =
        load_or_refresh_nns_inventory_report(&request.to_list_request(), source, progress).await?;
    let (index, resolved_from) = resolve_data_center(&snapshot.data_centers, &request.input)?;
    let data_center = snapshot.data_centers.swap_remove(index);
    Ok(NnsDataCenterInfoReport {
        schema_version: NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        provenance: snapshot.provenance,
        data_center,
    })
}

pub(super) fn resolve_data_center(
    rows: &[NnsDataCenterRow],
    input: &str,
) -> Result<(usize, String), NnsInventoryError> {
    resolve_nns_inventory_row(
        rows,
        |row| row.data_center_id.as_str(),
        input,
        NnsInventoryInputKind::Text,
        "data_center_id",
    )
    .map_err(|error| error.for_collection(super::COLLECTION))
}
