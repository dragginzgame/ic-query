//! Compact maintenance evidence; never a completion verdict.

use ic_host_text::{bordered_table, escape};
use ic_query_topology::subnet::{
    configuration::SubnetConfigurationReport,
    maintenance::{ConfigurationOutcome, MaintenanceReport, VersionMatch},
};
use std::fmt::Write;

pub(crate) fn render(report: &MaintenanceReport) -> String {
    let (before, after) = report.configuration.observations();
    let fields = |r: Option<&SubnetConfigurationReport>| {
        r.map(|c| {
            vec![
                format!("{:?}", c.replica_version_id),
                c.registered_subnet_type.to_string(),
                c.is_halted.to_string(),
                c.halt_at_cup_height.to_string(),
                c.registered_node_ids.len().to_string(),
            ]
        })
        .unwrap_or_else(|| vec!["unavailable".into(); 5])
    };
    let a = fields(Some(before));
    let b = fields(after);
    let mut rows: Vec<_> = [
        "Configured version",
        "Registered type (raw)",
        "Halted flag",
        "Halt-at-CUP flag",
        "Registered members",
    ]
    .iter()
    .enumerate()
    .map(|(i, label)| vec![label.to_string(), a[i].clone(), b[i].clone()])
    .collect();
    rows.push(vec![
        "Expected version".into(),
        version_match(&report.expected_before).into(),
        version_match(&report.expected_after).into(),
    ]);
    rows.push(vec![
        "Replicated messages".into(),
        report.metrics_before.report().map_or_else(
            || "unavailable".into(),
            |m| m.update_transactions_total.to_string(),
        ),
        report.metrics_after.report().map_or_else(
            || "unavailable".into(),
            |m| m.update_transactions_total.to_string(),
        ),
    ]);
    let mut text = format!(
        "Subnet maintenance evidence\n\nSubnet: {}\nRequested wait: {}s | elapsed: {:.2}s | collection {}\n\n{}\n",
        report.subnet_id,
        report.interval_seconds,
        report.elapsed.as_secs_f64(),
        if report.collection_complete {
            "complete"
        } else {
            "partial"
        },
        bordered_table(&["OBSERVATION", "BEFORE", "AFTER"], &rows)
    );
    if let Some(expected) = &report.expected_version {
        writeln!(
            text,
            "Requested configured version: {}",
            escape(&format!("{expected:?}"))
        )
        .expect("String write");
    }
    match &report.configuration {
        ConfigurationOutcome::Compared { report } => {
            writeln!(
                text,
                "Membership changes: {} added / {} removed",
                report.registered_nodes_added.len(),
                report.registered_nodes_removed.len()
            )
            .expect("String write");
        }
        ConfigurationOutcome::Failed { .. } => {
            text.push_str("Configuration comparison unavailable: second read failed.\n")
        }
        ConfigurationOutcome::Refused { reason, .. } => {
            writeln!(text, "Configuration comparison refused: {reason}").expect("String write");
        }
    }
    let change = match &report.message_change {
        Some(ic_query_topology::subnet::activity::MessageChange::Increased { by }) => {
            format!("increased by {by}")
        }
        Some(ic_query_topology::subnet::activity::MessageChange::Unchanged) => "unchanged".into(),
        Some(ic_query_topology::subnet::activity::MessageChange::Decreased) => {
            "decreased; cause unknown".into()
        }
        None => "unavailable".into(),
    };
    writeln!(text, "Message counter change: {change}").expect("String write");
    writeln!(
        text,
        "\nRegistry source: {} / {} / {}\nConfiguration observed at: {} / {} Unix seconds",
        before.evidence.endpoint,
        before.evidence.source_canister,
        before.evidence.source_method,
        before.evidence.observed_at_unix_seconds,
        after.map_or_else(
            || "unavailable".into(),
            |a| a.evidence.observed_at_unix_seconds.to_string()
        )
    )
    .expect("String write");
    for (label, metrics) in [
        ("Before", report.metrics_before.report()),
        ("After", report.metrics_after.report()),
    ] {
        if let Some(m) = metrics {
            writeln!(
                text,
                "{label} metrics: {} {} | observed {} | certificate verified: {}",
                m.evidence.endpoint,
                m.evidence.state_tree_path,
                m.evidence.observed_at_unix_seconds,
                m.evidence.state_certificate_verified
            )
            .expect("String write");
        }
    }
    text.push_str("Registry signatures and metrics certificates are separate evidence; no atomic snapshot.\nRunning replica version, continuous uptime and maintenance completion are unobserved.\nUse --json for raw evidence, exact membership changes and failure categories.\n");
    text
}

fn version_match(value: &VersionMatch) -> &'static str {
    match value {
        VersionMatch::NotRequested => "not requested",
        VersionMatch::Matches => "matches",
        VersionMatch::Differs => "differs",
        VersionMatch::Unavailable => "unavailable",
    }
}
