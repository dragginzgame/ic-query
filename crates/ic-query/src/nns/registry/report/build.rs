use super::{
    error::NnsRegistryHostError,
    model::{
        NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION, NnsRegistryVersionReport,
        NnsRegistryVersionRequest,
    },
    source::{NnsRegistrySource, NnsRegistryVersionData},
};
use crate::{
    nns::{LiveNnsSource, NnsSourceRequest},
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};

impl_nns_mainnet_network_enforcer!(NnsRegistryHostError);

pub fn build_nns_registry_version_report(
    request: &NnsRegistryVersionRequest,
) -> Result<NnsRegistryVersionReport, NnsRegistryHostError> {
    build_nns_registry_version_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_registry_version_report_with_source(
    request: &NnsRegistryVersionRequest,
    source: &dyn NnsRegistrySource,
) -> Result<NnsRegistryVersionReport, NnsRegistryHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let fetch_request = NnsSourceRequest::new(
        MAINNET_NETWORK,
        &request.source_endpoint,
        fetched_at,
        "ic-query",
    );
    let version = source.fetch_registry_version(&fetch_request)?;
    Ok(registry_version_report_from_version(version))
}

fn registry_version_report_from_version(
    version: NnsRegistryVersionData,
) -> NnsRegistryVersionReport {
    NnsRegistryVersionReport {
        schema_version: NNS_REGISTRY_VERSION_REPORT_SCHEMA_VERSION,
        network: version.network,
        registry_canister_id: version.registry_canister_id,
        registry_version: version.registry_version,
        fetched_at: version.fetched_at,
        source_endpoint: version.source_endpoint,
        fetched_by: version.fetched_by,
    }
}
