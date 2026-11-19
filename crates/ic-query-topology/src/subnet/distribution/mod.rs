//! Provider and registered-location concentration for one selected subnet.
//!
//! A common requested Registry version is not consensus certification, verified
//! geography, independent failure domains, live capacity or an atomic observation.

mod source;
mod summary;
#[cfg(test)]
mod tests;
mod types;

use std::{collections::BTreeSet, time::Duration};

use ic_platform_protocol::registry as protobuf;
use ic_platform_types::Principal;
use tokio::time::{Instant, sleep_until};

use super::configuration;
use crate::{mainnet, node};

use source::{Source, envelope};
pub use types::*;

/// Maximum selected nodes, distinct operators and distinct data centers per phase.
pub const MAX_NODES: usize = 128;

/// Maximum simultaneous logical reads, in bounded batches without spawned tasks.
pub const CONCURRENCY: usize = 4;
/// Overall asynchronous budget including initial version and configuration queries.
pub const DEADLINE: Duration = Duration::from_secs(60);

/// Collect selected-subnet provider/location relationships at one Registry version.
///
/// Uses the fixed mainnet endpoint and verified anonymous query signatures. Reads
/// the global version once, refuses zero, then pins all records to that version.
/// Reads at most 128 nodes, 128 distinct operators and 128 distinct data centers
/// in batches of four under a 60-second total deadline (386 logical queries max).
/// Every leaf retains the shared 20-second and 1 MiB HTTP limits, with 64 KiB
/// protobuf admission. Requires Tokio IO/time; no scans, extra retries or tasks.
///
/// A successful return may be partial: inspect `complete`, per-node relationships
/// and every record outcome. Cancellation retains completed rows and distinguishes
/// cancelled from unstarted reads. Synchronous bounded admission is not a hard
/// real-time deadline. Registry declarations do not prove independent locations,
/// liveness, usable TEE protection or capacity.
///
/// # Errors
/// Initial version/configuration, deadline or membership admission failures return
/// no report. Later record failures are retained in the returned partial report.
pub async fn fetch(subnet_id: Principal) -> Result<SubnetDistributionReport, DistributionError> {
    let started_at = mainnet::observed_at()?;
    let source = source::MainnetSource(mainnet::client()?);

    collect(subnet_id, started_at, &source).await
}

async fn collect(
    subnet_id: Principal,
    started_at: u64,
    source: &impl Source,
) -> Result<SubnetDistributionReport, DistributionError> {
    let started = Instant::now();
    let deadline = started + DEADLINE;
    let initial = async {
        let latest = source.latest().await?;
        let version =
            protobuf::decode_get_latest_version_response(&latest.bytes, mainnet::REGISTRY_LIMITS)?;
        if version == 0 {
            return Err(DistributionError::ZeroVersion);
        }
        let observation = source
            .value(&format!("subnet_record_{subnet_id}"), version)
            .await?;
        let configuration = configuration::project_response(
            subnet_id,
            envelope(&observation.bytes, version)?,
            observation.observed_at,
        )?;
        if configuration.registered_node_ids.len() > MAX_NODES {
            return Err(DistributionError::NodeLimit);
        }

        Ok((version, latest.observed_at, configuration))
    };
    let (version, version_observed_at, configuration) = tokio::select! {
        biased;
        _ = sleep_until(deadline) => return Err(DistributionError::InitialDeadline),
        result = initial => result?,
    };
    let nodes = phase(
        configuration
            .registered_node_ids
            .iter()
            .map(|&id| (id, format!("node_record_{id}"))),
        source,
        version,
        started,
        deadline,
        |id, response, time| Ok(node::project_response(*id, response, time)?),
    )
    .await;
    let operator_ids: BTreeSet<_> = nodes
        .iter()
        .filter_map(|row| row.outcome.report())
        .map(|node| node.node_operator_id)
        .collect();
    // Each member references at most one operator, each operator at most one DC;
    // the admitted membership bound therefore also bounds both relation phases.
    let operators = phase(
        operator_ids
            .into_iter()
            .map(|id| (id, format!("node_operator_record_{id}"))),
        source,
        version,
        started,
        deadline,
        |id, response, time| source::operator(id, &response, time),
    )
    .await;
    let data_center_ids: BTreeSet<_> = operators
        .iter()
        .filter_map(|row| row.outcome.report())
        .filter_map(|operator| operator.data_center.identity.clone())
        .collect();
    let data_centers = phase(
        data_center_ids.into_iter().map(|id| {
            let key = format!("data_center_record_{id}");

            (id, key)
        }),
        source,
        version,
        started,
        deadline,
        |id: &String, response, time| source::data_center(id, &response, time),
    )
    .await;
    let (relationships, summary) = summary::join(&nodes, &operators, &data_centers);
    let complete = summary.provider_unknown == 0 && summary.location_unknown == 0;
    // A deadline counts only when it prevented a needed read, not merely because
    // synchronous report assembly ended at the boundary after all reads completed.
    let overall_deadline_reached =
        unfinished(&nodes) || unfinished(&operators) || unfinished(&data_centers);

    Ok(SubnetDistributionReport {
        schema: 1,
        endpoint: mainnet::ENDPOINT,
        source_canister: mainnet::REGISTRY,
        version_method: "get_latest_version",
        requested_registry_version: version,
        version_observed_at_unix_seconds: version_observed_at,
        started_at_unix_seconds: started_at,
        elapsed: started.elapsed(),
        query_signature_verified: true,
        result_certified: false,
        overall_deadline_reached,
        complete,
        configuration,
        nodes,
        operators,
        data_centers,
        relationships,
        summary,
    })
}

fn unfinished<K, T>(rows: &[RecordRead<K, T>]) -> bool {
    rows.iter().any(|row| {
        matches!(
            row.outcome,
            RecordOutcome::NotStarted | RecordOutcome::OverallDeadline
        )
    })
}

async fn phase<K, T>(
    keys: impl Iterator<Item = (K, String)>,
    source: &impl Source,
    version: u64,
    started: Instant,
    deadline: Instant,
    project: impl Fn(&K, protobuf::GetValueResponse<'_>, u64) -> Result<T, DistributionError>,
) -> Vec<RecordRead<K, T>> {
    let mut rows: Vec<_> = keys
        .map(|(id, source_key)| RecordRead {
            id,
            source_key,
            started_after: None,
            finished_after: None,
            outcome: RecordOutcome::NotStarted,
        })
        .collect();
    for batch in rows.chunks_mut(CONCURRENCY) {
        if Instant::now() >= deadline {
            break;
        }
        let mut batch = batch.iter_mut();
        let work = async {
            tokio::join!(
                read_record(batch.next(), source, version, started, &project),
                read_record(batch.next(), source, version, started, &project),
                read_record(batch.next(), source, version, started, &project),
                read_record(batch.next(), source, version, started, &project),
            );
        };
        tokio::select! {
            biased;
            _ = sleep_until(deadline) => break,
            _ = work => {}
        }
    }

    rows
}

async fn read_record<K, T>(
    row: Option<&mut RecordRead<K, T>>,
    source: &impl Source,
    version: u64,
    started: Instant,
    project: &impl Fn(&K, protobuf::GetValueResponse<'_>, u64) -> Result<T, DistributionError>,
) {
    let Some(row) = row else { return };
    row.started_after = Some(started.elapsed());
    row.outcome = RecordOutcome::OverallDeadline;
    let result = async {
        let observation = source.value(&row.source_key, version).await?;

        project(
            &row.id,
            envelope(&observation.bytes, version)?,
            observation.observed_at,
        )
    }
    .await;
    row.outcome = match result {
        Ok(report) => RecordOutcome::Read { report },
        Err(error) => RecordOutcome::Failed { error },
    };
    row.finished_after = Some(started.elapsed());
}
