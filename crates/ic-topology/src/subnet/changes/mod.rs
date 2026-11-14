//! Compare selected Registry configurations without inferring maintenance events or uptime.

use std::time::Duration;

use ic_platform_types::Principal;
use serde::Serialize;
use thiserror::Error;

use super::configuration::{self, SubnetConfigurationError, SubnetConfigurationReport};

/// Two supplied observations and differences in their selected configuration fields.
#[derive(Debug, Serialize)]
pub struct SubnetChangesReport {
    /// Query-owned schema; remains 1 before 1.0.
    pub schema: u8,
    /// Requested wait for live sampling; None for a caller-supplied comparison.
    /// This is not the actual elapsed time or a certified observation interval.
    pub requested_interval_seconds: Option<u64>,
    /// First supplied configuration and all its evidence.
    pub before: SubnetConfigurationReport,
    /// Second supplied configuration and all its evidence.
    pub after: SubnetConfigurationReport,
    /// Members present only in the second record; not proof of node activation.
    pub registered_nodes_added: Vec<Principal>,
    /// Members present only in the first record; not proof of node removal execution.
    pub registered_nodes_removed: Vec<Principal>,
    /// Whether the raw configured replica version differs.
    pub replica_version_changed: bool,
    /// Whether the raw registered subnet type differs.
    pub subnet_type_changed: bool,
    /// Whether the configured halt flag differs.
    pub is_halted_changed: bool,
    /// Whether the configured halt-at-CUP flag differs.
    pub halt_at_cup_height_changed: bool,
}

/// Why two supplied configurations cannot support this comparison.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ComparisonRefusal {
    /// The observations identify different subnets.
    #[error("configuration observations identify different subnets")]
    DifferentSubnets,
    /// Selected membership is incomplete, excessive, duplicated or unordered.
    #[error("configuration membership must be complete, bounded and strictly ordered")]
    InvalidMembership,
    /// The second record claims an older mutation version; no rollback is inferred.
    #[error("configuration record version regressed; source ordering is uncertain")]
    VersionRegression,
    /// One mutation version claims different selected record content.
    #[error("configuration fields disagree at the same record mutation version")]
    ConflictingVersion,
    /// Query report schemas other than 1 are not admitted.
    #[error("unsupported configuration report schema")]
    Schema,
}

/// Failure preserves acquired observations without claiming completed change evidence.
#[derive(Debug, Error)]
pub enum SubnetChangesError {
    /// The sampling interval is outside the accepted range.
    #[error("configuration comparison interval must be between 1 and 60 seconds")]
    InvalidInterval,
    /// Initial configuration acquisition failed.
    #[error("first configuration observation failed: {0}")]
    First(#[source] SubnetConfigurationError),
    /// Second acquisition failed after retaining the first observation.
    #[error("second configuration observation failed after retaining the first: {source}")]
    Second {
        /// Successfully acquired first observation.
        before: Box<SubnetConfigurationReport>,
        /// Original typed source failure.
        #[source]
        source: SubnetConfigurationError,
    },
    /// Supplied observations failed comparison admission; both are retained.
    #[error("configuration comparison refused: {reason}")]
    Comparison {
        /// First supplied observation.
        before: Box<SubnetConfigurationReport>,
        /// Second supplied observation.
        after: Box<SubnetConfigurationReport>,
        /// Typed admission refusal.
        #[source]
        reason: ComparisonRefusal,
    },
}

/// Compare two configurations already acquired by the caller, without IO.
///
/// Preserves both sources, raw versions, timestamps and assurance flags. Reports
/// only differences in selected fields, not every intervening change, execution
/// of maintenance, running software or uptime. Unchanged selected fields can
/// accompany a newer mutation version because other record fields may change.
/// Does not reauthenticate evidence supplied by the caller.
///
/// # Errors
/// Refuses different subnets, unsupported schemas, incomplete/excessive or
/// noncanonical membership, regressing mutation versions and contradictory
/// selected fields at one mutation version. Retains both inputs on refusal.
pub fn compare(
    before: SubnetConfigurationReport,
    after: SubnetConfigurationReport,
) -> Result<SubnetChangesReport, SubnetChangesError> {
    let reason = if before.subnet_id != after.subnet_id {
        Some(ComparisonRefusal::DifferentSubnets)
    } else if before.schema != 1 || after.schema != 1 {
        Some(ComparisonRefusal::Schema)
    } else if [&before, &after].iter().any(|report| {
        !report.evidence.membership_complete
            || report.registered_node_ids.len() > crate::mainnet::REGISTRY_LIMITS.items
            || report
                .registered_node_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
    }) {
        Some(ComparisonRefusal::InvalidMembership)
    } else if after.evidence.record_version < before.evidence.record_version {
        Some(ComparisonRefusal::VersionRegression)
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(SubnetChangesError::Comparison {
            before: Box::new(before),
            after: Box::new(after),
            reason,
        });
    }
    let added = after
        .registered_node_ids
        .iter()
        .filter(|id| before.registered_node_ids.binary_search(id).is_err())
        .copied()
        .collect();
    let removed = before
        .registered_node_ids
        .iter()
        .filter(|id| after.registered_node_ids.binary_search(id).is_err())
        .copied()
        .collect();
    let report = SubnetChangesReport {
        schema: 1,
        requested_interval_seconds: None,
        registered_nodes_added: added,
        registered_nodes_removed: removed,
        replica_version_changed: before.replica_version_id != after.replica_version_id,
        subnet_type_changed: before.registered_subnet_type != after.registered_subnet_type,
        is_halted_changed: before.is_halted != after.is_halted,
        halt_at_cup_height_changed: before.halt_at_cup_height != after.halt_at_cup_height,
        before,
        after,
    };
    if report.before.evidence.record_version == report.after.evidence.record_version
        && report.has_selected_changes()
    {
        return Err(SubnetChangesError::Comparison {
            before: Box::new(report.before),
            after: Box::new(report.after),
            reason: ComparisonRefusal::ConflictingVersion,
        });
    }
    Ok(report)
}

impl SubnetChangesReport {
    /// Whether any selected field differs; false does not rule out other changes.
    #[must_use]
    pub fn has_selected_changes(&self) -> bool {
        !self.registered_nodes_added.is_empty()
            || !self.registered_nodes_removed.is_empty()
            || self.replica_version_changed
            || self.subnet_type_changed
            || self.is_halted_changed
            || self.halt_at_cup_height_changed
    }
}

/// Acquire and compare two configurations of one subnet, waiting 1–60 seconds.
///
/// The wait starts after the first read completes. Each reader retains its
/// 20-second asynchronous deadline and existing admission limits. Requires Tokio
/// IO/time. No persistent history, polling loop, retry or background task is added.
/// Source observation times remain local times, not certified event timestamps.
///
/// # Errors
/// Refuses invalid intervals before IO. Preserves source errors, the first
/// observation on second-read failure, and both inputs on comparison refusal.
pub async fn fetch(
    subnet_id: Principal,
    interval_seconds: u64,
) -> Result<SubnetChangesReport, SubnetChangesError> {
    sample(interval_seconds, || configuration::fetch(subnet_id)).await
}

async fn sample<F, Fut>(
    interval_seconds: u64,
    mut read: F,
) -> Result<SubnetChangesReport, SubnetChangesError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<SubnetConfigurationReport, SubnetConfigurationError>>,
{
    if !(1..=60).contains(&interval_seconds) {
        return Err(SubnetChangesError::InvalidInterval);
    }
    let before = read().await.map_err(SubnetChangesError::First)?;
    tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
    let after = match read().await {
        Ok(after) => after,
        Err(source) => {
            return Err(SubnetChangesError::Second {
                before: Box::new(before),
                source,
            });
        }
    };
    let mut report = compare(before, after)?;
    report.requested_interval_seconds = Some(interval_seconds);
    Ok(report)
}

#[cfg(test)]
mod tests;
