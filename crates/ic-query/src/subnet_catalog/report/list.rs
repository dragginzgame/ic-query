use super::{
    SubnetCatalogFilters, SubnetCatalogListReport, SubnetCatalogListRequest, SubnetCatalogSubnetRow,
};
use crate::subnet_catalog::{
    LiveNnsRegistryRefreshSource, SUBNET_CATALOG_LIST_REPORT_SCHEMA_VERSION, SubnetCatalog,
    SubnetCatalogHostError, SubnetCatalogSource, SubnetInfo, catalog_stale_status,
    load_or_refresh_subnet_catalog_with_source,
};

pub fn build_subnet_catalog_list_report(
    request: &SubnetCatalogListRequest,
) -> Result<SubnetCatalogListReport, SubnetCatalogHostError> {
    build_subnet_catalog_list_report_with_source(request, &LiveNnsRegistryRefreshSource)
}

pub fn build_subnet_catalog_list_report_with_source(
    request: &SubnetCatalogListRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<SubnetCatalogListReport, SubnetCatalogHostError> {
    let cached = load_or_refresh_subnet_catalog_with_source(
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
    let subnets = cached
        .catalog
        .subnets
        .iter()
        .filter(|subnet| subnet_matches_filters(subnet, request.filters))
        .map(|subnet| subnet_row(&cached.catalog, subnet, request))
        .collect::<Vec<_>>();

    Ok(SubnetCatalogListReport {
        schema_version: SUBNET_CATALOG_LIST_REPORT_SCHEMA_VERSION,
        network: cached.catalog.network,
        catalog_path: cached.path.display().to_string(),
        catalog_schema_version: cached.catalog.catalog_schema_version,
        registry_canister_id: cached.catalog.registry_canister_id,
        registry_version: cached.catalog.registry_version,
        fetched_at: cached.catalog.fetched_at,
        catalog_stale: stale.catalog_stale,
        stale_reason: stale.stale_reason,
        resolver_backend: cached.catalog.resolver_backend,
        subnets,
    })
}

fn subnet_matches_filters(subnet: &SubnetInfo, filters: SubnetCatalogFilters) -> bool {
    filters.kind.is_none_or(|kind| subnet.subnet_kind == kind)
        && filters
            .specialization
            .is_none_or(|specialization| subnet.subnet_specialization == specialization)
        && filters
            .geographic_scope
            .is_none_or(|scope| subnet.geographic_scope == scope)
}

fn subnet_row(
    catalog: &SubnetCatalog,
    subnet: &SubnetInfo,
    request: &SubnetCatalogListRequest,
) -> SubnetCatalogSubnetRow {
    let ranges = catalog.routing_ranges_for_subnet(&subnet.subnet_principal);
    let range_count = ranges.len();
    let shown_ranges = if request.show_ranges {
        ranges
            .into_iter()
            .skip(request.range_offset)
            .take(request.range_limit)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    SubnetCatalogSubnetRow {
        subnet_principal: subnet.subnet_principal.clone(),
        subnet_kind: subnet.subnet_kind,
        subnet_kind_source: subnet.subnet_kind_source,
        subnet_specialization: subnet.subnet_specialization,
        subnet_specialization_source: subnet.subnet_specialization_source,
        geographic_scope: subnet.geographic_scope,
        geographic_scope_source: subnet.geographic_scope_source,
        subnet_label: subnet.subnet_label.clone(),
        subnet_label_source: subnet.subnet_label_source,
        node_count: subnet.node_count,
        charges_apply_by_default: subnet.charges_apply_by_default,
        range_count,
        ranges_shown: shown_ranges.len(),
        range_offset: request.range_offset,
        range_limit: request.range_limit,
        ranges: shown_ranges,
    }
}
