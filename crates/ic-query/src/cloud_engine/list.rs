//! Module: cloud_engine::list
//!
//! Responsibility: join Registry CloudEngine inventory to exact public operator bindings.
//! Does not own: CLI parsing, cache mechanics, operator detail calls, or text rendering.
//! Boundary: one bounded control-plane lookup is attempted per Registry CloudEngine Subnet;
//! per-row failures remain data while invalid source contracts fail the report.

use super::{
    CLOUD_ENGINE_AUTHORITY, CLOUD_ENGINE_REPORT_SCHEMA_VERSION, CloudEngineHostError,
    CloudEngineListReport, CloudEngineListRow, CloudEngineOperatorBindingSource,
    CloudEngineOperatorBindingSourceData, CloudEngineOperatorLookupStatus,
    CloudEngineSourceRequest, LiveCloudEngineSource, MAINNET_CLOUD_ENGINE_CANISTER_ID,
    MAX_CLOUD_ENGINE_LIST_ROWS,
    build::{
        canonical_principal, invalid_source, validate_canonical_principal,
        validate_principal_match, validate_source_request,
    },
    enforce_mainnet_network,
};
use crate::nns::LiveNnsSource;
use crate::subnet_catalog::{
    SubnetCatalogFilters, SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogSource,
    SubnetKind, build_subnet_catalog_list_report_with_source,
};

const REGISTRY_AUTHORITY: &str = "nns_registry";
const MAX_LOOKUP_ERROR_BYTES: usize = 4_096;

/// Build a bounded CloudEngine inventory using the Registry catalog and live control plane.
pub fn build_cloud_engine_list_report(
    catalog_request: &SubnetCatalogListRequest,
    control_plane_request: &CloudEngineSourceRequest,
) -> Result<CloudEngineListReport, CloudEngineHostError> {
    build_cloud_engine_list_report_with_sources(
        catalog_request,
        control_plane_request,
        &LiveNnsSource,
        &LiveCloudEngineSource,
    )
}

/// Build a bounded CloudEngine inventory with separate custom sources for both authorities.
pub fn build_cloud_engine_list_report_with_sources(
    catalog_request: &SubnetCatalogListRequest,
    control_plane_request: &CloudEngineSourceRequest,
    catalog_source: &dyn SubnetCatalogSource,
    binding_source: &dyn CloudEngineOperatorBindingSource,
) -> Result<CloudEngineListReport, CloudEngineHostError> {
    validate_requests(catalog_request, control_plane_request)?;

    let mut inventory_request = catalog_request.clone();
    inventory_request.filters = SubnetCatalogFilters::default().with_kind(SubnetKind::CloudEngine);
    inventory_request.show_ranges = false;
    let catalog = build_subnet_catalog_list_report_with_source(&inventory_request, catalog_source)?;
    build_cloud_engine_list_report_from_catalog_with_source(
        control_plane_request,
        catalog,
        binding_source,
    )
}

pub(super) fn build_cloud_engine_list_report_from_catalog_with_source(
    control_plane_request: &CloudEngineSourceRequest,
    mut catalog: SubnetCatalogListReport,
    source: &dyn CloudEngineOperatorBindingSource,
) -> Result<CloudEngineListReport, CloudEngineHostError> {
    enforce_mainnet_network(&control_plane_request.network)?;
    if catalog.network != control_plane_request.network {
        return invalid_source(format!(
            "catalog network {:?} does not match control-plane network {:?}",
            catalog.network, control_plane_request.network
        ));
    }
    if catalog.subnets.len() > MAX_CLOUD_ENGINE_LIST_ROWS {
        return invalid_source(format!(
            "Registry catalog returned {} CloudEngine Subnets, maximum is {MAX_CLOUD_ENGINE_LIST_ROWS}",
            catalog.subnets.len()
        ));
    }

    catalog
        .subnets
        .sort_unstable_by(|left, right| left.subnet_principal.cmp(&right.subnet_principal));
    validate_catalog_rows(&catalog)?;

    let cloud_engines = collect_binding_rows(control_plane_request, &catalog, source)?;
    Ok(assemble_report(
        control_plane_request,
        catalog,
        cloud_engines,
    ))
}

fn collect_binding_rows(
    control_plane_request: &CloudEngineSourceRequest,
    catalog: &SubnetCatalogListReport,
    source: &dyn CloudEngineOperatorBindingSource,
) -> Result<Vec<CloudEngineListRow>, CloudEngineHostError> {
    let mut cloud_engines = Vec::with_capacity(catalog.subnets.len());
    for subnet in &catalog.subnets {
        let lookup =
            source.fetch_operator_binding(control_plane_request, subnet.subnet_principal.as_str());
        let (operator_lookup_status, operator_canister_id, operator_lookup_error) = match lookup {
            Ok(source_data) => {
                validate_binding_source(
                    control_plane_request,
                    &subnet.subnet_principal,
                    &source_data,
                )?;
                if source_data.operator_canister_id.is_some() {
                    (
                        CloudEngineOperatorLookupStatus::Resolved,
                        source_data.operator_canister_id,
                        None,
                    )
                } else {
                    (CloudEngineOperatorLookupStatus::Absent, None, None)
                }
            }
            Err(error) => (
                CloudEngineOperatorLookupStatus::Failed,
                None,
                Some(bounded_error_text(&error.to_string())),
            ),
        };

        cloud_engines.push(CloudEngineListRow {
            subnet_id: subnet.subnet_principal.clone(),
            subnet_label: subnet.subnet_label.clone(),
            subnet_label_source: subnet.subnet_label_source,
            registry_subnet_type: subnet.registry_subnet_type,
            subnet_kind: subnet.subnet_kind,
            subnet_kind_source: subnet.subnet_kind_source,
            subnet_specialization: subnet.subnet_specialization,
            subnet_specialization_source: subnet.subnet_specialization_source,
            geographic_scope: subnet.geographic_scope,
            geographic_scope_source: subnet.geographic_scope_source,
            node_count: subnet.node_count,
            charges_apply_by_default: subnet.charges_apply_by_default,
            range_count: subnet.range_count,
            operator_lookup_status,
            operator_canister_id,
            operator_lookup_error,
        });
    }
    Ok(cloud_engines)
}

fn assemble_report(
    control_plane_request: &CloudEngineSourceRequest,
    catalog: SubnetCatalogListReport,
    cloud_engines: Vec<CloudEngineListRow>,
) -> CloudEngineListReport {
    let operator_binding_count =
        count_status(&cloud_engines, CloudEngineOperatorLookupStatus::Resolved);
    let missing_operator_binding_count =
        count_status(&cloud_engines, CloudEngineOperatorLookupStatus::Absent);
    let operator_lookup_failure_count =
        count_status(&cloud_engines, CloudEngineOperatorLookupStatus::Failed);

    CloudEngineListReport {
        schema_version: CLOUD_ENGINE_REPORT_SCHEMA_VERSION,
        network: control_plane_request.network.clone(),
        registry_authority: REGISTRY_AUTHORITY.to_string(),
        registry_canister_id: catalog.registry_canister_id,
        registry_version: catalog.registry_version,
        registry_assurance: catalog.assurance,
        registry_source_endpoints: catalog.source_endpoints,
        registry_agreement_digest: catalog.agreement_digest,
        registry_query_call_count: catalog.registry_query_call_count,
        catalog_path: catalog.catalog_path,
        catalog_schema_version: catalog.catalog_schema_version,
        catalog_digest: catalog.catalog_digest,
        catalog_cache_disposition: catalog.cache_disposition,
        catalog_fetched_at: catalog.fetched_at,
        catalog_stale: catalog.catalog_stale,
        catalog_stale_reason: catalog.stale_reason,
        catalog_collector_version: catalog.collector_version,
        classification_schema_version: catalog.classification_schema_version,
        classification_policy_digest: catalog.classification_policy_digest,
        resolver_backend: catalog.resolver_backend,
        resolver_schema_version: catalog.resolver_schema_version,
        control_plane_authority: CLOUD_ENGINE_AUTHORITY.to_string(),
        control_plane_canister_id: MAINNET_CLOUD_ENGINE_CANISTER_ID.to_string(),
        control_plane_source_endpoint: control_plane_request.endpoint.clone(),
        control_plane_fetched_at: control_plane_request.fetched_at.clone(),
        control_plane_fetched_by: control_plane_request.fetched_by.clone(),
        control_plane_certified: false,
        control_plane_point_in_time_guaranteed: false,
        control_plane_lookup_attempt_count: cloud_engines.len(),
        registry_cloud_engine_subnet_count: cloud_engines.len(),
        operator_binding_count,
        missing_operator_binding_count,
        operator_lookup_failure_count,
        cloud_engines,
    }
}

fn validate_requests(
    catalog_request: &SubnetCatalogListRequest,
    control_plane_request: &CloudEngineSourceRequest,
) -> Result<(), CloudEngineHostError> {
    enforce_mainnet_network(&control_plane_request.network)?;
    if catalog_request.cache.network != control_plane_request.network {
        return invalid_source(format!(
            "catalog request network {:?} does not match control-plane network {:?}",
            catalog_request.cache.network, control_plane_request.network
        ));
    }
    Ok(())
}

fn validate_catalog_rows(catalog: &SubnetCatalogListReport) -> Result<(), CloudEngineHostError> {
    for subnet in &catalog.subnets {
        if subnet.subnet_kind != SubnetKind::CloudEngine {
            return invalid_source(format!(
                "catalog row {} has kind {}, expected cloud_engine",
                subnet.subnet_principal,
                subnet.subnet_kind.as_str()
            ));
        }
        let canonical = canonical_principal("subnet_id", &subnet.subnet_principal)?;
        if canonical != subnet.subnet_principal {
            return invalid_source(format!(
                "catalog subnet principal {:?} is not canonical; expected {canonical:?}",
                subnet.subnet_principal
            ));
        }
    }
    if catalog
        .subnets
        .windows(2)
        .any(|pair| pair[0].subnet_principal == pair[1].subnet_principal)
    {
        return invalid_source("Registry CloudEngine Subnet principals must be unique");
    }
    Ok(())
}

fn validate_binding_source(
    request: &CloudEngineSourceRequest,
    subnet_id: &str,
    source: &CloudEngineOperatorBindingSourceData,
) -> Result<(), CloudEngineHostError> {
    validate_source_request(request, &source.source)?;
    validate_principal_match("subnet_id", subnet_id, &source.subnet_id)?;
    if source.query_call_count != 1 {
        return invalid_source(format!(
            "an operator-binding lookup requires exactly one query call, got {}",
            source.query_call_count
        ));
    }
    if let Some(operator) = source.operator_canister_id.as_deref() {
        validate_canonical_principal("operator_canister_id", operator)?;
    }
    Ok(())
}

fn count_status(rows: &[CloudEngineListRow], status: CloudEngineOperatorLookupStatus) -> usize {
    rows.iter()
        .filter(|row| row.operator_lookup_status == status)
        .count()
}

fn bounded_error_text(error: &str) -> String {
    if error.len() <= MAX_LOOKUP_ERROR_BYTES {
        return error.to_string();
    }
    let mut boundary = MAX_LOOKUP_ERROR_BYTES;
    while !error.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{}...", &error[..boundary])
}
