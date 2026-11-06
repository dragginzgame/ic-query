use crate::nns::inventory::{NnsInventoryInputKind, resolve_nns_inventory_row};

use super::{
    NNS_NODE_INFO_REPORT_SCHEMA_VERSION, NnsInventoryError, NnsInventoryInfoRequest,
    NnsNodeInfoReport, NnsNodeListReport, NnsNodeListRequest, NnsNodeRow,
    filters::filter_node_list_report,
};
use crate::nns::{
    LiveNnsSource, NnsRegistryInventorySource, inventory::load_or_refresh_nns_inventory_report,
};
use crate::{QueryProgress, progress::IgnoreQueryProgress};

pub async fn build_nns_node_list_report(
    request: &NnsNodeListRequest,
) -> Result<NnsNodeListReport, NnsInventoryError> {
    build_nns_node_list_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        &mut IgnoreQueryProgress,
    )
    .await
}

pub async fn build_nns_node_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsNodeInfoReport, NnsInventoryError> {
    build_nns_node_info_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_list_report_with_source_and_progress(
    request: &NnsNodeListRequest,
    source: &dyn NnsRegistryInventorySource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeListReport, NnsInventoryError> {
    let snapshot =
        load_or_refresh_nns_inventory_report(&request.inventory, source, progress).await?;
    let report = NnsNodeListReport {
        schema_version: super::NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
        provenance: snapshot.provenance,
        nodes: snapshot.nodes,
    };
    Ok(filter_node_list_report(report, &request.filters))
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_info_report_with_source_and_progress(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsRegistryInventorySource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeInfoReport, NnsInventoryError> {
    let mut snapshot =
        load_or_refresh_nns_inventory_report(&request.to_list_request(), source, progress).await?;
    let (index, resolved_from) = resolve_node(&snapshot.nodes, &request.input)?;
    let node = snapshot.nodes.swap_remove(index);
    Ok(NnsNodeInfoReport {
        schema_version: NNS_NODE_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        provenance: snapshot.provenance,
        node,
    })
}

pub(super) fn resolve_node(
    rows: &[NnsNodeRow],
    input: &str,
) -> Result<(usize, String), NnsInventoryError> {
    resolve_nns_inventory_row(
        rows,
        |row| row.node_principal.as_str(),
        input,
        NnsInventoryInputKind::Principal,
        "node_principal",
    )
    .map_err(|error| error.for_collection(super::COLLECTION))
}
