use crate::nns::inventory::{NnsInventoryInputKind, resolve_nns_inventory_row};

use super::{
    NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION, NnsInventoryError, NnsInventoryInfoRequest,
    NnsInventoryListRequest, NnsNodeOperatorInfoReport, NnsNodeOperatorListReport,
    cache::{load_cached_nns_node_operator_report, nns_node_operator_cache_path},
    refresh::refresh_nns_node_operator_report_with_source,
    source::NnsNodeOperatorSource,
};
use crate::nns::{LiveNnsSource, inventory::load_or_refresh_nns_inventory_report};
use crate::{QueryProgress, progress::IgnoreQueryProgress};

pub async fn build_nns_node_operator_list_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsNodeOperatorListReport, NnsInventoryError> {
    build_nns_node_operator_list_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        &mut IgnoreQueryProgress,
    )
    .await
}

pub async fn build_nns_node_operator_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsNodeOperatorInfoReport, NnsInventoryError> {
    build_nns_node_operator_info_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_operator_list_report_with_source_and_progress(
    request: &NnsInventoryListRequest,
    source: &dyn NnsNodeOperatorSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeOperatorListReport, NnsInventoryError> {
    load_or_refresh_nns_inventory_report(
        request,
        nns_node_operator_cache_path(&request.cache.cache_root, &request.cache.network),
        progress,
        |cache| load_cached_nns_node_operator_report(cache).map(|cached| cached.report),
        async |refresh_request| {
            refresh_nns_node_operator_report_with_source(refresh_request, source)
                .await
                .map(|_| ())
        },
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_operator_info_report_with_source_and_progress(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsNodeOperatorSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeOperatorInfoReport, NnsInventoryError> {
    let list_request = request.to_list_request();
    let mut report = build_nns_node_operator_list_report_with_source_and_progress(
        &list_request,
        source,
        progress,
    )
    .await?;
    let (index, resolved_from) = resolve_node_operator(&report, &request.input)?;
    let operator = report.node_operators.swap_remove(index);
    Ok(NnsNodeOperatorInfoReport {
        schema_version: NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        provenance: report.provenance,
        node_operator: operator,
    })
}

pub(super) fn resolve_node_operator(
    report: &NnsNodeOperatorListReport,
    input: &str,
) -> Result<(usize, String), NnsInventoryError> {
    resolve_nns_inventory_row(
        &report.node_operators,
        |row| row.node_operator_principal.as_str(),
        input,
        NnsInventoryInputKind::Principal,
        "node_operator_principal",
    )
    .map_err(|error| error.for_collection(super::COLLECTION))
}
