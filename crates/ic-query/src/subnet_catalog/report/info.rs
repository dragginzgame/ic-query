use super::{
    SubnetCatalogInfoReport, SubnetCatalogInfoRequest,
    rate::{FORMULA_VERSION, catalog_cycles_per_billion, charge_applicability},
};
use crate::subnet_catalog::{
    LiveNnsRegistryRefreshSource, SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION,
    SubnetCatalogHostError, SubnetCatalogRefreshSource, catalog_stale_status,
    load_or_refresh_subnet_catalog,
};

pub fn build_subnet_catalog_info_report(
    request: &SubnetCatalogInfoRequest,
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    build_subnet_catalog_info_report_with_source(request, &LiveNnsRegistryRefreshSource)
}

fn build_subnet_catalog_info_report_with_source(
    request: &SubnetCatalogInfoRequest,
    source: &dyn SubnetCatalogRefreshSource,
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    let cached = load_or_refresh_subnet_catalog(
        &request.cache,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let stale = catalog_stale_status(
        &cached.catalog,
        request.now_unix_secs,
        request.stale_after_seconds,
    );
    let resolved = cached
        .catalog
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

    Ok(SubnetCatalogInfoReport {
        schema_version: SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION,
        input_principal: resolved.input_principal,
        resolved_as: resolved.resolved_as.as_str().to_string(),
        resolved_from: resolved.resolved_from,
        subnet_principal: resolved.subnet.subnet_principal,
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
        registry_canister_id: cached.catalog.registry_canister_id,
        registry_version: cached.catalog.registry_version,
        catalog_schema_version: cached.catalog.catalog_schema_version,
        catalog_path: cached.path.display().to_string(),
        fetched_at: cached.catalog.fetched_at,
        catalog_stale: stale.catalog_stale,
        stale_reason: stale.stale_reason,
        resolver_backend: cached.catalog.resolver_backend,
        matched_canister_principal: resolved.matched_canister_principal,
        matched_routing_range: resolved.matched_routing_range,
        cycles_per_billion_instructions,
        rate_source,
        formula_version,
    })
}
