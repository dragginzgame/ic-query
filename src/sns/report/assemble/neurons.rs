use super::super::{
    MainnetSns, MainnetSnsList, MainnetSnsNeurons, SNS_NEURONS_REPORT_SCHEMA_VERSION,
    SnsNeuronsReport, SnsNeuronsSort,
};

pub(in crate::sns::report) struct SnsNeuronsLiveReportParts {
    pub(in crate::sns::report) list: MainnetSnsList,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) sns: MainnetSns,
    pub(in crate::sns::report) requested_limit: u32,
    pub(in crate::sns::report) owner_principal_id: Option<String>,
    pub(in crate::sns::report) sort: SnsNeuronsSort,
    pub(in crate::sns::report) verbose: bool,
    pub(in crate::sns::report) neurons: MainnetSnsNeurons,
}

pub(in crate::sns::report) fn sns_neurons_report_from_parts(
    parts: SnsNeuronsLiveReportParts,
) -> SnsNeuronsReport {
    let neuron_count = parts.neurons.neurons.len();
    SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        owner_principal_id: parts.owner_principal_id,
        verbose: parts.verbose,
        data_source: "live".to_string(),
        sort: parts.sort.as_str().to_string(),
        cache_path: None,
        cache_complete: None,
        total_neuron_count: neuron_count,
        neuron_count,
        neurons: parts.neurons.neurons,
    }
}
