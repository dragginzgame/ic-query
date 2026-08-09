//! Module: cloud_engine::provider::host
//!
//! Responsibility: validate and build official Dashboard CloudEngine provider reports.
//! Does not own: HTTP transport, native control-plane queries, rendering, or caching.
//! Boundary: one complete source resource is validated before CloudEngine filtering.

use super::{
    CloudEngineProviderInfoReport, CloudEngineProviderInfoRequest,
    CloudEngineProviderInfoSourceData, CloudEngineProviderListReport,
    CloudEngineProviderListRequest, CloudEngineProviderListSourceData, CloudEngineProviderLocation,
    CloudEngineProviderRow, MAX_CLOUD_ENGINE_PROVIDER_LOCATIONS,
    MAX_CLOUD_ENGINE_PROVIDER_SOURCE_ROWS,
};
use crate::ic::{
    IcHostError, IcSourceRequest, LiveIcSource, canonical_request_principal,
    dashboard_source_request, invalid_source, report_provenance, validate_canonical_principal,
    validate_dashboard_network, validate_provenance,
};
use std::collections::HashSet;

///
/// CloudEngineProviderSource
///
/// Official Dashboard capability for complete and exact node-provider records.
///

pub trait CloudEngineProviderSource {
    /// Fetch the complete non-paginated node-provider resource once.
    fn fetch_cloud_engine_provider_list(
        &self,
        request: &IcSourceRequest,
    ) -> Result<CloudEngineProviderListSourceData, IcHostError>;

    /// Fetch one exact node-provider record once.
    fn fetch_cloud_engine_provider_info(
        &self,
        request: &IcSourceRequest,
        node_provider_id: &str,
    ) -> Result<CloudEngineProviderInfoSourceData, IcHostError>;
}

/// Build one live complete CloudEngine provider report from the official Dashboard.
pub fn build_cloud_engine_provider_list_report(
    request: &CloudEngineProviderListRequest,
) -> Result<CloudEngineProviderListReport, IcHostError> {
    build_cloud_engine_provider_list_report_with_source(request, &LiveIcSource)
}

/// Build one complete CloudEngine provider report through a custom Dashboard source.
pub fn build_cloud_engine_provider_list_report_with_source(
    request: &CloudEngineProviderListRequest,
    source: &dyn CloudEngineProviderSource,
) -> Result<CloudEngineProviderListReport, IcHostError> {
    validate_dashboard_network(&request.network)?;
    let expected = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let mut source_data = source.fetch_cloud_engine_provider_list(&expected)?;
    validate_provenance(&expected, &source_data.source)?;
    validate_complete_provider_resource(&mut source_data.providers)?;

    let source_node_provider_count = source_data.providers.len();
    let providers = source_data
        .providers
        .into_iter()
        .filter(CloudEngineProviderRow::has_cloud_engine_evidence)
        .collect::<Vec<_>>();
    Ok(CloudEngineProviderListReport {
        provenance: report_provenance(source_data.source),
        source_node_provider_count,
        cloud_engine_provider_count: providers.len(),
        providers,
    })
}

/// Build one live exact CloudEngine provider report from the official Dashboard.
pub fn build_cloud_engine_provider_info_report(
    request: &CloudEngineProviderInfoRequest,
) -> Result<CloudEngineProviderInfoReport, IcHostError> {
    build_cloud_engine_provider_info_report_with_source(request, &LiveIcSource)
}

/// Build one exact CloudEngine provider report through a custom Dashboard source.
pub fn build_cloud_engine_provider_info_report_with_source(
    request: &CloudEngineProviderInfoRequest,
    source: &dyn CloudEngineProviderSource,
) -> Result<CloudEngineProviderInfoReport, IcHostError> {
    validate_dashboard_network(&request.network)?;
    let requested_provider =
        canonical_request_principal("node_provider_id", &request.node_provider_id)?;
    let expected = dashboard_source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_cloud_engine_provider_info(&expected, &requested_provider)?;
    validate_provenance(&expected, &source_data.source)?;
    validate_provider(&source_data.provider)?;
    if source_data.provider.principal_id != requested_provider {
        return invalid_source(format!(
            "node provider id is {:?}, expected requested principal {requested_provider:?}",
            source_data.provider.principal_id
        ));
    }

    Ok(CloudEngineProviderInfoReport {
        provenance: report_provenance(source_data.source),
        cloud_engine_evidence_present: source_data.provider.has_cloud_engine_evidence(),
        provider: source_data.provider,
    })
}

fn validate_complete_provider_resource(
    providers: &mut [CloudEngineProviderRow],
) -> Result<(), IcHostError> {
    if providers.is_empty() {
        return invalid_source("complete provider resource contains no rows");
    }
    if providers.len() > MAX_CLOUD_ENGINE_PROVIDER_SOURCE_ROWS {
        return invalid_source(format!(
            "provider resource contains {} rows; maximum is {MAX_CLOUD_ENGINE_PROVIDER_SOURCE_ROWS}",
            providers.len()
        ));
    }

    let mut principals = HashSet::with_capacity(providers.len());
    for provider in providers.iter() {
        validate_provider(provider)?;
        if !principals.insert(provider.principal_id.as_str()) {
            return invalid_source(format!(
                "duplicate node-provider principal {:?}",
                provider.principal_id
            ));
        }
    }
    providers.sort_unstable_by(|left, right| left.principal_id.cmp(&right.principal_id));
    Ok(())
}

fn validate_provider(provider: &CloudEngineProviderRow) -> Result<(), IcHostError> {
    validate_canonical_principal("provider.principal_id", &provider.principal_id)?;
    validate_text("provider.display_name", &provider.display_name, 256)?;
    validate_optional_text("provider.website", provider.website.as_deref(), 2_048)?;
    validate_optional_text("provider.logo_url", provider.logo_url.as_deref(), 2_048)?;
    validate_locations(
        "provider.locations",
        provider.location_count,
        &provider.locations,
    )?;
    validate_locations(
        "provider.cloud_engine_locations",
        provider.cloud_engine_location_count,
        &provider.cloud_engine_locations,
    )?;
    if provider.total_unassigned_nodes > provider.total_nodes {
        return invalid_source(format!(
            "provider {} has {} unassigned nodes but only {} total nodes",
            provider.principal_id, provider.total_unassigned_nodes, provider.total_nodes
        ));
    }
    if provider.total_cloud_engine_unassigned_nodes > provider.total_cloud_engine_nodes {
        return invalid_source(format!(
            "provider {} has {} unassigned CloudEngine nodes but only {} total CloudEngine nodes",
            provider.principal_id,
            provider.total_cloud_engine_unassigned_nodes,
            provider.total_cloud_engine_nodes
        ));
    }
    Ok(())
}

fn validate_locations(
    field: &'static str,
    declared_count: usize,
    locations: &[CloudEngineProviderLocation],
) -> Result<(), IcHostError> {
    if locations.len() != declared_count {
        return invalid_source(format!(
            "{field} contains {} rows, expected declared count {declared_count}",
            locations.len()
        ));
    }
    if locations.len() > MAX_CLOUD_ENGINE_PROVIDER_LOCATIONS {
        return invalid_source(format!(
            "{field} contains {} rows; maximum is {MAX_CLOUD_ENGINE_PROVIDER_LOCATIONS}",
            locations.len()
        ));
    }
    let mut data_centers = HashSet::with_capacity(locations.len());
    for location in locations {
        validate_text("location.dc_key", &location.dc_key, 64)?;
        validate_text("location.display_name", &location.display_name, 256)?;
        validate_raw_label("location.owner", &location.owner, 256)?;
        validate_text("location.region", &location.region, 512)?;
        if !location.latitude.is_finite() || !(-90.0..=90.0).contains(&location.latitude) {
            return invalid_source(format!(
                "location {:?} latitude {} is outside the finite geographic range",
                location.dc_key, location.latitude
            ));
        }
        if !location.longitude.is_finite() || !(-180.0..=180.0).contains(&location.longitude) {
            return invalid_source(format!(
                "location {:?} longitude {} is outside the finite geographic range",
                location.dc_key, location.longitude
            ));
        }
        if !data_centers.insert(location.dc_key.as_str()) {
            return invalid_source(format!(
                "{field} contains duplicate data-center key {:?}",
                location.dc_key
            ));
        }
    }
    Ok(())
}

fn validate_optional_text(
    field: &'static str,
    value: Option<&str>,
    max_bytes: usize,
) -> Result<(), IcHostError> {
    if let Some(value) = value {
        validate_text(field, value, max_bytes)?;
    }
    Ok(())
}

fn validate_raw_label(
    field: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), IcHostError> {
    if value.trim().is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return invalid_source(format!(
            "{field} must contain visible text of at most {max_bytes} bytes without control characters"
        ));
    }
    Ok(())
}

fn validate_text(field: &'static str, value: &str, max_bytes: usize) -> Result<(), IcHostError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return invalid_source(format!(
            "{field} must be nonempty trimmed text of at most {max_bytes} bytes without control characters"
        ));
    }
    Ok(())
}
