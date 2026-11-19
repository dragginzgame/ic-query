//! Compact differences in node registration, retaining full raw values in JSON.

use std::fmt::Write;

use ic_query_topology::node::changes::NodeChangesReport;

pub(crate) fn node_changes(report: &NodeChangesReport) -> String {
    let before = &report.before;
    let after = &report.after;
    let mut text = format!(
        "Node {}\n\n{}\nRecord mutation version: {} -> {}\nObserved at (Unix seconds): {} -> {}\nSources: {:?} -> {:?}\n\n",
        before.node_id,
        if report.has_selected_changes() {
            "Selected registration changed"
        } else {
            "No selected registration changes observed"
        },
        before.evidence.record_version,
        after.evidence.record_version,
        before.evidence.observed_at_unix_seconds,
        after.evidence.observed_at_unix_seconds,
        before.evidence.endpoint,
        after.evidence.endpoint,
    );
    if report.operator_changed {
        writeln!(
            text,
            "Registered operator: {} -> {}",
            before.node_operator_id, after.node_operator_id
        )
        .expect("String write");
    }
    if report.chip_id_changed {
        writeln!(
            text,
            "Registered chip ID changed: {} -> {} (raw values in --json)",
            chip_presence(before.chip_id.as_deref()),
            chip_presence(after.chip_id.as_deref())
        )
        .expect("String write");
    }
    if report.hostos_version_changed {
        writeln!(
            text,
            "Configured HostOS: {} -> {}",
            hostos(before.hostos_version_id.as_deref()),
            hostos(after.hostos_version_id.as_deref())
        )
        .expect("String write");
    }
    text.push_str("Two observations do not establish event history, running software, node liveness or usable TEE protection.\n");

    text
}

fn chip_presence(value: Option<&[u8]>) -> String {
    match value {
        None => "absent".into(),
        Some([]) => "present-empty".into(),
        Some(bytes) => format!("present ({} bytes)", bytes.len()),
    }
}

fn hostos(value: Option<&str>) -> String {
    value.map_or_else(|| "absent".into(), |text| format!("{text:?}"))
}
