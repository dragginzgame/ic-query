//! Compare registered machine identifiers and configured software, not TEE readiness.

#[cfg(test)]
mod tests;

use std::time::Duration;

use ic_platform_types::Principal;
use serde::Serialize;
use thiserror::Error;

use super::{NodeError, NodeReport};
use crate::mainnet;

/// Two selected node observations and differences in their admitted raw fields.
#[derive(Debug, Serialize)]
pub struct NodeChangesReport {
    /// Query-owned schema, remaining 1 before 1.0.
    pub schema: u8,
    /// Requested wait for live reads; None for supplied observations.
    /// Not an actual elapsed or certified event interval.
    pub requested_interval_seconds: Option<u64>,
    /// First observation, retaining all original fields and evidence.
    pub before: NodeReport,
    /// Second observation, retaining all original fields and evidence.
    pub after: NodeReport,
    /// Whether the registered operator identity changed.
    pub operator_changed: bool,
    /// Whether the optional raw chip bytes changed, including optional presence.
    pub chip_id_changed: bool,
    /// Whether the optional raw configured HostOS changed, including presence.
    pub hostos_version_changed: bool,
}

impl NodeChangesReport {
    /// True if any selected registration field differs; not a hardware verdict.
    #[must_use]
    pub fn has_selected_changes(&self) -> bool {
        self.operator_changed || self.chip_id_changed || self.hostos_version_changed
    }
}

/// Why supplied node observations cannot support a registration comparison.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ComparisonRefusal {
    /// Inputs identify different nodes.
    #[error("node observations identify different nodes")]
    DifferentNodes,
    /// Only Query schema 1 is supported.
    #[error("unsupported node report schema")]
    Schema,
    /// Empty operator or excessive optional field bytes violate reader admission.
    #[error("node observations violate operator or field-size admission")]
    InvalidRecord,
    /// Evidence does not identify the selected mainnet Registry node key/method.
    #[error("node observations have an incompatible Registry source context")]
    SourceContext,
    /// The second mutation version is older; no rollback is inferred.
    #[error("node record version regressed; source ordering is uncertain")]
    VersionRegression,
    /// One record version claims different selected content.
    #[error("node fields disagree at the same record mutation version")]
    ConflictingVersion,
}

/// Failures preserve acquired observations without claiming completed comparison.
#[derive(Debug, Error)]
pub enum NodeChangesError {
    /// Wait outside the accepted range, refused before IO.
    #[error("node comparison interval must be between 1 and 60 seconds")]
    InvalidInterval,
    /// Initial node acquisition failed.
    #[error("first node observation failed: {0}")]
    First(#[source] NodeError),
    /// Second acquisition failed; first observation is retained.
    #[error("second node observation failed after retaining the first: {source}")]
    Second {
        /// Acquired first report with all evidence.
        before: Box<NodeReport>,
        /// Original typed source cause.
        #[source]
        source: NodeError,
    },
    /// Comparison admission failed; both supplied reports are retained.
    #[error("node comparison refused: {reason}")]
    Comparison {
        /// First supplied report.
        before: Box<NodeReport>,
        /// Second supplied report.
        after: Box<NodeReport>,
        /// Typed refusal, without interpreting source text as terminal output.
        #[source]
        reason: ComparisonRefusal,
    },
}

/// Compare two supplied node reports without IO or reauthentication.
///
/// Preserves raw optional presence and both evidence records, including assurance
/// flags and possibly regressing local timestamps. Chip registration is neither
/// verified attestation nor proof that new hardware came online. Configured
/// HostOS is not measured running software; equal observations are not history.
///
/// # Errors
/// Refuses different nodes, unsupported schemas, inadmissible fields or source
/// context, regressing record versions and conflicting selected content at one
/// mutation version. Both original inputs remain available on every refusal.
pub fn compare(
    before: NodeReport,
    after: NodeReport,
) -> Result<NodeChangesReport, NodeChangesError> {
    let reason = if before.node_id != after.node_id {
        Some(ComparisonRefusal::DifferentNodes)
    } else if before.schema != 1 || after.schema != 1 {
        Some(ComparisonRefusal::Schema)
    } else if [&before, &after].iter().any(|report| {
        report.node_operator_id.as_slice().is_empty()
            || report
                .chip_id
                .as_ref()
                .is_some_and(|bytes| bytes.len() > super::MAX_CHIP_ID_BYTES)
            || report
                .hostos_version_id
                .as_ref()
                .is_some_and(|text| text.len() > super::MAX_HOSTOS_VERSION_BYTES)
    }) {
        Some(ComparisonRefusal::InvalidRecord)
    } else if [&before, &after].iter().any(|report| {
        report.evidence.source_canister != mainnet::REGISTRY
            || report.evidence.source_method != mainnet::REGISTRY_METHOD
            || report.evidence.source_key != super::source_key(report.node_id)
    }) {
        Some(ComparisonRefusal::SourceContext)
    } else if after.evidence.record_version < before.evidence.record_version {
        Some(ComparisonRefusal::VersionRegression)
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(NodeChangesError::Comparison {
            before: Box::new(before),
            after: Box::new(after),
            reason,
        });
    }
    let report = NodeChangesReport {
        schema: 1,
        requested_interval_seconds: None,
        operator_changed: before.node_operator_id != after.node_operator_id,
        chip_id_changed: before.chip_id != after.chip_id,
        hostos_version_changed: before.hostos_version_id != after.hostos_version_id,
        before,
        after,
    };
    if report.before.evidence.record_version == report.after.evidence.record_version
        && report.has_selected_changes()
    {
        return Err(NodeChangesError::Comparison {
            before: Box::new(report.before),
            after: Box::new(report.after),
            reason: ComparisonRefusal::ConflictingVersion,
        });
    }

    Ok(report)
}

/// Acquire two observations of one node, waiting 1–60 seconds after the first.
///
/// Each existing reader retains its 20-second deadline and byte/field bounds.
/// Requires Tokio IO/time. No history, polling, additional retry or background
/// task is added. Cancellation drops the pending read/wait and starts no more IO.
///
/// # Errors
/// Refuses invalid intervals before IO; retains the first observation if the
/// second read fails and both observations if comparison admission fails.
pub async fn fetch(
    node_id: Principal,
    interval_seconds: u64,
) -> Result<NodeChangesReport, NodeChangesError> {
    sample(interval_seconds, || super::fetch(node_id)).await
}

async fn sample<F, Fut>(
    interval_seconds: u64,
    mut read: F,
) -> Result<NodeChangesReport, NodeChangesError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<NodeReport, NodeError>>,
{
    if !(1..=60).contains(&interval_seconds) {
        return Err(NodeChangesError::InvalidInterval);
    }
    let before = read().await.map_err(NodeChangesError::First)?;
    tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
    let after = match read().await {
        Ok(after) => after,
        Err(source) => {
            return Err(NodeChangesError::Second {
                before: Box::new(before),
                source,
            });
        }
    };
    let mut report = compare(before, after)?;
    report.requested_interval_seconds = Some(interval_seconds);

    Ok(report)
}
