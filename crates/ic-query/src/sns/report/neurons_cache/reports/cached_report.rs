//! Module: sns::report::neurons_cache::reports::cached_report
//!
//! Responsibility: build normal SNS neuron list reports from complete local snapshots.
//! Does not own: live neuron fetching, cache refresh, text rendering, or CLI parsing.
//! Boundary: applies view sorting/limit over one complete cached neuron snapshot.

use crate::report::ReportProvenance;
use crate::sns::report::SnsReportContext;
use crate::sns::report::{
    SnsHostError, SnsNeuronsReport, SnsNeuronsRequest, assemble::sns_neurons_report_from_rows,
    cache_storage::load_sns_cache_for_governance, view::sort_sns_neurons,
};

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
