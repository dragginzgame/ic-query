//! Module: nns::neuron::report::cache
//!
//! Responsibility: complete NNS neuron snapshot refresh and inspection.
//! Does not own: command parsing, live governance transport, or text rendering.
//! Boundary: stores complete public neuron snapshots and refresh-attempt metadata.

mod collection;
mod model;
mod paths;
mod refresh;
mod reports;

pub use model::{NnsNeuronCacheStatusReport, NnsNeuronCacheSummary};
pub use paths::{
    nns_neuron_cache_path, nns_neuron_refresh_attempt_path, nns_neuron_refresh_lock_path,
};
pub use refresh::{
    DEFAULT_NNS_NEURON_REFRESH_LOCK_STALE_SECONDS, refresh_nns_neuron_cache,
    refresh_nns_neuron_cache_with_progress, refresh_nns_neuron_cache_with_source,
};
pub use reports::{
    build_nns_neuron_cache_status_report, build_nns_neuron_distribution_report_from_cache,
    build_nns_neuron_info_report_from_cache, build_nns_neuron_list_report_from_cache,
};

const NNS_NEURON_CACHE_SCHEMA_VERSION: u32 = 1;
const NNS_NEURON_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_NEURON_CACHE_COMPONENT: &str = "NNS neuron";
