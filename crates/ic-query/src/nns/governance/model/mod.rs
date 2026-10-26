//! Module: nns::governance::model
//!
//! Responsibility: expose shared NNS Governance report and collection models.
//! Does not own: live transport, CLI parsing, caching, or text rendering.
//! Boundary: preserves one explicit facade across native Governance report families.

mod classification;
mod economics;
mod events;
mod metrics;

use super::{NNS_GOVERNANCE_REPORT_SCHEMA_VERSION, NnsGovernanceRequest};
use crate::nns::MAINNET_GOVERNANCE_CANISTER_ID;
#[cfg(feature = "nns-host")]
use crate::{nns::governance::NnsGovernanceCacheMetadata, snapshot_cache::SnapshotEnvelope};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::fmt;

pub use classification::NnsGovernanceVote;
pub(in crate::nns) use classification::{
    deserialize_code_label, native_code_classification, native_optional_code_classification,
};

///
/// NnsGovernanceCollectionStatus
///
/// Lifecycle of a caller-owned resumable NNS Governance collection.
/// Each collection owns its cursor and API exhaustion rules.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NnsGovernanceCollectionStatus {
    /// No source page has been admitted yet.
    Ready,
    /// Another bounded page may be requested.
    Collecting,
    /// Governance API exhaustion was observed.
    Complete,
    /// Another cursor exists, but the configured page ceiling was consumed.
    PageLimitReached,
}

impl NnsGovernanceCollectionStatus {
    /// Return the stable JSON and display label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Collecting => "collecting",
            Self::Complete => "complete",
            Self::PageLimitReached => "page_limit_reached",
        }
    }
}

impl fmt::Display for NnsGovernanceCollectionStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

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

impl NnsGovernanceReportContext {
    pub(in crate::nns) fn from_source(
        request: &NnsGovernanceRequest,
        source: NnsGovernanceSourceProvenance,
    ) -> Self {
        Self {
            schema_version: NNS_GOVERNANCE_REPORT_SCHEMA_VERSION,
            network: request.network.clone(),
            governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
            fetched_at: request.fetched_at.clone(),
            source,
        }
    }

    #[cfg(feature = "nns-host")]
    pub(in crate::nns) fn from_snapshot<Rows>(
        cache: &SnapshotEnvelope<NnsGovernanceCacheMetadata, Rows>,
    ) -> Self {
        Self {
            schema_version: 1,
            network: cache.network.clone(),
            governance_canister_id: cache.metadata.governance_canister_id.clone(),
            fetched_at: cache.fetched_at.clone(),
            source: NnsGovernanceSourceProvenance::ReplicaQuery {
                endpoint: cache.source_endpoint.clone(),
                fetched_by: cache.fetched_by.clone(),
            },
        }
    }
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

#[cfg(test)]
mod tests {
    use super::NnsGovernanceCollectionStatus;

    #[test]
    fn collection_status_json_and_display_use_the_supported_labels() {
        for (status, label) in [
            (NnsGovernanceCollectionStatus::Ready, "ready"),
            (NnsGovernanceCollectionStatus::Collecting, "collecting"),
            (NnsGovernanceCollectionStatus::Complete, "complete"),
            (
                NnsGovernanceCollectionStatus::PageLimitReached,
                "page_limit_reached",
            ),
        ] {
            let value = serde_json::json!(label);
            assert_eq!(
                serde_json::to_value(status).expect("serialize status"),
                value
            );
            assert_eq!(
                serde_json::from_value::<NnsGovernanceCollectionStatus>(value)
                    .expect("deserialize status"),
                status
            );
            assert_eq!(status.as_str(), label);
            assert_eq!(status.to_string(), label);
        }
    }
}
