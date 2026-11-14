//! Certified resource observations for one subnet, without inferred capacity or rates.

use ic_platform_types::Principal;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::{ReadError, mainnet};

/// One subnet's four certified resource values, not an uptime measurement.
#[derive(Debug, Serialize)]
pub struct SubnetMetricsReport {
    /// Query-owned schema; remains 1 before 1.0.
    pub schema: u8,
    /// Subnet whose authority and metrics path the Agent verified.
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
    /// Exact certified tree leaf selected by the Agent.
    pub state_tree_path: String,
    /// Local completion time in Unix seconds; not the certificate timestamp.
    pub observed_at_unix_seconds: u64,
    /// Agent verified the certificate, subnet authority and its timestamp policy.
    pub state_certificate_verified: bool,
}

/// A failed observation is unavailable evidence, not proof of subnet downtime.
#[derive(Debug, Error)]
pub enum SubnetMetricsError {
    /// Transport, certificate, decoding, missing-leaf, deadline or clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// Selected identity could not cross the Agent boundary.
    #[error("invalid subnet identity: {0}")]
    SourceIdentity(String),
}

/// Read one subnet's certified metrics using the published Platform Agent.
///
/// Uses the shared mainnet trust policy, 20-second deadline and 1 MiB HTTP ceiling.
/// Requires a Tokio runtime with IO and time. No polling, retries, caches, rates
/// or available-capacity estimates are introduced. Zero values remain valid.
///
/// # Errors
/// Refuses an invalid identity, failed read or verification, absent/malformed
/// metrics, deadline expiry or unavailable local observation time. Missing values
/// never become zero defaults. Agent owns external decoding and verification.
pub async fn fetch(subnet_id: Principal) -> Result<SubnetMetricsReport, SubnetMetricsError> {
    let agent = mainnet::agent()?;
    let external_id = subnet_id
        .as_slice()
        .try_into()
        .map_err(|error| SubnetMetricsError::SourceIdentity(format!("{error}")))?;
    let metrics = mainnet::read(agent.read_state_subnet_metrics(external_id)).await?;
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

#[cfg(test)]
mod tests;
