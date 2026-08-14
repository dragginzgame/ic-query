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
/// NnsGovernanceExecutionAssurance
///
/// Execution semantics derived from direct Governance source provenance.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsGovernanceExecutionAssurance {
    /// An ordinary replica query without replicated execution semantics.
    UnreplicatedQuery,
    /// A replicated inter-canister call executed by the IC.
    ReplicatedExecution,
}

impl NnsGovernanceExecutionAssurance {
    /// Return the stable display label for this assurance.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnreplicatedQuery => "unreplicated_query",
            Self::ReplicatedExecution => "replicated_execution",
        }
    }
}

///
/// NnsGovernanceSourceProvenance
///
/// Transport and collector evidence for one direct NNS Governance report.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(tag = "source_transport", rename_all = "snake_case")]
pub enum NnsGovernanceSourceProvenance {
    /// An ordinary unreplicated query submitted through a replica endpoint.
    ReplicaQuery {
        /// Replica endpoint used for the query.
        endpoint: String,
        /// Collector identity supplied by the caller.
        fetched_by: String,
    },
    /// A replicated call made by one collector canister.
    ReplicatedInterCanisterCall {
        /// Principal of the canister that executed the call.
        collector_canister_id: String,
    },
}

impl NnsGovernanceSourceProvenance {
    /// Return the execution assurance implied by this transport.
    #[must_use]
    pub const fn execution_assurance(&self) -> NnsGovernanceExecutionAssurance {
        match self {
            Self::ReplicaQuery { .. } => NnsGovernanceExecutionAssurance::UnreplicatedQuery,
            Self::ReplicatedInterCanisterCall { .. } => {
                NnsGovernanceExecutionAssurance::ReplicatedExecution
            }
        }
    }
}

///
/// NnsGovernanceReportContext
///
/// Shared context embedded in every direct NNS Governance report.
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
    /// Transport-specific source provenance.
    pub source: NnsGovernanceSourceProvenance,
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
