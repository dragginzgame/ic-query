//! Coordinated maintenance observations without an uptime or completion verdict.
use super::{
    activity::{self, MessageChange},
    changes::{self, ComparisonRefusal, SubnetChangesError, SubnetChangesReport},
    configuration::{self, SubnetConfigurationError, SubnetConfigurationReport},
    metrics::{self, SubnetMetricsError, SubnetMetricsReport},
};
use crate::{ReadError, mainnet};
use ic_platform_types::Principal;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::{future::Future, time::Duration};
use thiserror::Error;
use tokio::time::Instant;

pub use super::metrics::MetricsOutcome;

/// Four bounded observations and independent comparisons.
#[derive(Debug, Serialize)]
pub struct MaintenanceReport {
    /// Query-owned schema.
    pub schema: u8,
    /// Selected subnet.
    pub subnet_id: Principal,
    /// Requested wait, not a certified interval.
    pub interval_seconds: u64,
    /// Optional exact configured-version expectation; empty is meaningful.
    pub expected_version: Option<String>,
    /// Local start time.
    pub started_at_unix_seconds: u64,
    /// Monotonic elapsed time.
    pub elapsed: Duration,
    /// All four reads succeeded, independent of comparison refusal.
    pub collection_complete: bool,
    /// Always false.
    pub atomic_snapshot: bool,
    /// Configurations retained exactly once with comparison or failure.
    pub configuration: ConfigurationOutcome,
    /// First metrics observation or failure.
    pub metrics_before: MetricsOutcome,
    /// Second metrics observation or failure.
    pub metrics_after: MetricsOutcome,
    /// None when either metrics observation failed.
    pub message_change: Option<MessageChange>,
    /// First configured-version assessment.
    pub expected_before: VersionMatch,
    /// Second configured-version assessment.
    pub expected_after: VersionMatch,
}
/// Exact configured-string agreement, not running-software agreement.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionMatch {
    /// No expectation supplied.
    NotRequested,
    /// Exact string match.
    Matches,
    /// Different raw string.
    Differs,
    /// Configuration read unavailable.
    Unavailable,
}
/// Configuration evidence, retaining inputs on every later failure.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ConfigurationOutcome {
    /// Admitted comparison and both original inputs.
    Compared {
        /// Existing comparison report.
        report: Box<SubnetChangesReport>,
    },
    /// Second read failed.
    Failed {
        /// First observation.
        before: Box<SubnetConfigurationReport>,
        /// Original typed cause.
        #[serde(serialize_with = "configuration_error")]
        error: SubnetConfigurationError,
    },
    /// Both reads succeeded but comparison was refused.
    Refused {
        /// First observation.
        before: Box<SubnetConfigurationReport>,
        /// Second observation.
        after: Box<SubnetConfigurationReport>,
        /// Typed refusal.
        reason: ComparisonRefusal,
    },
}
impl ConfigurationOutcome {
    /// Borrow the original observations without copying evidence.
    #[must_use]
    pub fn observations(
        &self,
    ) -> (
        &SubnetConfigurationReport,
        Option<&SubnetConfigurationReport>,
    ) {
        match self {
            Self::Compared { report } => (&report.before, Some(&report.after)),
            Self::Failed { before, .. } => (before, None),
            Self::Refused { before, after, .. } => (before, Some(after)),
        }
    }
}
impl MaintenanceReport {
    /// Complete collection and available comparisons, not successful maintenance.
    #[must_use]
    pub fn comparisons_available(&self) -> bool {
        self.collection_complete
            && matches!(self.configuration, ConfigurationOutcome::Compared { .. })
            && self.message_change.is_some()
    }
}
/// Failures before a report can be anchored.
#[derive(Debug, Error)]
pub enum MaintenanceError {
    /// Invalid interval refused before IO.
    #[error("maintenance interval must be between 1 and 60 seconds")]
    InvalidInterval,
    /// Excessive expectation refused before IO.
    #[error("expected version exceeds 256 UTF-8 bytes")]
    VersionLimit,
    /// Initial clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// Initial configuration failed; no later reads started.
    #[error("initial maintenance configuration failed: {0}")]
    Initial(#[source] SubnetConfigurationError),
}
/// Observe configuration then metrics twice, with one wait between rounds.
///
/// Each existing reader keeps its 20-second deadline and admission bounds.
/// At most four reads and one 1–60-second wait; no retries or spawned tasks.
/// Later failures remain in the returned report. Cancellation starts no later work.
/// Requires Tokio IO/time. No running-version or uptime evidence is inferred.
///
/// # Errors
/// Invalid parameters, clock failure or the first configuration failure.
pub async fn fetch(
    subnet_id: Principal,
    interval_seconds: u64,
    expected_version: Option<String>,
) -> Result<MaintenanceReport, MaintenanceError> {
    collect(
        subnet_id,
        interval_seconds,
        expected_version,
        || configuration::fetch(subnet_id),
        || metrics::fetch(subnet_id),
    )
    .await
}
async fn collect<C, CF, M, MF>(
    subnet_id: Principal,
    interval_seconds: u64,
    expected_version: Option<String>,
    mut configuration: C,
    mut metrics: M,
) -> Result<MaintenanceReport, MaintenanceError>
where
    C: FnMut() -> CF,
    CF: Future<Output = Result<SubnetConfigurationReport, SubnetConfigurationError>>,
    M: FnMut() -> MF,
    MF: Future<Output = Result<SubnetMetricsReport, SubnetMetricsError>>,
{
    if !(1..=60).contains(&interval_seconds) {
        return Err(MaintenanceError::InvalidInterval);
    }
    if expected_version.as_ref().is_some_and(|s| s.len() > 256) {
        return Err(MaintenanceError::VersionLimit);
    }
    let start = Instant::now();
    let started_at_unix_seconds = mainnet::observed_at()?;
    let before = configuration().await.map_err(MaintenanceError::Initial)?;
    let metrics_before = MetricsOutcome::from(metrics().await);
    tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
    let after = configuration().await;
    let metrics_after = MetricsOutcome::from(metrics().await);
    let expected_before = assess(Some(&before), expected_version.as_deref());
    let expected_after = assess(after.as_ref().ok(), expected_version.as_deref());
    let collection_complete =
        after.is_ok() && metrics_before.report().is_some() && metrics_after.report().is_some();
    let configuration = match after {
        Err(error) => ConfigurationOutcome::Failed {
            before: Box::new(before),
            error,
        },
        Ok(after) => match changes::compare(before, after) {
            Ok(mut report) => {
                report.requested_interval_seconds = Some(interval_seconds);
                ConfigurationOutcome::Compared {
                    report: Box::new(report),
                }
            }
            Err(SubnetChangesError::Comparison {
                before,
                after,
                reason,
            }) => ConfigurationOutcome::Refused {
                before,
                after,
                reason,
            },
            Err(_) => unreachable!("pure comparison only returns comparison refusals"),
        },
    };
    let message_change = metrics_before
        .report()
        .zip(metrics_after.report())
        .map(|(a, b)| {
            activity::message_change(a.update_transactions_total, b.update_transactions_total)
        });
    Ok(MaintenanceReport {
        schema: 1,
        subnet_id,
        interval_seconds,
        expected_version,
        started_at_unix_seconds,
        elapsed: start.elapsed(),
        collection_complete,
        atomic_snapshot: false,
        configuration,
        metrics_before,
        metrics_after,
        message_change,
        expected_before,
        expected_after,
    })
}
fn assess(report: Option<&SubnetConfigurationReport>, expected: Option<&str>) -> VersionMatch {
    match (report, expected) {
        (None, _) => VersionMatch::Unavailable,
        (Some(_), None) => VersionMatch::NotRequested,
        (Some(r), Some(e)) if r.replica_version_id == e => VersionMatch::Matches,
        _ => VersionMatch::Differs,
    }
}
fn read_category(error: &ReadError) -> &'static str {
    match error {
        ReadError::Transport(_) => "transport",
        ReadError::Deadline => "read_deadline",
        ReadError::Clock(_) => "clock",
    }
}
fn configuration_error<S: Serializer>(
    error: &SubnetConfigurationError,
    s: S,
) -> Result<S::Ok, S::Error> {
    use SubnetConfigurationError as E;
    let category = match error {
        E::Read(e) => read_category(e),
        E::SourceIdentity(_) => "source_identity",
        E::Codec(_) => "codec",
        E::RegistryRejected { .. } => "registry_rejected",
        E::MissingContent => "missing_content",
        E::InvalidIdentity { .. } => "invalid_identity",
        E::DuplicateIdentity => "duplicate_identity",
        E::VersionLimit => "version_limit",
    };
    let mut value = s.serialize_struct("ConfigurationFailure", 3)?;
    value.serialize_field("category", category)?;
    value.serialize_field(
        "registry_code",
        &match error {
            E::RegistryRejected { code, .. } => Some(*code),
            _ => None,
        },
    )?;
    value.serialize_field(
        "identity_index",
        &match error {
            E::InvalidIdentity { index, .. } => Some(*index),
            _ => None,
        },
    )?;
    value.end()
}
#[cfg(test)]
mod tests;
