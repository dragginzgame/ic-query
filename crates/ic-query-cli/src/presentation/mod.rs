//! Pure human-facing projections; process output remains in the entrypoint.

use std::collections::BTreeSet;
use std::fmt::Write;

use ic_topology::node::NodeReport;
use ic_topology::placement::PlacementReport;
use ic_topology::subnet::SubnetReport;
use ic_topology::subnet::activity::{MessageChange, SubnetActivityReport};
use ic_topology::subnet::changes::SubnetChangesReport;
use ic_topology::subnet::configuration::SubnetConfigurationReport;
use ic_topology::subnet::list::SubnetListReport;
use ic_topology::subnet::membership::SubnetMembershipReport;
use ic_topology::subnet::metrics::SubnetMetricsReport;
use ic_topology::subnet::nodes::{NodeReadOutcome, SubnetNodesReport};

pub(crate) fn subnet_changes(report: &SubnetChangesReport) -> String {
    let before = &report.before;
    let after = &report.after;
    let summary = if report.has_selected_changes() {
        "Selected configuration changed"
    } else {
        "No selected configuration changes observed"
    };
    let mut text = format!(
        "Subnet {}\n\n{}\nRecord mutation version: {} -> {}\nObserved at (Unix seconds): {} -> {}\nSources: {} -> {}\n\n",
        before.subnet_id,
        summary,
        before.evidence.record_version,
        after.evidence.record_version,
        before.evidence.observed_at_unix_seconds,
        after.evidence.observed_at_unix_seconds,
        before.evidence.endpoint,
        after.evidence.endpoint,
    );
    for (changed, name, old, new) in [
        (
            report.replica_version_changed,
            "Configured replica",
            format!("{:?}", before.replica_version_id),
            format!("{:?}", after.replica_version_id),
        ),
        (
            report.subnet_type_changed,
            "Registered subnet type",
            before.registered_subnet_type.to_string(),
            after.registered_subnet_type.to_string(),
        ),
        (
            report.is_halted_changed,
            "Configured halt",
            before.is_halted.to_string(),
            after.is_halted.to_string(),
        ),
        (
            report.halt_at_cup_height_changed,
            "Configured halt at CUP",
            before.halt_at_cup_height.to_string(),
            after.halt_at_cup_height.to_string(),
        ),
    ] {
        if changed {
            writeln!(text, "{name}: {old} -> {new}").expect("writing to a String cannot fail");
        }
    }
    writeln!(
        text,
        "Registered members: {} -> {} ({} added, {} removed)",
        before.registered_node_ids.len(),
        after.registered_node_ids.len(),
        report.registered_nodes_added.len(),
        report.registered_nodes_removed.len()
    )
    .expect("writing to a String cannot fail");
    for (label, nodes) in [
        ("Added", &report.registered_nodes_added),
        ("Removed", &report.registered_nodes_removed),
    ] {
        if !nodes.is_empty() {
            writeln!(text, "\n{label} registered nodes").expect("writing to a String cannot fail");
            for id in nodes {
                writeln!(text, "  {id}").expect("writing to a String cannot fail");
            }
        }
    }
    text.push_str("\nTwo Registry observations, not a complete change history or proof of running software, maintenance completion or uptime.\nUse --json for both full configurations and their source/verification evidence.");
    text
}

pub(crate) fn subnet_nodes(report: &SubnetNodesReport) -> String {
    let records: Vec<_> = report
        .nodes
        .iter()
        .filter_map(|row| match &row.outcome {
            NodeReadOutcome::Read { report } => Some(report.as_ref()),
            _ => None,
        })
        .collect();
    // Local references keep repeated identities/versions out of the table without
    // abbreviating principals or pretending hash prefixes are unique identities.
    let operators: Vec<_> = records
        .iter()
        .map(|node| node.node_operator_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let versions: Vec<_> = records
        .iter()
        .filter_map(|node| node.hostos_version_id.as_deref())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let state = if report.all_node_reads_succeeded {
        "complete"
    } else {
        "partial"
    };
    let mut text = format!(
        "Subnet {}\n\nNodes: {} / {} read | {:.2}s | {}\nStarted: {} Unix seconds | Atomic snapshot: {}\nSource: {} / {} / {}\n",
        report.configuration.subnet_id,
        records.len(),
        report.nodes.len(),
        report.elapsed.as_secs_f64(),
        state,
        report.started_at_unix_seconds,
        report.atomic_snapshot,
        report.configuration.evidence.endpoint,
        report.configuration.evidence.source_canister,
        report.configuration.evidence.source_method,
    );
    if report.overall_deadline_reached {
        text.push_str("Overall deadline reached; unfinished reads are marked below.\n");
    }
    write!(
        text,
        "\n{:<59}  {:<8}  {:<6}  {:<7}  READ\n",
        "NODE", "OPERATOR", "HOSTOS", "CHIP ID"
    )
    .expect("writing to a String cannot fail");
    for row in &report.nodes {
        let (operator, hostos, chip, status) = match &row.outcome {
            NodeReadOutcome::Read { report } => {
                let operator = operators
                    .binary_search(&report.node_operator_id)
                    .expect("collected operator");
                let hostos = report.hostos_version_id.as_deref().map_or_else(
                    || "-".to_owned(),
                    |version| {
                        format!(
                            "H{}",
                            versions.binary_search(&version).expect("collected version") + 1
                        )
                    },
                );
                let chip = match report.chip_id.as_deref() {
                    None => "absent",
                    Some([]) => "empty",
                    Some(_) => "present",
                };
                (format!("O{}", operator + 1), hostos, chip, "ok")
            }
            NodeReadOutcome::Failed { .. } => ("-".into(), "-".into(), "-", "failed"),
            NodeReadOutcome::IdentityMismatch { .. } => {
                ("-".into(), "-".into(), "-", "ID mismatch")
            }
            NodeReadOutcome::OverallDeadline => ("-".into(), "-".into(), "-", "cancelled"),
            NodeReadOutcome::NotStarted => ("-".into(), "-".into(), "-", "not started"),
        };
        writeln!(
            text,
            "{:<59}  {operator:<8}  {hostos:<6}  {chip:<7}  {status}",
            row.node_id.to_string()
        )
        .expect("writing to a String cannot fail");
    }
    if !operators.is_empty() {
        text.push_str("\nRegistered operators\n");
        for (index, operator) in operators.iter().enumerate() {
            writeln!(text, "  O{}  {operator}", index + 1)
                .expect("writing to a String cannot fail");
        }
    }
    if !versions.is_empty() {
        text.push_str("\nConfigured HostOS versions\n");
        for (index, version) in versions.iter().enumerate() {
            writeln!(text, "  H{}  {version:?}", index + 1)
                .expect("writing to a String cannot fail");
        }
    }
    text.push_str("\nChip ID shows registration only; '-' means unavailable. Read failures do not prove downtime.\nRunning software, node liveness and TEE attestation are not established.\nUse --json for full chip IDs, per-node timing, verification and source evidence.");
    text
}

pub(crate) fn node(report: &NodeReport) -> String {
    let chip_id = report.chip_id.as_ref().map_or_else(
        || "absent".to_owned(),
        |bytes| {
            let mut hex = String::from("0x");
            for byte in bytes {
                write!(hex, "{byte:02x}").expect("writing to a String cannot fail");
            }
            hex
        },
    );
    let hostos = report
        .hostos_version_id
        .as_ref()
        .map_or_else(|| "absent".to_owned(), |value| format!("{value:?}"));
    format!(
        "Source: {} / {} / {} / {}\nObserved at: {} Unix seconds\nRecord mutation version: {}\nRecord mutation timestamp: {} Unix nanoseconds\nQuery signature verified: {}\nResult consensus-certified: {}\nTEE attestation verified: {}\n\nNode: {}\nRegistered operator: {}\nRegistered SEV-SNP chip ID (hex; 0x alone means present-empty): {}\nConfigured HostOS version: {}\nRunning HostOS version: not observed\nA registered chip ID does not establish usable TEE protection, node liveness or reduced authentication requirements; absence does not prove missing hardware capability.",
        report.evidence.endpoint,
        report.evidence.source_canister,
        report.evidence.source_method,
        report.evidence.source_key,
        report.evidence.observed_at_unix_seconds,
        report.evidence.record_version,
        report.evidence.record_timestamp_nanoseconds,
        report.evidence.query_signature_verified,
        report.evidence.result_certified,
        report.evidence.tee_attestation_verified,
        report.node_id,
        report.node_operator_id,
        chip_id,
        hostos,
    )
}

pub(crate) fn subnet_membership(report: &SubnetMembershipReport) -> String {
    let registered_only = report
        .registered_without_observed_key
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    let observed_only = report
        .observed_key_without_registration
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Atomic snapshot: {}\nRegistered IDs without an observed key ({}):\n{}\nObserved key IDs absent from the returned Registry record ({}):\n{}\nDifferences may reflect transitions, source age or coverage; matching sets do not establish live nodes.\n\nRegistry observation:\n{}\n\nSubnet-certified observation:\n{}",
        report.atomic_snapshot,
        report.registered_without_observed_key.len(),
        registered_only,
        report.observed_key_without_registration.len(),
        observed_only,
        subnet_configuration(&report.configuration),
        subnet(&report.observation),
    )
}

pub(crate) fn subnet_configuration(report: &SubnetConfigurationReport) -> String {
    let nodes = report
        .registered_node_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Source: {} / {} / {} / {}\nObserved at: {} Unix seconds\nRecord mutation version: {}\nRecord mutation timestamp: {} Unix nanoseconds\nQuery signature verified: {}\nResult consensus-certified: {}\nReturned record membership complete: {}\n\nSubnet: {}\nRegistered subnet type (raw): {}\nConfigured replica version: {:?}\nEffective/running version: not resolved (empty configuration can use a separate cloud-engine record)\nConfigured is_halted: {}\nConfigured halt_at_cup_height: {}\nHalt flags do not establish observed downtime or a reached CUP height.\nRegistered node identities ({}):\n{}",
        report.evidence.endpoint,
        report.evidence.source_canister,
        report.evidence.source_method,
        report.evidence.source_key,
        report.evidence.observed_at_unix_seconds,
        report.evidence.record_version,
        report.evidence.record_timestamp_nanoseconds,
        report.evidence.query_signature_verified,
        report.evidence.result_certified,
        report.evidence.membership_complete,
        report.subnet_id,
        report.registered_subnet_type,
        report.replica_version_id,
        report.is_halted,
        report.halt_at_cup_height,
        report.registered_node_ids.len(),
        nodes,
    )
}

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
