//! Certified resource observations for one subnet, without inferred capacity or rates.

#[cfg(test)]
mod tests;

use ic_platform_types::Principal;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::{ReadError, mainnet};

/// One subnet's four certified resource values, not an uptime measurement.
#[derive(Debug, Serialize)]
pub struct SubnetMetricsReport {
    /// Query-owned schema; remains 1 before 1.0.
    pub schema: u8,
    /// Subnet whose authority and metrics path Platform verified.
    pub subnet_id: Principal,
    /// Current canister count; this may decrease.
    pub num_canisters: u64,
    /// Current canister state bytes, refreshed by the source every ten blocks.
    pub canister_state_bytes: u64,
    /// Raw nominal-cycle accounting total, which may decrease after refunds or splits.
    /// JSON uses a decimal string to preserve all 128 bits in ordinary JSON consumers.
    #[serde(serialize_with = "decimal_cycles")]
    pub consumed_cycles_total: u128,
    /// Cumulative messages executed in replicated mode, not a transaction rate.
    pub update_transactions_total: u64,
    /// Source and verification evidence for this observation.
    pub evidence: SubnetMetricsEvidence,
}

/// Source evidence for one read, with no implied Registry version or time series.
#[derive(Debug, Serialize)]
pub struct SubnetMetricsEvidence {
    /// Fixed mainnet HTTPS endpoint.
    pub endpoint: &'static str,
    /// Subnet-scoped HTTP read-state path.
    pub source_path: String,
    /// Exact certified tree leaf selected by Platform.
    pub state_tree_path: String,
    /// Local completion time in Unix seconds; not the certificate timestamp.
    pub observed_at_unix_seconds: u64,
    /// Platform verified the certificate, subnet authority and its timestamp policy.
    pub state_certificate_verified: bool,
}

/// A failed observation is unavailable evidence, not proof of subnet downtime.
#[derive(Debug, Error)]
pub enum SubnetMetricsError {
    /// Transport, certificate, decoding, missing-leaf, deadline or clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
}

/// Read one subnet's certified metrics using the published Platform native client.
///
/// Uses the shared mainnet trust policy, 20-second deadline and 1 MiB HTTP ceiling.
/// Requires a Tokio runtime with IO and time. No polling, retries, caches, rates
/// or available-capacity estimates are introduced. Zero values remain valid.
///
/// # Errors
/// Refuses an invalid identity, failed read or verification, absent/malformed
/// metrics, deadline expiry or unavailable local observation time. Missing values
/// never become zero defaults. Platform owns external decoding and verification.
pub async fn fetch(subnet_id: Principal) -> Result<SubnetMetricsReport, SubnetMetricsError> {
    let client = mainnet::client()?;
    let observation = mainnet::subnet(&client, subnet_id).await?;
    let metrics = observation.metrics().ok_or_else(|| {
        ReadError::Transport(Box::new(ic_platform::native::ReadError::MissingPath))
    })?;

    Ok(SubnetMetricsReport {
        schema: 1,
        subnet_id,
        num_canisters: metrics.num_canisters,
        canister_state_bytes: metrics.canister_state_bytes,
        consumed_cycles_total: metrics.consumed_cycles_total,
        update_transactions_total: metrics.update_transactions_total,
        evidence: SubnetMetricsEvidence {
            endpoint: mainnet::ENDPOINT,
            source_path: format!("/api/v3/subnet/{subnet_id}/read_state"),
            state_tree_path: format!("/subnet/{subnet_id}/metrics"),
            observed_at_unix_seconds: mainnet::observed_at()?,
            state_certificate_verified: true,
        },
    })
}

fn decimal_cycles<S: Serializer>(value: &u128, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}

/// Independent metrics acquisition, retaining failures separately from zero values.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MetricsOutcome {
    /// Successful certified observation.
    Read {
        /// Full raw evidence.
        report: SubnetMetricsReport,
    },
    /// Unavailable evidence, not a health verdict.
    Failed {
        /// Original typed cause; JSON exposes a stable category.
        #[serde(serialize_with = "metrics_error")]
        error: SubnetMetricsError,
    },
}

impl MetricsOutcome {
    /// Borrow successfully acquired evidence.
    #[must_use]
    pub fn report(&self) -> Option<&SubnetMetricsReport> {
        match self {
            Self::Read { report } => Some(report),
            Self::Failed { .. } => None,
        }
    }
}

impl From<Result<SubnetMetricsReport, SubnetMetricsError>> for MetricsOutcome {
    fn from(value: Result<SubnetMetricsReport, SubnetMetricsError>) -> Self {
        match value {
            Ok(report) => Self::Read { report },
            Err(error) => Self::Failed { error },
        }
    }
}

fn metrics_error<S: Serializer>(error: &SubnetMetricsError, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(match error {
        SubnetMetricsError::Read(ReadError::Transport(_)) => "transport",
        SubnetMetricsError::Read(ReadError::Deadline) => "read_deadline",
        SubnetMetricsError::Read(ReadError::Clock(_) | ReadError::ClockRange) => "clock",
    })
}
