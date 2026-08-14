//! Reusable NNS Governance economics, metrics, reward, and collection contracts.

mod build;
#[cfg(feature = "nns-host")]
pub(super) mod collection;
mod error;
mod model;
mod request;
mod source;
mod text;
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
    NnsGovernanceAttemptReadError, NnsGovernanceCacheMetadata, mainnet_governance_cache_metadata,
    read_governance_refresh_attempt_status, validate_governance_cache_metadata,
    write_complete_governance_refresh_attempt, write_failed_governance_refresh_attempt,
    write_running_governance_refresh_attempt, write_starting_governance_refresh_attempt,
};
pub use error::NnsGovernanceError;
#[cfg(feature = "nns-host")]
pub use error::NnsGovernanceHostError;
pub use model::{
    NnsGovernanceDecimal, NnsGovernanceEconomics, NnsGovernanceEconomicsReport,
    NnsGovernanceExecutionAssurance, NnsGovernanceMaturityModulation,
    NnsGovernanceMaturityModulationReport, NnsGovernanceMetricBucket, NnsGovernanceMetrics,
    NnsGovernanceMetricsReport, NnsGovernanceNeuronSubsetMetrics, NnsGovernancePercentage,
    NnsGovernanceProposalId, NnsGovernanceReportContext, NnsGovernanceRewardEvent,
    NnsGovernanceRewardEventReport, NnsGovernanceSourceProvenance, NnsNeuronsFundEconomics,
    NnsNeuronsFundMatchedFundingCurveCoefficients, NnsVotingPowerEconomics,
};
pub use request::{NnsGovernanceRequest, NnsGovernanceSourceSelection};
#[cfg(all(feature = "canister", target_arch = "wasm32"))]
pub use source::CanisterNnsSource;
pub use source::{NnsGovernanceSource, NnsGovernanceSourceData, NnsGovernanceSourceFuture};
pub use text::{
    nns_governance_economics_report_text, nns_governance_maturity_modulation_report_text,
    nns_governance_metrics_report_text, nns_governance_reward_event_report_text,
};
/// Default replica endpoint used for direct NNS Governance reports.
pub const DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT: &str = "https://icp-api.io";

/// Maximum raw response bytes accepted from one direct Governance call.
pub const MAX_NNS_GOVERNANCE_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

const NNS_GOVERNANCE_REPORT_SCHEMA_VERSION: u32 = 1;

#[cfg(all(test, feature = "nns-host"))]
mod tests;
