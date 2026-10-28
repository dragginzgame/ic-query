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

pub use paths::{
    nns_neuron_cache_path, nns_neuron_refresh_attempt_path, nns_neuron_refresh_lock_path,
};
pub use refresh::{
    refresh_nns_neuron_cache, refresh_nns_neuron_cache_with_progress,
    refresh_nns_neuron_cache_with_source,
};
pub(super) use reports::AdmittedNeuronCache;

pub use reports::{
    build_nns_neuron_cache_status_report, build_nns_neuron_distribution_report_from_cache,
    build_nns_neuron_info_report_from_cache, build_nns_neuron_info_report_with_cache_and_source,
    build_nns_neuron_list_report_from_cache, build_nns_neuron_list_report_with_cache_and_source,
};

const NNS_NEURON_CACHE_SCHEMA_VERSION: u32 = 1;
const NNS_NEURON_CACHE_COMPONENT: &str = "NNS neuron";
