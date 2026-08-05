use super::{
    SubnetCatalogInfoReport, SubnetCatalogInfoRequest,
    rate::{FORMULA_VERSION, catalog_cycles_per_billion, charge_applicability},
};
use crate::{
    nns::LiveNnsSource,
    subnet_catalog::{
        SUBNET_CATALOG_INFO_REPORT_SCHEMA_VERSION, SubnetCatalogHostError,
        SubnetCatalogLoadRequest, SubnetCatalogSource, catalog_stale_status,
        load_subnet_catalog_with_source,
    },
};

pub fn build_subnet_catalog_info_report(
    request: &SubnetCatalogInfoRequest,
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    build_subnet_catalog_info_report_with_source(request, &LiveNnsSource)
}

pub fn build_subnet_catalog_info_report_with_source(
    request: &SubnetCatalogInfoRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<SubnetCatalogInfoReport, SubnetCatalogHostError> {
    let load_request =
        SubnetCatalogLoadRequest::cache_only(request.cache.clone(), request.now_unix_secs)
            .with_policy(request.read_policy.clone());
    let cached = load_subnet_catalog_with_source(&load_request, source)?;
    let stale = catalog_stale_status(
        cached.catalog.raw(),
        request.now_unix_secs,
        request.stale_after_seconds,
    );
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

    Ok(SubnetCatalogInfoReport {
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
        registry_canister_id: cached.catalog.provenance().registry_canister_id.clone(),
        registry_version: cached.catalog.provenance().registry_version,
        assurance: cached.catalog.provenance().assurance,
        source_endpoints: cached.catalog.provenance().source_endpoints.clone(),
        catalog_digest: cached.catalog.raw().catalog_digest.clone(),
        cache_disposition: cached.disposition,
        catalog_schema_version: cached.catalog.raw().catalog_schema_version,
        catalog_path: cached.path.display().to_string(),
        fetched_at: cached.catalog.provenance().fetched_at.clone(),
        catalog_stale: stale.catalog_stale,
        stale_reason: stale.stale_reason,
        resolver_backend: cached.catalog.provenance().resolver_backend.clone(),
        collector_version: cached.catalog.provenance().collector_version.clone(),
        classification_schema_version: cached.catalog.provenance().classification_schema_version,
        classification_policy_digest: cached
            .catalog
            .provenance()
            .classification_policy_digest
            .clone(),
        resolver_schema_version: cached.catalog.provenance().resolver_schema_version,
        matched_canister_principal: resolved.matched_canister_principal,
        matched_routing_range: resolved.matched_routing_range,
        cycles_per_billion_instructions,
        rate_source,
        formula_version,
    })
}
