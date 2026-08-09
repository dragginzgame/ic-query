//! Reusable NNS Governance economics, metrics, reward, and collection contracts.

#[cfg(feature = "nns-host")]
mod build;
#[cfg(feature = "nns-host")]
pub(super) mod collection;
mod model;
#[cfg(feature = "nns-host")]
mod source;
mod text;
#[cfg(feature = "nns-host")]
mod wire;

#[cfg(feature = "nns-host")]
use crate::{nns::NnsGovernanceQueryError, runtime::RuntimeError};
#[cfg(feature = "nns-host")]
use thiserror::Error as ThisError;

#[cfg(feature = "nns-host")]
pub use build::{
    build_nns_governance_economics_report, build_nns_governance_economics_report_with_source,
    build_nns_governance_maturity_modulation_report,
    build_nns_governance_maturity_modulation_report_with_source,
    build_nns_governance_metrics_report, build_nns_governance_metrics_report_with_source,
    build_nns_governance_reward_event_report, build_nns_governance_reward_event_report_with_source,
};
#[cfg(feature = "nns-host")]
pub(super) use collection::{
    NnsGovernanceAttemptReadError, NnsGovernanceCacheMetadata, mainnet_governance_cache_metadata,
    read_governance_refresh_attempt_status, validate_governance_cache_metadata,
    write_complete_governance_refresh_attempt, write_failed_governance_refresh_attempt,
    write_running_governance_refresh_attempt, write_starting_governance_refresh_attempt,
};
pub use model::{
    NnsGovernanceDecimal, NnsGovernanceEconomics, NnsGovernanceEconomicsReport,
    NnsGovernanceMaturityModulation, NnsGovernanceMaturityModulationReport,
    NnsGovernanceMetricBucket, NnsGovernanceMetrics, NnsGovernanceMetricsReport,
    NnsGovernanceNeuronSubsetMetrics, NnsGovernancePercentage, NnsGovernanceProposalId,
    NnsGovernanceReportContext, NnsGovernanceRewardEvent, NnsGovernanceRewardEventReport,
    NnsNeuronsFundEconomics, NnsNeuronsFundMatchedFundingCurveCoefficients,
    NnsVotingPowerEconomics,
};
#[cfg(feature = "nns-host")]
pub use source::NnsGovernanceSource;
pub use text::{
    nns_governance_economics_report_text, nns_governance_maturity_modulation_report_text,
    nns_governance_metrics_report_text, nns_governance_reward_event_report_text,
};

/// Default replica endpoint used for direct NNS Governance reports.
pub const DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT: &str = "https://icp-api.io";

#[cfg(feature = "nns-host")]
const NNS_GOVERNANCE_REPORT_SCHEMA_VERSION: u32 = 1;

///
/// NnsGovernanceHostError
///
/// Error returned while querying direct NNS Governance reports.
///

#[cfg(feature = "nns-host")]
#[derive(Debug, ThisError)]
pub enum NnsGovernanceHostError {
    /// The requested network is not the supported mainnet identity.
    #[error(
        "`icq nns governance` supports only the mainnet `ic` network\n\nThese reports query the Internet Computer mainnet Governance canister.\n\nTry:\n  icq --network ic nns governance economics"
    )]
    UnsupportedNetwork {
        /// Rejected network identity.
        network: String,
    },

    /// Shared NNS Governance transport failed.
    #[error(transparent)]
    GovernanceQuery(#[from] NnsGovernanceQueryError),

    /// Governance returned its typed application-level error.
    #[error("NNS Governance rejected the metrics query with code {error_type}: {message}")]
    Governance {
        /// Raw Governance error type.
        error_type: i32,
        /// Governance error message.
        message: String,
    },

    /// Governance returned a non-finite metric that JSON cannot preserve.
    #[error("NNS Governance metric {field} bucket {key} has non-finite value {value}")]
    InvalidMetrics {
        /// Native Governance metric field.
        field: &'static str,
        /// Raw metric bucket key.
        key: u64,
        /// Rejected non-finite bucket value.
        value: f64,
    },

    /// The synchronous host runtime could not execute the live query.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

#[cfg(feature = "nns-host")]
fn enforce_mainnet_network(network: &str) -> Result<(), NnsGovernanceHostError> {
    crate::network::enforce_mainnet_network_with(network, |network| {
        NnsGovernanceHostError::UnsupportedNetwork { network }
    })
}

#[cfg(feature = "nns-host")]
fn validate_governance_metrics(
    metrics: &NnsGovernanceMetrics,
) -> Result<(), NnsGovernanceHostError> {
    for (field, buckets) in [
        (
            "not_dissolving_neurons_e8s_buckets",
            metrics.not_dissolving_neurons_e8s_buckets.as_slice(),
        ),
        (
            "dissolving_neurons_staked_maturity_e8s_equivalent_buckets",
            metrics
                .dissolving_neurons_staked_maturity_e8s_equivalent_buckets
                .as_slice(),
        ),
        (
            "not_dissolving_neurons_e8s_buckets_ect",
            metrics.not_dissolving_neurons_e8s_buckets_ect.as_slice(),
        ),
        (
            "dissolving_neurons_e8s_buckets_seed",
            metrics.dissolving_neurons_e8s_buckets_seed.as_slice(),
        ),
        (
            "not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets",
            metrics
                .not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets
                .as_slice(),
        ),
        (
            "dissolving_neurons_e8s_buckets_ect",
            metrics.dissolving_neurons_e8s_buckets_ect.as_slice(),
        ),
        (
            "dissolving_neurons_e8s_buckets",
            metrics.dissolving_neurons_e8s_buckets.as_slice(),
        ),
        (
            "not_dissolving_neurons_e8s_buckets_seed",
            metrics.not_dissolving_neurons_e8s_buckets_seed.as_slice(),
        ),
    ] {
        if let Some(bucket) = buckets.iter().find(|bucket| !bucket.value.is_finite()) {
            return Err(NnsGovernanceHostError::InvalidMetrics {
                field,
                key: bucket.key,
                value: bucket.value,
            });
        }
    }
    Ok(())
}

#[cfg(all(test, feature = "nns-host"))]
mod tests;
