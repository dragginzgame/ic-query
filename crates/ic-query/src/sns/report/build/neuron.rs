//! Module: sns::report::build::neuron
//!
//! Responsibility: build one exact SNS neuron detail report.
//! Does not own: command parsing, live wire decoding, or text rendering.
//! Boundary: validates the id, resolves one SNS, fetches one neuron, and validates source evidence.

use crate::sns::report::{
    SnsHostError, SnsNeuronDetailReport, SnsNeuronRequest,
    assemble::{SnsNeuronDetailReportParts, sns_neuron_detail_report_from_parts},
    live::LiveSnsSource,
    lookup::{lookup_request_from_parts, resolve_sns_lookup},
    source::{SnsNeuronSource, sns_neuron_id_from_text, validate_mainnet_sns_neuron},
};

/// Build one exact live SNS neuron detail report.
pub fn build_sns_neuron_detail_report(
    request: &SnsNeuronRequest,
) -> Result<SnsNeuronDetailReport, SnsHostError> {
    build_sns_neuron_detail_report_with_source(request, &LiveSnsSource)
}

/// Build one exact SNS neuron detail report through an explicit source adapter.
pub fn build_sns_neuron_detail_report_with_source(
    request: &SnsNeuronRequest,
    source: &dyn SnsNeuronSource,
) -> Result<SnsNeuronDetailReport, SnsHostError> {
    sns_neuron_id_from_text(&request.neuron_id)?;
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let neuron = source.fetch_sns_neuron(&lookup.fetch_request, &lookup.sns, &request.neuron_id)?;
    validate_mainnet_sns_neuron(&neuron, &request.neuron_id)?;
    Ok(sns_neuron_detail_report_from_parts(
        SnsNeuronDetailReportParts {
            list: lookup.list,
            id: lookup.id,
            sns: lookup.sns,
            neuron_id: request.neuron_id.clone(),
            neuron,
        },
    ))
}
