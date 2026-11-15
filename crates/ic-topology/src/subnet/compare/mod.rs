//! Two selected placement candidates, without ranking or capacity inference.
use super::{
    distribution::{self, DistributionError, SubnetDistributionReport},
    metrics::{self, MetricsOutcome, SubnetMetricsError, SubnetMetricsReport},
};
use crate::{ReadError, mainnet};
use ic_platform_types::Principal;
use serde::Serialize;
use std::{future::Future, time::Duration};
use thiserror::Error;
use tokio::time::Instant;

/// Independently acquired evidence for two candidates in argument order.
#[derive(Debug, Serialize)]
pub struct SubnetComparisonReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// First candidate and its outcomes.
    pub left: Candidate,
    /// Second candidate and its outcomes.
    pub right: Candidate,
    /// Local start time, not certified time.
    pub started_at_unix_seconds: u64,
    /// Monotonic elapsed acquisition time.
    pub elapsed: Duration,
    /// Both distributions complete and both metrics reads successful.
    pub collection_complete: bool,
    /// Always false: these inputs are independent observations.
    pub atomic_snapshot: bool,
    /// Whether both requested Registry versions match; absent if either unavailable.
    pub registry_versions_agree: Option<bool>,
}
/// One selected candidate, including unavailable or partial evidence.
#[derive(Debug, Serialize)]
pub struct Candidate {
    /// Explicit requested identity.
    pub subnet_id: Principal,
    /// Configuration, relationships and their original coverage.
    pub distribution: DistributionOutcome,
    /// Independent certified resource observation.
    pub metrics: MetricsOutcome,
}
/// A distribution failure does not erase the other acquisitions.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DistributionOutcome {
    /// Acquired report, which can itself be partial.
    Read {
        /// Retained exactly once, including its configuration.
        report: Box<SubnetDistributionReport>,
    },
    /// Failure before a distribution report could be anchored.
    Failed {
        /// Original cause with the existing structured JSON projection.
        error: DistributionError,
    },
}
impl DistributionOutcome {
    /// Borrow an acquired distribution, including partial reports.
    #[must_use]
    pub fn report(&self) -> Option<&SubnetDistributionReport> {
        match self {
            Self::Read { report } => Some(report),
            Self::Failed { .. } => None,
        }
    }
}
impl Candidate {
    fn complete(&self) -> bool {
        self.distribution.report().is_some_and(|r| r.complete) && self.metrics.report().is_some()
    }
}
/// Failures before any candidate acquisition.
#[derive(Debug, Error)]
pub enum ComparisonError {
    /// Two distinct canonical identities are required.
    #[error("subnet comparison requires two different subnet IDs")]
    SameSubnet,
    /// Local start time could not be established.
    #[error(transparent)]
    Read(#[from] ReadError),
}
/// Collect left distribution/metrics, then right distribution/metrics.
///
/// Retains all component failures and continues independent reads. Each distribution
/// keeps its 60-second budget and each metrics read its 20-second budget: at most
/// 160 asynchronous seconds and 774 logical calls, plus bounded synchronous work.
/// No retry, wait, scan or background task. Cancellation starts no later work.
/// Requires Tokio IO/time. Separate versions and certificates are not a snapshot.
///
/// # Errors
/// Duplicate subnet IDs or initial clock failure, before network IO.
pub async fn fetch(
    left: Principal,
    right: Principal,
) -> Result<SubnetComparisonReport, ComparisonError> {
    collect(left, right, distribution::fetch, metrics::fetch).await
}
async fn collect<D, DF, M, MF>(
    left: Principal,
    right: Principal,
    mut distribution: D,
    mut metrics: M,
) -> Result<SubnetComparisonReport, ComparisonError>
where
    D: FnMut(Principal) -> DF,
    DF: Future<Output = Result<SubnetDistributionReport, DistributionError>>,
    M: FnMut(Principal) -> MF,
    MF: Future<Output = Result<SubnetMetricsReport, SubnetMetricsError>>,
{
    if left == right {
        return Err(ComparisonError::SameSubnet);
    }
    let started_at_unix_seconds = mainnet::observed_at()?;
    let started = Instant::now();
    let left = acquire(left, &mut distribution, &mut metrics).await;
    let right = acquire(right, &mut distribution, &mut metrics).await;
    let registry_versions_agree = left
        .distribution
        .report()
        .zip(right.distribution.report())
        .map(|(a, b)| a.requested_registry_version == b.requested_registry_version);
    Ok(SubnetComparisonReport {
        schema: 1,
        collection_complete: left.complete() && right.complete(),
        left,
        right,
        started_at_unix_seconds,
        elapsed: started.elapsed(),
        atomic_snapshot: false,
        registry_versions_agree,
    })
}
async fn acquire<D, DF, M, MF>(
    subnet_id: Principal,
    distribution: &mut D,
    metrics: &mut M,
) -> Candidate
where
    D: FnMut(Principal) -> DF,
    DF: Future<Output = Result<SubnetDistributionReport, DistributionError>>,
    M: FnMut(Principal) -> MF,
    MF: Future<Output = Result<SubnetMetricsReport, SubnetMetricsError>>,
{
    let distribution = match distribution(subnet_id).await {
        Ok(report) => DistributionOutcome::Read {
            report: Box::new(report),
        },
        Err(error) => DistributionOutcome::Failed { error },
    };
    let metrics = metrics(subnet_id).await.into();
    Candidate {
        subnet_id,
        distribution,
        metrics,
    }
}
#[cfg(test)]
mod tests;
