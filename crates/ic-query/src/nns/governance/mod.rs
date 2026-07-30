//! Reusable NNS Governance economics, metrics, reward, and collection contracts.

#[cfg(feature = "host")]
mod build;
#[cfg(feature = "host")]
pub(super) mod collection;
mod model;
#[cfg(feature = "host")]
mod source;
mod text;
#[cfg(feature = "host")]
mod wire;

#[cfg(feature = "host")]
use crate::{nns::NnsGovernanceQueryError, runtime::RuntimeError};
#[cfg(feature = "host")]
use thiserror::Error as ThisError;

#[cfg(feature = "host")]
pub use build::{
    build_nns_governance_economics_report, build_nns_governance_economics_report_with_source,
    build_nns_governance_maturity_modulation_report,
    build_nns_governance_maturity_modulation_report_with_source,
    build_nns_governance_metrics_report, build_nns_governance_metrics_report_with_source,
    build_nns_governance_reward_event_report, build_nns_governance_reward_event_report_with_source,
};
#[cfg(feature = "host")]
pub(super) use collection::{
    NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS, NnsGovernanceCacheMetadata,
    governance_refresh_attempt_status, governance_refresh_progress,
    mainnet_governance_cache_metadata, validate_governance_cache_metadata,
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
#[cfg(feature = "host")]
pub use source::NnsGovernanceSource;
pub use text::{
    nns_governance_economics_report_text, nns_governance_maturity_modulation_report_text,
    nns_governance_metrics_report_text, nns_governance_reward_event_report_text,
};

/// Default replica endpoint used for direct NNS Governance reports.
pub const DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT: &str = "https://icp-api.io";

#[cfg(feature = "host")]
const NNS_GOVERNANCE_REPORT_SCHEMA_VERSION: u32 = 1;

///
/// NnsGovernanceHostError
///
/// Error returned while querying direct NNS Governance reports.
///

#[cfg(feature = "host")]
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

#[cfg(feature = "host")]
fn enforce_mainnet_network(network: &str) -> Result<(), NnsGovernanceHostError> {
    crate::network::enforce_mainnet_network_with(network, |network| {
        NnsGovernanceHostError::UnsupportedNetwork { network }
    })
}

#[cfg(feature = "host")]
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

#[cfg(all(test, feature = "host"))]
mod tests;
