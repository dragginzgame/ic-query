//! Module: sns::report::build::neuron
//!
//! Responsibility: build one exact SNS neuron detail report.
//! Does not own: command parsing, live wire decoding, or text rendering.
//! Boundary: validates the id, resolves one SNS, fetches one neuron, and validates source evidence.

use crate::report::ReportDataSource;
use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SNS_NEURON_DETAIL_REPORT_SCHEMA_VERSION, SnsHostError,
    SnsNeuronDetailReport, SnsNeuronRequest,
    live::LiveSnsSource,
    lookup::resolve_sns_lookup,
    source::{SnsNeuronSource, sns_neuron_id_from_text, validate_mainnet_sns_neuron},
};

/// Build one exact live SNS neuron detail report.
pub async fn build_sns_neuron_detail_report(
    request: &SnsNeuronRequest,
) -> Result<SnsNeuronDetailReport, SnsHostError> {
    build_sns_neuron_detail_report_with_source(request, &LiveSnsSource::default()).await
}

/// Build one exact SNS neuron detail report through an explicit source adapter.
pub async fn build_sns_neuron_detail_report_with_source(
    request: &SnsNeuronRequest,
    source: &dyn SnsNeuronSource,
) -> Result<SnsNeuronDetailReport, SnsHostError> {
    sns_neuron_id_from_text(&request.neuron_id)?;
    let lookup = resolve_sns_lookup(&request.lookup, source).await?;
    let neuron = source
        .fetch_sns_neuron(&lookup.fetch_request, &lookup.sns, &request.neuron_id)
        .await?;
    validate_mainnet_sns_neuron(&neuron, &request.neuron_id)?;
    Ok(SnsNeuronDetailReport {
        schema_version: SNS_NEURON_DETAIL_REPORT_SCHEMA_VERSION,
        network: lookup.fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: lookup.fetch_request.fetched_at,
        source_endpoint: lookup.fetch_request.endpoint,
        fetched_by: lookup.fetch_request.fetched_by,
        id: lookup.sns.id,
        name: lookup.sns.name,
        root_canister_id: lookup.sns.canisters.root_canister_id,
        governance_canister_id: lookup.sns.canisters.governance_canister_id,
        neuron_id: request.neuron_id.clone(),
        data_source: ReportDataSource::Live,
        detail: neuron,
    })
}
