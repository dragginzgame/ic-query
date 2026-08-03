//! Module: sns::report::assemble::neuron
//!
//! Responsibility: assemble one exact SNS neuron detail report.
//! Does not own: neuron fetching, target discovery, validation, or rendering.
//! Boundary: maps resolved live source parts into the serializable detail report.

use crate::sns::report::{
    JoinedMainnetSnsInventory, MainnetSns, MainnetSnsNeuron,
    SNS_NEURON_DETAIL_REPORT_SCHEMA_VERSION, SnsNeuronDetailReport,
};

///
/// SnsNeuronDetailReportParts
///
/// Resolved live inputs needed to assemble one exact SNS neuron detail report.
///

pub(in crate::sns::report) struct SnsNeuronDetailReportParts {
    pub(in crate::sns::report) list: JoinedMainnetSnsInventory,
    pub(in crate::sns::report) id: usize,
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
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        neuron_id: parts.neuron_id,
        data_source: "live".to_string(),
        detail: parts.neuron.detail,
    }
}
