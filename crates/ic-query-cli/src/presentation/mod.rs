//! Pure human-facing projections; process output remains in the entrypoint.

use ic_topology::placement::PlacementReport;
use ic_topology::subnet::SubnetReport;
use ic_topology::subnet::activity::{MessageChange, SubnetActivityReport};
use ic_topology::subnet::list::SubnetListReport;
use ic_topology::subnet::metrics::SubnetMetricsReport;

pub(crate) fn subnet_activity(report: &SubnetActivityReport) -> String {
    let change = match report.message_change {
        MessageChange::Increased { by } => format!("increased by {by}"),
        MessageChange::Unchanged => "unchanged (does not establish downtime)".to_owned(),
        MessageChange::Decreased => "decreased (cause and state ordering unknown)".to_owned(),
    };
    format!(
        "First observation:\n{}\n\nRequested wait between reads: {} seconds\n\nSecond observation:\n{}\n\nReplicated-message counter: {}\nTwo samples do not establish uptime, throughput or a certified time interval.",
        subnet_metrics(&report.before),
        report.interval_seconds,
        subnet_metrics(&report.after),
        change,
    )
}

pub(crate) fn subnet_metrics(report: &SubnetMetricsReport) -> String {
    format!(
        "Source: {}{}\nState tree: {}\nObserved at: {} Unix seconds\nState certificate verified: {}\n\nSubnet: {}\nCurrent canisters: {}\nCanister state bytes (refreshed every 10 blocks): {}\nNominal cycles consumed (may decrease): {}\nReplicated messages total: {}\nThese values do not establish available capacity, rates or uptime.",
        report.evidence.endpoint,
        report.evidence.source_path,
        report.evidence.state_tree_path,
        report.evidence.observed_at_unix_seconds,
        report.evidence.state_certificate_verified,
        report.subnet_id,
        report.num_canisters,
        report.canister_state_bytes,
        report.consumed_cycles_total,
        report.update_transactions_total,
    )
}

pub(crate) fn subnet_list(report: &SubnetListReport) -> String {
    let subnets = report
        .subnet_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Source: {} / {} / {} / {}\nObserved at: {} Unix seconds\nRecord mutation version: {}\nRecord mutation timestamp: {} Unix nanoseconds\nQuery signature verified: {}\nResult consensus-certified: {}\nSelected record complete: {}\n\nRegistered subnets ({}):\n{}",
        report.evidence.endpoint,
        report.evidence.source_canister,
        report.evidence.source_method,
        report.evidence.source_key,
        report.evidence.observed_at_unix_seconds,
        report.evidence.record_version,
        report.evidence.record_timestamp_nanoseconds,
        report.evidence.query_signature_verified,
        report.evidence.result_certified,
        report.evidence.selected_record_complete,
        report.subnet_ids.len(),
        subnets,
    )
}

pub(crate) fn placement(report: &PlacementReport) -> String {
    format!(
        "Source: {} / {} / {}\nObserved at: {} Unix seconds\nQuery signature verified: {}\nResult consensus-certified: {}\nSelected lookup complete: {}\nRegistry version: {}\n\nCanister: {}\nSubnet: {}",
        report.evidence.endpoint,
        report.evidence.source_canister,
        report.evidence.source_method,
        report.evidence.observed_at_unix_seconds,
        report.evidence.query_signature_verified,
        report.evidence.result_certified,
        report.evidence.selected_lookup_complete,
        registry_version(report.evidence.registry_version),
        report.canister_id,
        report.subnet_id,
    )
}

pub(crate) fn subnet(report: &SubnetReport) -> String {
    let subnet_type = report
        .observed_subnet_type
        .as_ref()
        .map_or_else(|| "absent".to_owned(), |value| format!("{value:?}"));
    let nodes = report
        .observed_node_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Source: {}{}\nObserved at: {} Unix seconds\nState certificate verified: {}\nComplete Registry membership established: {}\nRegistry version: {}\n\nSubnet: {}\nObserved subnet type (subnet-certified, not Registry-confirmed): {}\nObserved node identities ({}):\n{}",
        report.evidence.endpoint,
        report.evidence.source_path,
        report.evidence.observed_at_unix_seconds,
        report.evidence.state_certificate_verified,
        report.evidence.membership_complete,
        registry_version(report.evidence.registry_version),
        report.subnet_id,
        subnet_type,
        report.observed_node_ids.len(),
        nodes,
    )
}

fn registry_version(version: Option<u64>) -> String {
    version.map_or_else(|| "unavailable".to_owned(), |value| value.to_string())
}

#[cfg(test)]
mod tests;
