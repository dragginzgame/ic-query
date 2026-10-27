use super::{
    SubnetCatalogInfoReport, SubnetCatalogInfoRequest, load_catalog_for_report,
    rate::{FORMULA_VERSION, catalog_cycles_per_billion, charge_applicability},
};
use crate::{QueryProgress, progress::IgnoreQueryProgress};
use crate::{
    nns::LiveNnsSource,
    subnet_catalog::{
        SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION, SubnetCatalogHostError, SubnetCatalogSource,
    },
};

/// Resolve one subject using the request's explicit cache and live-source policy.
pub async fn build_subnet_catalog_info_report(
    request: &SubnetCatalogInfoRequest,
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    build_subnet_catalog_info_report_with_source(request, &LiveNnsSource::default()).await
}

/// Build the report while reporting authorized live refreshes.
pub async fn build_subnet_catalog_info_report_with_progress(
    request: &SubnetCatalogInfoRequest,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    build_subnet_catalog_info_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        progress,
    )
    .await
}

/// Resolve one subject using the selected cache policy and caller-owned refresh source.
pub async fn build_subnet_catalog_info_report_with_source(
    request: &SubnetCatalogInfoRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    build_subnet_catalog_info_report_with_source_and_progress(
        request,
        source,
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_subnet_catalog_info_report_with_source_and_progress(
    request: &SubnetCatalogInfoRequest,
    source: &dyn SubnetCatalogSource,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    let (cached, stale) = load_catalog_for_report(
        &request.cache,
        &request.read_policy,
        request.now_unix_secs,
        request.stale_after_seconds,
        source,
        progress,
    )
    .await?;
    let resolved = cached
        .catalog
        .raw()
        .resolve_principal_or_prefix(&request.input, request.forced)?;
    let (charges_apply_to_subject, charge_applicability_reason) =
        charge_applicability(resolved.resolved_as, resolved.subnet.subnet_kind);
    let cycles_per_billion_instructions = catalog_cycles_per_billion(&resolved.subnet);
    let rate_source = cycles_per_billion_instructions
        .is_some()
        .then(|| "nns-registry-cache".to_string());
    let formula_version = cycles_per_billion_instructions
        .is_some()
        .then(|| FORMULA_VERSION.to_string());

    Ok(
        super::catalog_report!(SubnetCatalogInfoReport, cached, stale, {
            schema_version: SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION,
            input_principal: resolved.input_principal,
            resolved_as: resolved.resolved_as.as_str().to_string(),
            resolved_from: resolved.resolved_from,
            subnet_principal: resolved.subnet.subnet_principal,
            registry_subnet_type: resolved.subnet.registry_subnet_type,
            subnet_kind: resolved.subnet.subnet_kind,
            subnet_kind_source: resolved.subnet.subnet_kind_source,
            subnet_specialization: resolved.subnet.subnet_specialization,
            subnet_specialization_source: resolved.subnet.subnet_specialization_source,
            geographic_scope: resolved.subnet.geographic_scope,
            geographic_scope_source: resolved.subnet.geographic_scope_source,
            subnet_label: resolved.subnet.subnet_label,
            subnet_label_source: resolved.subnet.subnet_label_source,
            node_count: resolved.subnet.node_count,
            charges_apply_to_subject,
            charge_applicability_reason,
            matched_canister_principal: resolved.matched_canister_principal,
            matched_routing_range: resolved.matched_routing_range,
            cycles_per_billion_instructions,
            rate_source,
            formula_version,
        }),
    )
}
