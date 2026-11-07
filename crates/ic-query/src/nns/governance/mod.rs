//! Reusable NNS Governance economics, metrics, reward, and collection contracts.

mod build;
mod continuation;
pub(in crate::nns) use continuation::{GovernanceCollectionState, governance_collection_accessors};
#[cfg(feature = "nns-host")]
pub(super) mod collection;
mod error;
mod model;
mod request;
mod source;
mod validation;
#[cfg(any(feature = "nns-host", feature = "canister"))]
mod wire;

#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_governance_economics_report, build_nns_governance_maturity_modulation_report,
    build_nns_governance_metrics_report, build_nns_governance_reward_event_report,
};
pub use build::{
    build_nns_governance_economics_report_with_source,
    build_nns_governance_maturity_modulation_report_with_source,
    build_nns_governance_metrics_report_with_source,
    build_nns_governance_reward_event_report_with_source,
};
#[cfg(feature = "nns-host")]
pub(super) use collection::{
    NnsGovernanceAttemptReadError, NnsGovernanceCacheMetadata,
    build_governance_cache_status_report, governance_refresh_bookkeeping,
    run_governance_snapshot_refresh, validate_governance_cache_metadata,
    write_running_governance_refresh_attempt,
};
#[cfg(feature = "nns-host")]
pub use collection::{
    NnsGovernanceCacheAdmission, NnsGovernanceCacheStatusReport, NnsGovernanceCacheSummary,
    NnsGovernanceRefreshReport, NnsGovernanceValidCacheSummary,
};
pub use error::NnsGovernanceError;
pub use model::{
    NnsGovernanceAnalyticsContext, NnsGovernanceCollection, NnsGovernanceCollectionStatus,
    NnsGovernanceDecimal, NnsGovernanceEconomics, NnsGovernanceEconomicsReport,
    NnsGovernanceExecutionAssurance, NnsGovernanceMaturityModulation,
    NnsGovernanceMaturityModulationReport, NnsGovernanceMetricBucket, NnsGovernanceMetrics,
    NnsGovernanceMetricsReport, NnsGovernanceNeuronSubsetMetrics, NnsGovernancePercentage,
    NnsGovernanceProposalId, NnsGovernanceReportContext, NnsGovernanceRewardEvent,
    NnsGovernanceRewardEventReport, NnsGovernanceSourceProvenance, NnsGovernanceVote,
    NnsNeuronsFundEconomics, NnsNeuronsFundMatchedFundingCurveCoefficients,
    NnsVotingPowerEconomics,
};
pub(in crate::nns) use model::{
    deserialize_code_label, native_code_classification, native_optional_code_classification,
};
pub use request::{NnsGovernanceRequest, NnsGovernanceSourceSelection};
#[cfg(all(feature = "canister", target_arch = "wasm32"))]
pub use source::CanisterNnsSource;
#[cfg(any(
    feature = "nns-host",
    all(feature = "canister", target_arch = "wasm32")
))]
pub(in crate::nns) use source::GovernanceQuery;
pub use source::{NnsGovernanceSource, NnsGovernanceSourceData, NnsGovernanceSourceFuture};
#[cfg(feature = "nns-host")]
pub(crate) use validation::enforce_mainnet_network as enforce_governance_mainnet_network;
pub(in crate::nns) use validation::validate_governance_continuation;
pub(crate) use validation::{
    advance_collection_counts, collection_status_after_page,
    validate_governance_collection_identity, validate_governance_collection_page_bounds,
    validate_governance_continuation_request, validate_governance_request,
    validate_governance_time_interval, validate_source_provenance,
};
#[cfg(any(feature = "nns-host", feature = "canister"))]
pub(in crate::nns) use wire::{
    GovernanceError, NnsGovernanceNeuronId, ProposalId as GovernanceProposalId,
};
/// Default replica endpoint used for direct NNS Governance reports.
pub const DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT: &str = "https://icp-api.io";

/// Maximum raw response bytes accepted from one direct Governance call.
pub const MAX_NNS_GOVERNANCE_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

const NNS_GOVERNANCE_REPORT_SCHEMA_VERSION: u32 = 1;

#[cfg(all(test, feature = "nns-host"))]
mod tests;
