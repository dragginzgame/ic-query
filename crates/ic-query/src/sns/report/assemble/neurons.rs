//! Module: sns::report::assemble::neurons
//!
//! Responsibility: assemble live and cached SNS neuron list reports.
//! Does not own: neuron fetching, sorting, cache loading, or rendering.
//! Boundary: carries source scope and pre-limit totals supplied by each caller.

use crate::report::ReportProvenance;
use crate::sns::report::SnsReportContext;
use crate::sns::report::{
    SNS_NEURONS_REPORT_SCHEMA_VERSION, SnsNeuronRow, SnsNeuronsReport, SnsNeuronsRequest,
};

/// Assemble selected neuron rows with their original acquisition scope and total.
pub(in crate::sns::report) fn sns_neurons_report_from_rows(
    request: &SnsNeuronsRequest,
    context: SnsReportContext,
    provenance: ReportProvenance,
    owner_principal_id: Option<String>,
    total_neuron_count: usize,
    neurons: Vec<SnsNeuronRow>,
) -> SnsNeuronsReport {
    SnsNeuronsReport {
        provenance,
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        context,
        requested_limit: request.limit,
        owner_principal_id,

        sort: request.collection.sort_label().to_string(),
        total_neuron_count,

        neurons,
    }
}
