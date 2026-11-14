//! Bounded registered-node inventory for one subnet, retaining partial observations.

use std::time::Duration;

use ic_platform_types::Principal;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;
use tokio::time::{Instant, sleep_until};

use super::configuration::{self, SubnetConfigurationError, SubnetConfigurationReport};
use crate::node::{self, NodeError, NodeReport};
use crate::{ReadError, mainnet};

/// Maximum selected membership; larger records are refused before node reads.
pub const MAX_NODES: usize = 128;
/// Node reads proceed in batches of at most four, without background tasks.
pub const CONCURRENCY: usize = 4;
/// Overall asynchronous budget, including the initial configuration read.
pub const DEADLINE: Duration = Duration::from_secs(60);

/// Configuration and one explicit outcome for every member of that record.
#[derive(Debug, Serialize)]
pub struct SubnetNodesReport {
    /// Query-owned schema; remains 1 before 1.0.
    pub schema: u8,
    /// False: latest-record reads do not share a pinned Registry version.
    pub atomic_snapshot: bool,
    /// Local wall-clock time sampled before collection, in raw Unix seconds.
    pub started_at_unix_seconds: u64,
    /// Monotonic collection duration; JSON retains seconds and nanoseconds.
    pub elapsed: Duration,
    /// True only if each selected member produced a matching node report.
    /// An empty membership satisfies this without establishing active membership.
    pub all_node_reads_succeeded: bool,
    /// Whether the overall budget stopped collection after configuration succeeded.
    pub overall_deadline_reached: bool,
    /// First observation, retaining its selected membership and source evidence.
    pub configuration: SubnetConfigurationReport,
    /// Sorted in the configuration reader's canonical principal order.
    pub nodes: Vec<NodeRead>,
}

/// One selected node's attempt, including failures and reads never started.
#[derive(Debug, Serialize)]
pub struct NodeRead {
    /// Identity selected from the returned subnet configuration.
    pub node_id: Principal,
    /// Monotonic offset from collection start; absent if never started.
    pub started_after: Option<Duration>,
    /// Monotonic completion offset; absent for unstarted or cancelled reads.
    pub finished_after: Option<Duration>,
    /// A successful report or explicit reason the node report is unavailable.
    pub outcome: NodeReadOutcome,
}

/// Per-node outcomes do not imply node health, attestation or usable capacity.
#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum NodeReadOutcome {
    /// A matching node report, retaining all of its evidence and raw values.
    Read {
        /// Independently acquired latest Registry record.
        report: Box<NodeReport>,
    },
    /// The bounded node reader failed; other selected reads can still proceed.
    Failed {
        /// Original typed cause in Rust; JSON projects a stable kind and raw
        /// Registry rejection fields, without stringifying transport diagnostics.
        #[serde(serialize_with = "serialize_node_error")]
        error: NodeError,
    },
    /// Refused a source result identifying a different node.
    IdentityMismatch {
        /// Unexpected identity; the requested identity remains on the row.
        observed_node_id: Principal,
    },
    /// Started, then cancelled by the overall collection deadline.
    OverallDeadline,
    /// The overall deadline prevented this read from starting.
    NotStarted,
}

/// Failures before node collection; no partial node inventory is emitted.
#[derive(Debug, Error)]
pub enum SubnetNodesError {
    /// Local start time could not be acquired.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// Initial configuration read failed, preserving its typed cause.
    #[error("node inventory configuration read failed: {0}")]
    Configuration(#[source] SubnetConfigurationError),
    /// The overall budget expired before configuration could be acquired.
    #[error("node inventory deadline expired during configuration")]
    ConfigurationDeadline,
    /// Refused configuration for another subnet before any node read.
    #[error("node inventory configuration identifies {observed}, expected {requested}")]
    IdentityMismatch {
        /// Selected subnet.
        requested: Principal,
        /// Identity returned by the configuration reader.
        observed: Principal,
    },
    /// Selected membership exceeds the collection budget; it is not truncated.
    #[error("node inventory membership exceeds the {MAX_NODES}-node limit")]
    NodeLimit {
        /// Retained configuration so the caller can inspect the refused scope.
        configuration: Box<SubnetConfigurationReport>,
    },
}

/// Read one subnet's registered nodes for Canic software and hardware planning.
///
/// Reads configuration first, then at most 128 node records in batches of four,
/// within one 60-second asynchronous deadline. Each existing leaf reader retains
/// its 20-second deadline, 1 MiB HTTP ceiling and bounded protobuf admission.
/// Requires Tokio IO and time. No extra retries, scans, tasks or relation joins.
/// Cancellation drops pending reads. Synchronous admission is bounded separately;
/// an asynchronous deadline is not a hard real-time execution guarantee.
///
/// A successful return can be partial: inspect `all_node_reads_succeeded` and
/// every row's outcome. Separate latest-record reads are not an atomic snapshot.
/// Registration does not establish attestation, running software or live nodes.
///
/// # Errors
/// Returns typed errors for initial clock/configuration failure, mismatched
/// configuration identity or excessive membership. Per-node failures remain in
/// the returned report with their original typed causes.
pub async fn fetch(subnet_id: Principal) -> Result<SubnetNodesReport, SubnetNodesError> {
    let started_at = mainnet::observed_at()?;
    collect(
        subnet_id,
        started_at,
        configuration::fetch(subnet_id),
        node::fetch,
    )
    .await
}

async fn collect<F>(
    subnet_id: Principal,
    started_at: u64,
    configuration: impl Future<Output = Result<SubnetConfigurationReport, SubnetConfigurationError>>,
    reader: impl Fn(Principal) -> F,
) -> Result<SubnetNodesReport, SubnetNodesError>
where
    F: Future<Output = Result<NodeReport, NodeError>>,
{
    let started = Instant::now();
    let deadline = started + DEADLINE;
    let configuration = tokio::select! {
        biased;
        _ = sleep_until(deadline) => return Err(SubnetNodesError::ConfigurationDeadline),
        result = configuration => result.map_err(SubnetNodesError::Configuration)?,
    };
    if configuration.subnet_id != subnet_id {
        return Err(SubnetNodesError::IdentityMismatch {
            requested: subnet_id,
            observed: configuration.subnet_id,
        });
    }
    if configuration.registered_node_ids.len() > MAX_NODES {
        return Err(SubnetNodesError::NodeLimit {
            configuration: Box::new(configuration),
        });
    }
    // The configuration reader already admits, sorts and rejects duplicate IDs.
    let mut nodes: Vec<_> = configuration
        .registered_node_ids
        .iter()
        .map(|&node_id| NodeRead {
            node_id,
            started_after: None,
            finished_after: None,
            outcome: NodeReadOutcome::NotStarted,
        })
        .collect();
    let mut overall_deadline_reached = false;
    for batch in nodes.chunks_mut(CONCURRENCY) {
        if Instant::now() >= deadline {
            overall_deadline_reached = true;
            break;
        }
        let mut rows = batch.iter_mut();
        let work = async {
            tokio::join!(
                read_node(rows.next(), &reader, started),
                read_node(rows.next(), &reader, started),
                read_node(rows.next(), &reader, started),
                read_node(rows.next(), &reader, started),
            );
        };
        tokio::select! {
            biased;
            _ = sleep_until(deadline) => {
                overall_deadline_reached = true;
                break;
            }
            _ = work => {}
        }
    }
    Ok(SubnetNodesReport {
        schema: 1,
        atomic_snapshot: false,
        started_at_unix_seconds: started_at,
        elapsed: started.elapsed(),
        all_node_reads_succeeded: nodes
            .iter()
            .all(|row| matches!(row.outcome, NodeReadOutcome::Read { .. })),
        overall_deadline_reached,
        configuration,
        nodes,
    })
}

async fn read_node<F>(
    row: Option<&mut NodeRead>,
    reader: &impl Fn(Principal) -> F,
    started: Instant,
) where
    F: Future<Output = Result<NodeReport, NodeError>>,
{
    let Some(row) = row else { return };
    row.started_after = Some(started.elapsed());
    // If the owning deadline cancels this future, the retained row is explicit.
    row.outcome = NodeReadOutcome::OverallDeadline;
    row.outcome = match reader(row.node_id).await {
        Ok(report) if report.node_id == row.node_id => NodeReadOutcome::Read {
            report: Box::new(report),
        },
        Ok(report) => NodeReadOutcome::IdentityMismatch {
            observed_node_id: report.node_id,
        },
        Err(error) => NodeReadOutcome::Failed { error },
    };
    row.finished_after = Some(started.elapsed());
}

fn serialize_node_error<S: Serializer>(
    error: &NodeError,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let kind = match error {
        NodeError::Read(ReadError::Transport(_)) => "transport_or_verification",
        NodeError::Read(ReadError::Deadline) => "read_deadline",
        NodeError::Read(ReadError::Clock(_)) => "clock",
        NodeError::SourceIdentity(_) => "source_identity",
        NodeError::Codec(_) => "codec",
        NodeError::RegistryRejected { .. } => "registry_rejected",
        NodeError::MissingContent => "missing_content",
        NodeError::MissingOperator => "missing_operator",
        NodeError::InvalidOperator(_) => "invalid_operator",
        NodeError::ChipIdLimit => "chip_id_limit",
        NodeError::HostosVersionLimit => "hostos_version_limit",
    };
    let rejected = matches!(error, NodeError::RegistryRejected { .. });
    let mut fields = serializer.serialize_struct("NodeError", if rejected { 4 } else { 1 })?;
    fields.serialize_field("kind", kind)?;
    if let NodeError::RegistryRejected { code, reason, key } = error {
        fields.serialize_field("code", code)?;
        fields.serialize_field("reason", reason)?;
        fields.serialize_field("key", key)?;
    }
    fields.end()
}

#[cfg(test)]
mod tests;
