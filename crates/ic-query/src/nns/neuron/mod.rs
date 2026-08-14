//! Module: nns::neuron
//!
//! Responsibility: expose public NNS neuron reporting contracts.
//! Does not own: NNS proposal reporting or Dashboard-derived neuron analytics.
//! Boundary: re-exports the direct Governance neuron report family.

mod report;

#[cfg(feature = "nns-host")]
pub use report::{
    DEFAULT_NNS_NEURON_REFRESH_LOCK_STALE_SECONDS, NnsNeuronCacheStatusReport,
    NnsNeuronCacheSummary, NnsNeuronHostError, NnsNeuronRefreshReport,
    build_nns_neuron_cache_status_report, build_nns_neuron_info_report,
    build_nns_neuron_info_report_from_cache, build_nns_neuron_list_report,
    build_nns_neuron_list_report_from_cache, nns_neuron_cache_path,
    nns_neuron_cache_status_report_text, nns_neuron_refresh_attempt_path,
    nns_neuron_refresh_lock_path, nns_neuron_refresh_report_text, refresh_nns_neuron_cache,
    refresh_nns_neuron_cache_with_progress, refresh_nns_neuron_cache_with_source,
};
pub use report::{
    DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, NNS_NEURON_MAX_PAGE_SIZE, NnsKnownNeuronData,
    NnsNeuronBallotRow, NnsNeuronError, NnsNeuronInfoReport, NnsNeuronInfoRequest,
    NnsNeuronListReport, NnsNeuronListRequest, NnsNeuronPage, NnsNeuronRow, NnsNeuronSource,
    NnsNeuronSourceFuture, NnsNeuronState, NnsNeuronType, NnsNeuronVisibility, NnsNeuronVote,
    build_nns_neuron_info_report_with_source, build_nns_neuron_list_report_with_source,
    nns_neuron_info_report_text, nns_neuron_list_report_text,
};
