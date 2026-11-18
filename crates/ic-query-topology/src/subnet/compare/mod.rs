//! Two selected placement candidates, without ranking or capacity inference.

#[cfg(test)]
mod tests;

use super::{
    distribution::{self, DistributionError, SubnetDistributionReport},
    metrics::{self, MetricsOutcome, SubnetMetricsError, SubnetMetricsReport},
};
use crate::{ReadError, mainnet};
use ic_platform_types::Principal;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::{future::Future, time::Duration};
use thiserror::Error;
use tokio::time::Instant;

/// Independently acquired evidence for two candidates in argument order.
#[derive(Debug)]
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

/// One declared identity observed in both selected subnet distributions.
#[derive(Debug, Serialize)]
pub struct SharedGroup<T> {
    /// Canonical provider principal or admitted lowercase data-center identity.
    pub identity: T,
    /// Selected left-subnet nodes assigned to this identity.
    pub left_nodes: usize,
    /// Selected right-subnet nodes assigned to this identity.
    pub right_nodes: usize,
}

/// Observed intersection with independent missing-relationship coverage.
#[derive(Debug, Serialize)]
pub struct RelationshipOverlap<T> {
    /// Shared identities, in canonical order; empty does not imply independence.
    pub shared: Vec<SharedGroup<T>>,
    /// Left selected nodes with unresolved relationships of this kind.
    pub left_unknown_nodes: usize,
    /// Right selected nodes with unresolved relationships of this kind.
    pub right_unknown_nodes: usize,
    /// Both sides resolved this relationship for all selected nodes.
    /// This is coverage of declarations, not proof of independent failure domains.
    pub coverage_complete: bool,
}

/// Shared registered infrastructure across independently acquired distributions.
#[derive(Debug, Serialize)]
pub struct InfrastructureOverlap {
    /// Provider identities joined through node operators.
    pub providers: RelationshipOverlap<Principal>,
    /// Data-center identities with successfully admitted location records.
    pub data_centers: RelationshipOverlap<String>,
}

impl SubnetComparisonReport {
    /// Compare the existing distribution summaries without IO or new claims.
    ///
    /// Returns None when either distribution is unavailable. Partial distributions
    /// still expose observed intersections and unknown counts independently for
    /// providers and locations. Versions/times remain those of the original inputs;
    /// shared identities do not establish simultaneous assignment or failure domains.
    /// Uses admitted summaries from `fetch`; does not revalidate caller-built reports.
    #[must_use]
    pub fn infrastructure_overlap(&self) -> Option<InfrastructureOverlap> {
        let left = &self.left.distribution.report()?.summary;
        let right = &self.right.distribution.report()?.summary;
        Some(InfrastructureOverlap {
            providers: overlap(
                &left.providers,
                &right.providers,
                left.provider_unknown,
                right.provider_unknown,
            ),
            data_centers: overlap(
                &left.data_centers,
                &right.data_centers,
                left.location_unknown,
                right.location_unknown,
            ),
        })
    }
}

fn overlap<T: Ord + Clone>(
    left: &[super::distribution::GroupCount<T>],
    right: &[super::distribution::GroupCount<T>],
    left_unknown_nodes: usize,
    right_unknown_nodes: usize,
) -> RelationshipOverlap<T> {
    // Source summaries are canonical. Merge their sorted groups without another
    // inventory/index, retaining per-side node counts rather than summing them.
    let mut shared = Vec::new();
    let (mut a, mut b) = (0, 0);
    while a < left.len() && b < right.len() {
        match left[a].value.cmp(&right[b].value) {
            std::cmp::Ordering::Less => a += 1,
            std::cmp::Ordering::Greater => b += 1,
            std::cmp::Ordering::Equal => {
                shared.push(SharedGroup {
                    identity: left[a].value.clone(),
                    left_nodes: left[a].nodes,
                    right_nodes: right[b].nodes,
                });
                a += 1;
                b += 1;
            }
        }
    }
    RelationshipOverlap {
        shared,
        left_unknown_nodes,
        right_unknown_nodes,
        coverage_complete: left_unknown_nodes == 0 && right_unknown_nodes == 0,
    }
}

impl Serialize for SubnetComparisonReport {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Derive from the retained inputs at output time: no second mutable copy
        // of overlap evidence and no new required field for Rust constructors.
        let mut value = serializer.serialize_struct("SubnetComparisonReport", 9)?;
        value.serialize_field("schema", &self.schema)?;
        value.serialize_field("left", &self.left)?;
        value.serialize_field("right", &self.right)?;
        value.serialize_field("started_at_unix_seconds", &self.started_at_unix_seconds)?;
        value.serialize_field("elapsed", &self.elapsed)?;
        value.serialize_field("collection_complete", &self.collection_complete)?;
        value.serialize_field("atomic_snapshot", &self.atomic_snapshot)?;
        value.serialize_field("registry_versions_agree", &self.registry_versions_agree)?;
        value.serialize_field("infrastructure_overlap", &self.infrastructure_overlap())?;
        value.end()
    }
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

/// Optional settings composition; the original comparison remains intact once.
#[derive(Debug, Serialize)]
pub struct SettingsComparisonReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// Original comparison and its independent evidence.
    pub comparison: SubnetComparisonReport,
    /// Left settings, read after the comparison at an independent Registry version.
    pub left_settings: super::settings::SettingsOutcome,
    /// Right settings, attempted even when left settings failed.
    pub right_settings: super::settings::SettingsOutcome,
    /// Total monotonic duration including the original comparison.
    pub elapsed: Duration,
    /// Original comparison complete and both settings reads successful.
    pub collection_complete: bool,
}

/// Compare candidates and then read each candidate's registered settings.
///
/// Keeps the original comparison contract, followed by two 20-second settings
/// reads. At most 200 asynchronous seconds and 776 logical calls, plus bounded
/// processing/scheduling. Retains partial evidence, without retries or tasks.
/// Settings versions are independent of distribution versions; no snapshot,
/// effective defaults, capacity or attestation claim follows. Requires Tokio IO/time.
///
/// # Errors
/// Duplicate subnet IDs or failure to establish the initial comparison time.
pub async fn fetch_with_settings(
    left: Principal,
    right: Principal,
) -> Result<SettingsComparisonReport, ComparisonError> {
    collect_with_settings(left, right, fetch, super::settings::fetch).await
}

async fn collect_with_settings<C, CF, S, SF>(
    left: Principal,
    right: Principal,
    comparison: C,
    mut settings: S,
) -> Result<SettingsComparisonReport, ComparisonError>
where
    C: FnOnce(Principal, Principal) -> CF,
    CF: Future<Output = Result<SubnetComparisonReport, ComparisonError>>,
    S: FnMut(Principal) -> SF,
    SF: Future<
        Output = Result<
            super::settings::SubnetSettingsReport,
            super::settings::SubnetSettingsError,
        >,
    >,
{
    if left == right {
        return Err(ComparisonError::SameSubnet);
    }
    let started = Instant::now();
    let comparison = comparison(left, right).await?;
    let left_settings: super::settings::SettingsOutcome = settings(left).await.into();
    let right_settings: super::settings::SettingsOutcome = settings(right).await.into();
    Ok(SettingsComparisonReport {
        schema: 1,
        collection_complete: comparison.collection_complete
            && left_settings.report().is_some()
            && right_settings.report().is_some(),
        comparison,
        left_settings,
        right_settings,
        elapsed: started.elapsed(),
    })
}
