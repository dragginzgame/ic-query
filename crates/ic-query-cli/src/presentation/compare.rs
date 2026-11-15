//! Side-by-side placement evidence without scoring candidates.
use ic_host_text::table::render_bordered;
use ic_topology::subnet::compare::{Candidate, SubnetComparisonReport};
use std::fmt::Write;

pub(crate) fn render(report: &SubnetComparisonReport) -> String {
    let values = |side: &Candidate| {
        let mut rows = if let Some(d) = side.distribution.report() {
            let c = &d.configuration;
            vec![
                c.registered_subnet_type.to_string(),
                format!("{:?}", c.replica_version_id),
                c.is_halted.to_string(),
                c.halt_at_cup_height.to_string(),
                c.registered_node_ids.len().to_string(),
                format!(
                    "{} known / {} unknown",
                    d.summary.provider_resolved, d.summary.provider_unknown
                ),
                format!(
                    "{} known / {} unknown",
                    d.summary.location_resolved, d.summary.location_unknown
                ),
                d.summary.providers.len().to_string(),
                d.summary
                    .providers
                    .iter()
                    .map(|p| p.nodes)
                    .max()
                    .map_or_else(|| "none observed".into(), |n| n.to_string()),
                d.summary.data_centers.len().to_string(),
                d.summary
                    .data_centers
                    .iter()
                    .map(|p| p.nodes)
                    .max()
                    .map_or_else(|| "none observed".into(), |n| n.to_string()),
                d.requested_registry_version.to_string(),
                if d.complete { "complete" } else { "partial" }.into(),
            ]
        } else {
            vec!["unavailable".into(); 13]
        };
        if let Some(m) = side.metrics.report() {
            rows.extend([
                m.num_canisters.to_string(),
                format!("{} bytes", m.canister_state_bytes),
            ]);
        } else {
            rows.extend(["unavailable".into(), "unavailable".into()]);
        }
        rows
    };
    let left = values(&report.left);
    let right = values(&report.right);
    let labels = [
        "Registered type (raw)",
        "Configured version",
        "Halted flag",
        "Halt-at-CUP flag",
        "Registered members",
        "Provider coverage",
        "Location coverage",
        "Observed providers",
        "Largest provider (nodes)",
        "Observed data centers",
        "Largest data center (nodes)",
        "Requested Registry version",
        "Distribution coverage",
        "Canisters",
        "Canister state",
    ];
    let rows: Vec<_> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| vec![label.to_string(), left[i].clone(), right[i].clone()])
        .collect();
    let mut text = format!(
        "Subnet placement comparison\n\nLeft: {}\nRight: {}\nCollection: {} | elapsed: {:.2}s\n\n{}\n",
        report.left.subnet_id,
        report.right.subnet_id,
        if report.collection_complete {
            "complete"
        } else {
            "partial"
        },
        report.elapsed.as_secs_f64(),
        render_bordered(&["OBSERVATION", "LEFT", "RIGHT"], &rows)
    );
    for (label, side) in [("Left", &report.left), ("Right", &report.right)] {
        if let Some(d) = side.distribution.report() {
            writeln!(text, "{label} Registry: {} / {} | version {} observed {} | signature verified: {} | consensus certified: {}", d.endpoint, d.source_canister, d.requested_registry_version, d.version_observed_at_unix_seconds, d.query_signature_verified, d.result_certified).expect("String write");
        }
        if let Some(m) = side.metrics.report() {
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
    text.push_str("Independent observations; no atomic snapshot, capacity ranking or merge-feasibility claim.\nProvider/location counts describe registered declarations, not verified failure domains.\nUse --json for raw evidence, per-record coverage and failure categories.\n");
    text
}
