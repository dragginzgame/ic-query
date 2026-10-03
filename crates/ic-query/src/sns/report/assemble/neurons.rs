//! Module: sns::report::assemble::neurons
//!
//! Responsibility: assemble live SNS neuron list reports.
//! Does not own: neuron fetching, cache-backed sorting, cache loading, or rendering.
//! Boundary: maps resolved live neuron rows into the serializable neuron report DTO.

use super::SnsReportProvenance;
use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsNeurons, SNS_NEURONS_REPORT_SCHEMA_VERSION,
    SnsNeuronsReport, SnsNeuronsSort, SnsSourceRequest,
};

///
/// SnsNeuronsLiveReportParts
///
/// Live-source inputs needed to assemble an SNS neurons report.
///

pub(in crate::sns::report) struct SnsNeuronsLiveReportParts {
    pub(in crate::sns::report) fetch_request: SnsSourceRequest,
    pub(in crate::sns::report) sns: MainnetSns,
    pub(in crate::sns::report) requested_limit: u32,
    pub(in crate::sns::report) owner_principal_id: Option<String>,
    pub(in crate::sns::report) sort: SnsNeuronsSort,
    pub(in crate::sns::report) verbose: bool,
    pub(in crate::sns::report) neurons: MainnetSnsNeurons,
}

/// Assemble an SNS neurons report from resolved live-source parts.
pub(in crate::sns::report) fn sns_neurons_report_from_parts(
    parts: SnsNeuronsLiveReportParts,
) -> SnsNeuronsReport {
    let neuron_count = parts.neurons.neurons.len();
    let provenance = SnsReportProvenance::live();
    SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: parts.fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: parts.fetch_request.fetched_at,
        source_endpoint: parts.fetch_request.endpoint,
        fetched_by: parts.fetch_request.fetched_by,
        id: parts.sns.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        owner_principal_id: parts.owner_principal_id,
        verbose: parts.verbose,
        data_source: provenance.data_source,
        sort: parts.sort.as_str().to_string(),
        cache_path: provenance.cache_path,
        cache_complete: provenance.cache_complete,
        total_neuron_count: neuron_count,
        neuron_count,
        neurons: parts.neurons.neurons,
    }
}
