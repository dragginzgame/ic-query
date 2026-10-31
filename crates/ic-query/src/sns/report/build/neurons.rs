//! Module: sns::report::build::neurons
//!
//! Responsibility: build SNS neuron list reports.
//! Does not own: command parsing, cache file primitives, row rendering, or amount formatting.
//! Boundary: routes cache-backed sorts to cache reports or live API reads to assembly.

use crate::report::ReportProvenance;
use crate::sns::report::SnsReportContext;
use crate::sns::report::{
    SnsHostError, SnsNeuronCollection, SnsNeuronsReport, SnsNeuronsRequest,
    assemble::sns_neurons_report_from_rows,
    live::LiveSnsSource,
    lookup::resolve_sns_lookup,
    neurons_cache,
    source::{SnsNeuronsSource, validate_mainnet_sns_neurons},
};

pub async fn build_sns_neurons_report(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    build_sns_neurons_report_with_source(request, &LiveSnsSource::default()).await
}

pub async fn build_sns_neurons_report_with_source(
    request: &SnsNeuronsRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsReport, SnsHostError> {
    let owner_principal_id = match &request.collection {
        SnsNeuronCollection::Cached { cache_root, sort } => {
            return neurons_cache::build_sns_neurons_report_from_cache(request, cache_root, *sort);
        }
        SnsNeuronCollection::Live { owner_principal_id } => owner_principal_id,
    };

    let lookup = resolve_sns_lookup(&request.lookup, source).await?;
    let neurons = source
        .fetch_sns_neurons(
            &lookup.fetch_request,
            &lookup.sns,
            request.limit,
            owner_principal_id.as_deref(),
        )
        .await?;
    validate_mainnet_sns_neurons(&neurons, request.limit)?;
    let total_neuron_count = neurons.len();
    Ok(sns_neurons_report_from_rows(
        request,
        SnsReportContext::from_live(lookup.fetch_request, lookup.sns),
        ReportProvenance::live(),
        owner_principal_id.clone(),
        total_neuron_count,
        neurons,
    ))
}
