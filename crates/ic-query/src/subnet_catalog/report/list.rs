use super::{
    SubnetCatalogFilters, SubnetCatalogListReport, SubnetCatalogListRequest,
    SubnetCatalogSubnetRow, load_catalog_for_report,
};
use crate::{QueryProgress, progress::IgnoreQueryProgress};
use crate::{
    nns::LiveNnsSource,
    subnet_catalog::{
        RoutingRange, SUBNET_CATALOG_LIST_REPORT_SCHEMA_VERSION, SubnetCatalogHostError,
        SubnetCatalogSource, SubnetInfo,
    },
};
use std::collections::BTreeMap;

/// Load the catalog under the request's cache policy and project its filtered Subnet rows.
pub async fn build_subnet_catalog_list_report(
    request: &SubnetCatalogListRequest,
) -> Result<SubnetCatalogListReport, SubnetCatalogHostError> {
    build_subnet_catalog_list_report_with_source(request, &LiveNnsSource::default()).await
}

/// Build the report while reporting authorized live refreshes.
pub async fn build_subnet_catalog_list_report_with_progress(
    request: &SubnetCatalogListRequest,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<SubnetCatalogListReport, SubnetCatalogHostError> {
    build_subnet_catalog_list_report_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        progress,
    )
    .await
}

/// Project filtered Subnet rows using the cache policy and caller-owned refresh source.
pub async fn build_subnet_catalog_list_report_with_source(
    request: &SubnetCatalogListRequest,
    source: &dyn SubnetCatalogSource,
) -> Result<SubnetCatalogListReport, SubnetCatalogHostError> {
    build_subnet_catalog_list_report_with_source_and_progress(
        request,
        source,
        &mut IgnoreQueryProgress,
    )
    .await
}

/// Build the report with a caller-owned source and progress sink.
pub async fn build_subnet_catalog_list_report_with_source_and_progress(
    request: &SubnetCatalogListRequest,
    source: &dyn SubnetCatalogSource,
    progress: &mut (dyn QueryProgress + Send),
) -> Result<SubnetCatalogListReport, SubnetCatalogHostError> {
    let (cached, stale) = load_catalog_for_report(
        &request.cache,
        &request.read_policy,
        request.now_unix_secs,
        request.stale_after_seconds,
        source,
        progress,
    )
    .await?;
    let subnets = cached
        .catalog
        .subnets()
        .iter()
        .filter(|subnet| subnet_matches_filters(subnet, request.filters))
        .collect::<Vec<_>>();
    let mut ranges_by_subnet = subnets
        .iter()
        .map(|subnet| (subnet.subnet_principal.as_str(), (0_usize, Vec::new())))
        .collect::<BTreeMap<_, _>>();
    for range in &cached.catalog.raw().routing_ranges {
        if let Some((count, shown)) = ranges_by_subnet.get_mut(range.subnet_principal.as_str()) {
            if request.show_ranges
                && *count >= request.range_offset
                && shown.len() < request.range_limit
            {
                shown.push(range.clone());
            }
            *count += 1;
        }
    }
    let subnets = subnets
        .into_iter()
        .map(|subnet| {
            let (count, ranges) = ranges_by_subnet
                .remove(subnet.subnet_principal.as_str())
                .unwrap_or_default();
            subnet_row(subnet, request, count, ranges)
        })
        .collect();

    Ok(
        super::catalog_report!(SubnetCatalogListReport, cached, stale, {
            schema_version: SUBNET_CATALOG_LIST_REPORT_SCHEMA_VERSION,
            network: cached.catalog.provenance().network.clone(),
            subnets,
        }),
    )
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
    subnet: &SubnetInfo,
    request: &SubnetCatalogListRequest,
    range_count: usize,
    shown_ranges: Vec<RoutingRange>,
) -> SubnetCatalogSubnetRow {
    SubnetCatalogSubnetRow {
        subnet_principal: subnet.subnet_principal.clone(),
        registry_subnet_type: subnet.registry_subnet_type,
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
