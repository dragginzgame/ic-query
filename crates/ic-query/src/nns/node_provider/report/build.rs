use crate::nns::inventory::{NnsInventoryInputKind, resolve_nns_inventory_row};

use super::{
    NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION, NnsInventoryError, NnsInventoryInfoRequest,
    NnsInventoryListRequest, NnsNodeProviderInfoReport, NnsNodeProviderListReport,
    source::NnsNodeProviderSource,
};
use crate::nns::{LiveNnsSource, inventory::load_or_refresh_nns_inventory_report};
use crate::{QueryProgress, progress::IgnoreQueryProgress};

pub async fn build_nns_node_provider_list_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsNodeProviderListReport, NnsInventoryError> {
    build_nns_node_provider_list_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        &mut IgnoreQueryProgress,
    )
    .await
}

pub async fn build_nns_node_provider_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsNodeProviderInfoReport, NnsInventoryError> {
    build_nns_node_provider_info_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_provider_list_report_with_source_and_progress(
    request: &NnsInventoryListRequest,
    source: &dyn NnsNodeProviderSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeProviderListReport, NnsInventoryError> {
    let snapshot = load_or_refresh_nns_inventory_report(request, source, progress).await?;
    let metadata =
        super::cache::load_or_refresh_metadata(request, source, &snapshot, progress).await?;
    Ok(node_provider_report_from_snapshot(&snapshot, metadata))
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_nns_node_provider_info_report_with_source_and_progress(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsNodeProviderSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsNodeProviderInfoReport, NnsInventoryError> {
    let list_request = request.to_list_request();
    let mut report = build_nns_node_provider_list_report_with_source_and_progress(
        &list_request,
        source,
        progress,
    )
    .await?;
    let (index, resolved_from) = resolve_node_provider(&report, &request.input)?;
    let provider = report.node_providers.swap_remove(index);
    Ok(NnsNodeProviderInfoReport {
        schema_version: NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        provenance: report.provenance,
        governance_canister_id: report.governance_canister_id,
        governance_source: report.governance_source,
        node_provider: provider,
    })
}

pub(super) fn resolve_node_provider(
    report: &NnsNodeProviderListReport,
    input: &str,
) -> Result<(usize, String), NnsInventoryError> {
    resolve_nns_inventory_row(
        &report.node_providers,
        |row| row.node_provider_principal.as_str(),
        input,
        NnsInventoryInputKind::Principal,
        "node_provider_principal",
    )
    .map_err(|error| error.for_collection(super::COLLECTION))
}

pub(in crate::nns) fn node_provider_report_from_snapshot(
    snapshot: &crate::nns::NnsRegistryInventorySnapshot,
    metadata: super::NnsNodeProviderMetadata,
) -> NnsNodeProviderListReport {
    let mut counts = std::collections::BTreeMap::<&str, u32>::new();
    for node in &snapshot.nodes {
        let count = counts
            .entry(node.node_provider_principal.as_str())
            .or_default();
        *count = count.saturating_add(1);
    }
    let mut node_providers = metadata.node_providers;
    for provider in &mut node_providers {
        provider.node_count = Some(
            counts
                .get(provider.node_provider_principal.as_str())
                .copied()
                .unwrap_or(0),
        );
    }
    NnsNodeProviderListReport {
        schema_version: super::NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
        provenance: snapshot.provenance.clone(),
        governance_canister_id: metadata.governance_canister_id,
        governance_source: metadata.source,
        node_providers,
    }
}
