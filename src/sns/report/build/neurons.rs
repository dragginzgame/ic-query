//! Module: sns::report::build::neurons
//!
//! Responsibility: build SNS neuron list reports.
//! Does not own: command parsing, cache file primitives, row rendering, or amount formatting.
//! Boundary: routes cache-backed sorts to cache reports or live API reads to assembly.

use crate::sns::report::{
    SnsHostError, SnsNeuronsReport, SnsNeuronsRequest,
    assemble::{SnsNeuronsLiveReportParts, sns_neurons_report_from_parts},
    live::LiveSnsSource,
    lookup::{lookup_request_from_parts, resolve_sns_lookup},
    neurons_cache,
    source::SnsNeuronsSource,
};

pub fn build_sns_neurons_report(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    build_sns_neurons_report_with_source(request, &LiveSnsSource)
}

pub(in crate::sns::report) fn build_sns_neurons_report_with_source(
    request: &SnsNeuronsRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsReport, SnsHostError> {
    if request.sort.uses_cache() {
        return neurons_cache::build_sns_neurons_report_from_cache(request);
    }

    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let neurons = source.fetch_sns_neurons(
        &lookup.fetch_request,
        &lookup.sns,
        request.limit,
        request.owner_principal_id.as_deref(),
    )?;
    Ok(sns_neurons_report_from_parts(SnsNeuronsLiveReportParts {
        list: lookup.list,
        id: lookup.id,
        sns: lookup.sns,
        requested_limit: request.limit,
        owner_principal_id: request.owner_principal_id.clone(),
        sort: request.sort,
        verbose: request.verbose,
        neurons,
    }))
}
