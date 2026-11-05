//! Governance identity and acquisition evidence for retained neuron reports.
use serde::{Deserialize, Serialize};

///
/// SnsInstanceContext
///
/// Acquisition metadata for an explicitly addressed Governance canister.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnsInstanceContext {
    /// Requested network.
    pub network: String,
    /// Canonical acquisition timestamp.
    pub fetched_at: String,
    /// Explicit endpoint.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
}

///
/// SnsReportContext
///
/// Explicit Governance target and acquisition metadata for neuron queries.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnsReportContext {
    /// Acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub instance: SnsInstanceContext,
    /// Governance canister holding the neurons.
    pub governance_canister_id: String,
}
