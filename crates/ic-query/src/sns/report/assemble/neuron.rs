//! Module: sns::report::assemble::neuron
//!
//! Responsibility: assemble one exact SNS neuron detail report.
//! Does not own: neuron fetching, target discovery, validation, or rendering.
//! Boundary: maps resolved live source parts into the serializable detail report.

use crate::{
    report::ReportDataSource,
    sns::report::{
        MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsNeuron,
        SNS_NEURON_DETAIL_REPORT_SCHEMA_VERSION, SnsNeuronDetailReport, SnsSourceRequest,
    },
};

///
/// SnsNeuronDetailReportParts
///
/// Resolved live inputs needed to assemble one exact SNS neuron detail report.
///

pub(in crate::sns::report) struct SnsNeuronDetailReportParts {
    pub(in crate::sns::report) fetch_request: SnsSourceRequest,
    pub(in crate::sns::report) sns: MainnetSns,
    pub(in crate::sns::report) neuron_id: String,
    pub(in crate::sns::report) neuron: MainnetSnsNeuron,
}

/// Assemble an exact SNS neuron detail report from resolved live source parts.
pub(in crate::sns::report) fn sns_neuron_detail_report_from_parts(
    parts: SnsNeuronDetailReportParts,
) -> SnsNeuronDetailReport {
    SnsNeuronDetailReport {
        schema_version: SNS_NEURON_DETAIL_REPORT_SCHEMA_VERSION,
        network: parts.fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: parts.fetch_request.fetched_at,
        source_endpoint: parts.fetch_request.endpoint,
        fetched_by: parts.fetch_request.fetched_by,
        id: parts.sns.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        neuron_id: parts.neuron_id,
        data_source: ReportDataSource::Live,
        detail: parts.neuron.detail,
    }
}
