//! Module: sns::report::neurons_cache
//!
//! Responsibility: manage complete SNS neuron snapshot cache behavior.
//! Does not own: CLI parsing, live SNS transport, generic snapshot mechanics, or text rendering.
//! Boundary: coordinates neuron cache refresh, lookup, storage, and cache-backed reports.

use super::{
    SnsHostError, SnsNeuronsReport, SnsNeuronsRequest, SnsReportContext,
    assemble::sns_neurons_report_from_rows, cache_storage::load_sns_cache_for_governance,
    view::sort_sns_neurons,
};
use crate::report::ReportProvenance;

mod collection;
pub(super) mod model;
mod paths;
mod refresh;

pub use paths::{
    sns_neurons_cache_path, sns_neurons_refresh_attempt_path, sns_neurons_refresh_lock_path,
};
pub(in crate::sns::report) use refresh::SnsRefreshContext;
pub use refresh::{refresh_sns_neurons_cache, refresh_sns_neurons_cache_with_source_and_progress};

pub(super) const SNS_NEURONS_CACHE_SCHEMA_VERSION: u32 = 1;

pub(in crate::sns::report) fn build_sns_neurons_report_from_cache(
    request: &SnsNeuronsRequest,
    cache_root: &std::path::Path,
    sort: crate::sns::SnsNeuronsCacheSort,
) -> Result<SnsNeuronsReport, SnsHostError> {
    let (cache_path, mut cache) = load_sns_cache_for_governance(
        cache_root,
        &request.target.network,
        &request.target.governance_canister_id,
    )?;
    sort_sns_neurons(&mut cache.data.neurons, sort);
    let total_neuron_count = cache.data.neurons.len();
    let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
    cache.data.neurons.truncate(limit);
    let provenance = ReportProvenance::cache(&cache_path);
    let (context, data) = SnsReportContext::from_snapshot(cache);
    Ok(sns_neurons_report_from_rows(
        request,
        context,
        provenance,
        None,
        total_neuron_count,
        data.neurons,
    ))
}
