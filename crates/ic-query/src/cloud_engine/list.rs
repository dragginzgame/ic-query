//! Module: cloud_engine::list
//!
//! Responsibility: join Registry CloudEngine inventory to exact public operator bindings.
//! Does not own: CLI parsing, cache mechanics, operator detail calls, or text rendering.
//! Boundary: one bounded control-plane lookup is attempted per Registry CloudEngine Subnet;
//! per-row failures remain data while invalid source contracts fail the report.

use super::{
    CLOUD_ENGINE_AUTHORITY, CLOUD_ENGINE_REPORT_SCHEMA_VERSION, CloudEngineHostError,
    CloudEngineListReport, CloudEngineListRow, CloudEngineOperatorBindingSource,
    CloudEngineOperatorBindingSourceData, CloudEngineSourceRequest, LiveCloudEngineSource,
    MAINNET_CLOUD_ENGINE_CANISTER_ID, MAX_CLOUD_ENGINE_LIST_ROWS,
    build::{
        invalid_source, validate_canonical_principal, validate_principal_match,
        validate_source_request,
    },
    enforce_mainnet_network,
};
use crate::nns::LiveNnsSource;
use crate::subnet_catalog::{
    SubnetCatalogFilters, SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogSource,
    SubnetKind, build_subnet_catalog_list_report_with_source_and_progress,
};
use crate::{QueryProgress, progress::IgnoreQueryProgress};
use ic_platform::candid::contract::Principal;

const REGISTRY_AUTHORITY: &str = "nns_registry";
const MAX_LOOKUP_ERROR_BYTES: usize = 4_096;

/// Build a bounded CloudEngine inventory using the Registry catalog and live control plane.
pub async fn build_cloud_engine_list_report(
    catalog_request: &SubnetCatalogListRequest,
    control_plane_request: &CloudEngineSourceRequest,
) -> Result<CloudEngineListReport, CloudEngineHostError> {
    build_cloud_engine_list_report_with_sources(
        catalog_request,
        control_plane_request,
        &LiveNnsSource::default(),
        &LiveCloudEngineSource::default(),
    )
    .await
}

/// Build a CloudEngine inventory while reporting authorized catalog refreshes.
pub async fn build_cloud_engine_list_report_with_progress(
    catalog_request: &SubnetCatalogListRequest,
    control_plane_request: &CloudEngineSourceRequest,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<CloudEngineListReport, CloudEngineHostError> {
    build_cloud_engine_list_report_with_sources_and_progress(
        catalog_request,
        control_plane_request,
        &LiveNnsSource::default(),
        &LiveCloudEngineSource::default(),
        progress,
    )
    .await
}

/// Build a bounded CloudEngine inventory with separate custom sources for both authorities.
pub async fn build_cloud_engine_list_report_with_sources(
    catalog_request: &SubnetCatalogListRequest,
    control_plane_request: &CloudEngineSourceRequest,
    catalog_source: &dyn SubnetCatalogSource,
    binding_source: &dyn CloudEngineOperatorBindingSource,
) -> Result<CloudEngineListReport, CloudEngineHostError> {
    build_cloud_engine_list_report_with_sources_and_progress(
        catalog_request,
        control_plane_request,
        catalog_source,
        binding_source,
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build a CloudEngine inventory with caller-owned sources and progress sink.
pub async fn build_cloud_engine_list_report_with_sources_and_progress(
    catalog_request: &SubnetCatalogListRequest,
    control_plane_request: &CloudEngineSourceRequest,
    catalog_source: &dyn SubnetCatalogSource,
    binding_source: &dyn CloudEngineOperatorBindingSource,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<CloudEngineListReport, CloudEngineHostError> {
    validate_requests(catalog_request, control_plane_request)?;

    let mut inventory_request = catalog_request.clone();
    inventory_request.filters = SubnetCatalogFilters::default().with_kind(SubnetKind::CloudEngine);
    inventory_request.show_ranges = false;
    let catalog = build_subnet_catalog_list_report_with_source_and_progress(
        &inventory_request,
        catalog_source,
        progress,
    )
    .await?;
    build_cloud_engine_list_report_from_catalog_with_source(
        control_plane_request,
        catalog,
        binding_source,
    )
    .await
}

pub(super) async fn build_cloud_engine_list_report_from_catalog_with_source(
    control_plane_request: &CloudEngineSourceRequest,
    catalog: SubnetCatalogListReport,
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

    validate_catalog_rows(&catalog)?;

    let cloud_engines = collect_binding_rows(control_plane_request, &catalog, source).await?;
    Ok(assemble_report(
        control_plane_request,
        catalog,
        cloud_engines,
    ))
}

async fn collect_binding_rows(
    control_plane_request: &CloudEngineSourceRequest,
    catalog: &SubnetCatalogListReport,
    source: &dyn CloudEngineOperatorBindingSource,
) -> Result<Vec<CloudEngineListRow>, CloudEngineHostError> {
    let mut cloud_engines = Vec::with_capacity(catalog.subnets.len());
    for subnet in &catalog.subnets {
        let lookup = source
            .fetch_operator_binding(control_plane_request, subnet.subnet_principal.as_str())
            .await;
        let operator = match lookup {
            Ok(source_data) => Ok(validate_binding_source(
                control_plane_request,
                &subnet.subnet_principal,
                &source_data,
            )?),
            Err(error) => Err(bounded_error_text(&error.to_string())),
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
            operator,
        });
    }
    Ok(cloud_engines)
}

fn assemble_report(
    control_plane_request: &CloudEngineSourceRequest,
    catalog: SubnetCatalogListReport,
    cloud_engines: Vec<CloudEngineListRow>,
) -> CloudEngineListReport {
    let mut operator_binding_count = 0;
    let mut missing_operator_binding_count = 0;
    let mut operator_lookup_failure_count = 0;
    for row in &cloud_engines {
        match &row.operator {
            Ok(Some(_)) => operator_binding_count += 1,
            Ok(None) => missing_operator_binding_count += 1,
            Err(_) => operator_lookup_failure_count += 1,
        }
    }

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
        crate::principal_text::validate_canonical(
            &subnet.subnet_principal,
            |reason| CloudEngineHostError::InvalidPrincipal {
                field: "subnet_id",
                reason,
            },
            |canonical| CloudEngineHostError::InvalidSourceData {
                reason: format!(
                    "catalog subnet principal {:?} is not canonical; expected {canonical:?}",
                    subnet.subnet_principal
                ),
            },
        )?;
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
) -> Result<Option<Principal>, CloudEngineHostError> {
    validate_source_request(request, &source.source)?;
    validate_principal_match("subnet_id", subnet_id, &source.subnet_id)?;
    if source.query_call_count != 1 {
        return invalid_source(format!(
            "an operator-binding lookup requires exactly one query call, got {}",
            source.query_call_count
        ));
    }
    source
        .operator_canister_id
        .as_deref()
        .map(|operator| validate_canonical_principal("operator_canister_id", operator))
        .transpose()
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
