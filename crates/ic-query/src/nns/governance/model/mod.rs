//! Module: nns::governance::model
//!
//! Responsibility: expose direct NNS Governance report models.
//! Does not own: live transport, CLI parsing, caching, or text rendering.
//! Boundary: preserves one explicit facade across native Governance report families.

mod economics;
mod events;
mod metrics;

use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// NnsGovernanceReportContext
///
/// Shared provenance flattened into every direct NNS Governance report.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceReportContext {
    /// Report schema version.
    pub schema_version: u32,
    /// Queried network identity.
    pub network: String,
    /// NNS Governance canister principal.
    pub governance_canister_id: String,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Replica endpoint used for the query.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
}

pub use economics::{
    NnsGovernanceDecimal, NnsGovernanceEconomics, NnsGovernanceEconomicsReport,
    NnsGovernancePercentage, NnsNeuronsFundEconomics,
    NnsNeuronsFundMatchedFundingCurveCoefficients, NnsVotingPowerEconomics,
};
pub use events::{
    NnsGovernanceMaturityModulation, NnsGovernanceMaturityModulationReport,
    NnsGovernanceProposalId, NnsGovernanceRewardEvent, NnsGovernanceRewardEventReport,
};
pub use metrics::{
    NnsGovernanceMetricBucket, NnsGovernanceMetrics, NnsGovernanceMetricsReport,
    NnsGovernanceNeuronSubsetMetrics,
};
