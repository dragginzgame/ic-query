//! Module: sns::report::build::info
//!
//! Responsibility: build one deployed SNS info report.
//! Does not own: command parsing, SNS-W transport internals, or rendering.
//! Boundary: resolves lookup input through a source and constructs its report.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SNS_INFO_REPORT_SCHEMA_VERSION, SnsHostError, SnsInfoReport,
    SnsLookupRequest, live::LiveSnsSource, lookup::resolve_sns_lookup, source::SnsDiscoverySource,
};

pub async fn build_sns_info_report(
    request: &SnsLookupRequest,
) -> Result<SnsInfoReport, SnsHostError> {
    build_sns_info_report_with_source(request, &LiveSnsSource::default()).await
}

pub async fn build_sns_info_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsDiscoverySource,
) -> Result<SnsInfoReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source).await?;
    Ok(SnsInfoReport {
        schema_version: SNS_INFO_REPORT_SCHEMA_VERSION,
        network: lookup.fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: lookup.fetch_request.fetched_at,
        source_endpoint: lookup.fetch_request.endpoint,
        fetched_by: lookup.fetch_request.fetched_by,
        id: lookup.sns.id,
        name: lookup.sns.name,
        description: lookup.sns.description,
        url: lookup.sns.url,
        canisters: lookup.sns.canisters,
        metadata_error: lookup.sns.metadata_error,
    })
}
