//! Module: sns::report::model::reports::neurons::report
//!
//! Responsibility: SNS neuron listing report DTO.
//! Does not own: live fetching, cache-backed sorting, or rendering.
//! Boundary: preserves neuron listing metadata and rows for text and JSON.

use super::row::SnsNeuronRow;
use crate::report::ReportDataSource;
use crate::sns::report::SnsReportContext;
use serde::Serialize;

///
/// SnsNeuronsReport
///
/// Serializable report for bounded or cache-backed SNS neuron listings.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsReport {
    pub schema_version: u32,
    /// SNS identity and provenance, serialized as flat report fields.
    #[serde(flatten)]
    pub context: SnsReportContext,
    pub requested_limit: u32,
    pub owner_principal_id: Option<String>,
    pub verbose: bool,
    pub data_source: ReportDataSource,
    pub sort: String,
    pub cache_path: Option<String>,
    pub cache_complete: Option<bool>,
    pub total_neuron_count: usize,
    pub neuron_count: usize,
    pub neurons: Vec<SnsNeuronRow>,
}
