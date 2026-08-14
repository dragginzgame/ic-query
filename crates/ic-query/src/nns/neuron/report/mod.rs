//! Module: nns::neuron::report
//!
//! Responsibility: build direct NNS Governance neuron reports.
//! Does not own: Dashboard analytics, authenticated neuron management, or process output.
//! Boundary: preserves the public `NeuronInfo` view and its Governance provenance.

#[cfg(feature = "nns-host")]
mod cache;
mod classification;
mod error;
mod model;
mod source;
mod text;
#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32"),
    test
))]
mod wire;

#[cfg(feature = "nns-host")]
pub use cache::{
    DEFAULT_NNS_NEURON_REFRESH_LOCK_STALE_SECONDS, NnsNeuronCacheStatusReport,
    NnsNeuronCacheSummary, NnsNeuronRefreshReport, build_nns_neuron_cache_status_report,
    build_nns_neuron_info_report_from_cache, build_nns_neuron_list_report_from_cache,
    nns_neuron_cache_path, nns_neuron_refresh_attempt_path, nns_neuron_refresh_lock_path,
    refresh_nns_neuron_cache, refresh_nns_neuron_cache_with_progress,
    refresh_nns_neuron_cache_with_source,
};
pub use classification::{NnsNeuronState, NnsNeuronType, NnsNeuronVisibility, NnsNeuronVote};
pub use error::NnsNeuronError;
#[cfg(feature = "nns-host")]
pub use error::NnsNeuronHostError;
pub use model::{
    NnsKnownNeuronData, NnsNeuronBallotRow, NnsNeuronInfoReport, NnsNeuronInfoRequest,
    NnsNeuronListReport, NnsNeuronListRequest, NnsNeuronRow,
};
pub use source::{
    NnsNeuronPage, NnsNeuronSource, NnsNeuronSourceFuture,
    build_nns_neuron_info_report_with_source, build_nns_neuron_list_report_with_source,
};
#[cfg(feature = "nns-host")]
pub use source::{build_nns_neuron_info_report, build_nns_neuron_list_report};
#[cfg(feature = "nns-host")]
pub use text::{nns_neuron_cache_status_report_text, nns_neuron_refresh_report_text};
pub use text::{nns_neuron_info_report_text, nns_neuron_list_report_text};

#[cfg(all(test, feature = "nns-host"))]
mod tests;

/// Default replica endpoint used for direct NNS Governance neuron queries.
pub const DEFAULT_NNS_NEURON_SOURCE_ENDPOINT: &str = "https://icp-api.io";

/// Largest page size accepted by the public Governance neuron index.
pub const NNS_NEURON_MAX_PAGE_SIZE: u32 = 300;

const NNS_NEURON_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_NEURON_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
#[cfg(feature = "nns-host")]
const NNS_NEURON_FETCHED_BY: &str = "ic-query";

#[cfg(feature = "nns-host")]
fn enforce_mainnet_network(network: &str) -> Result<(), NnsNeuronHostError> {
    crate::nns::governance::enforce_governance_mainnet_network(network)
        .map_err(NnsNeuronError::from)
        .map_err(NnsNeuronHostError::from)
}
