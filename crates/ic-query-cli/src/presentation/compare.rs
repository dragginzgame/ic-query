//! Side-by-side placement evidence without scoring candidates.

use ic_host_text::bordered_table;
use ic_query_topology::subnet::compare::{Candidate, SharedGroup, SubnetComparisonReport};
use std::fmt::Write;

pub(crate) fn render(report: &SubnetComparisonReport, overlap_details: bool) -> String {
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
        bordered_table(&["OBSERVATION", "LEFT", "RIGHT"], &rows)
    );
    if let Some(overlap) = report.infrastructure_overlap() {
        let coverage = |complete| if complete { "complete" } else { "partial" };
        let selected =
            |side: &Candidate| side.distribution.report().map(|d| d.summary.selected_nodes);
        let left_selected = selected(&report.left);
        let right_selected = selected(&report.right);
        let rows = vec![
            vec![
                "Providers".into(),
                overlap.providers.shared.len().to_string(),
                shared_nodes(&overlap.providers.shared, true, left_selected),
                shared_nodes(&overlap.providers.shared, false, right_selected),
                coverage(overlap.providers.coverage_complete).into(),
                overlap.providers.left_unknown_nodes.to_string(),
                overlap.providers.right_unknown_nodes.to_string(),
            ],
            vec![
                "Data centers".into(),
                overlap.data_centers.shared.len().to_string(),
                shared_nodes(&overlap.data_centers.shared, true, left_selected),
                shared_nodes(&overlap.data_centers.shared, false, right_selected),
                coverage(overlap.data_centers.coverage_complete).into(),
                overlap.data_centers.left_unknown_nodes.to_string(),
                overlap.data_centers.right_unknown_nodes.to_string(),
            ],
        ];
        writeln!(
            text,
            "\nObserved infrastructure overlap\n\n{}\n",
            bordered_table(
                &[
                    "RELATIONSHIP",
                    "SHARED IDENTITIES",
                    "LEFT NODES",
                    "RIGHT NODES",
                    "COVERAGE",
                    "LEFT UNKNOWN NODES",
                    "RIGHT UNKNOWN NODES"
                ],
                &rows
            )
        )
        .expect("String write");
        text.push_str("Node fractions count observed assignments to shared identities out of each subnet's selected membership.\n");
        text.push_str("Zero shared identities means none observed; it does not prove independent failure domains.\n");
        if overlap_details {
            write_shared_groups(
                &mut text,
                "Shared provider identities",
                &overlap.providers.shared,
            );
            write_shared_groups(
                &mut text,
                "Shared data-center identities",
                &overlap.data_centers.shared,
            );
        }
    } else {
        text.push_str(
            "\nInfrastructure overlap unavailable: both distribution reports are required.\n",
        );
    }
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

fn shared_nodes<T>(groups: &[SharedGroup<T>], left: bool, selected: Option<usize>) -> String {
    let count = groups.iter().try_fold(0usize, |sum, group| {
        sum.checked_add(if left {
            group.left_nodes
        } else {
            group.right_nodes
        })
    });

    match count.zip(selected) {
        Some((count, selected)) if count <= selected => format!("{count} / {selected}"),
        _ => "unavailable".into(),
    }
}

fn write_shared_groups<T: std::fmt::Display>(
    text: &mut String,
    title: &str,
    groups: &[SharedGroup<T>],
) {
    writeln!(text, "\n{title}").expect("String write");
    if groups.is_empty() {
        text.push_str("None observed in the acquired evidence.\n");

        return;
    }
    let rows: Vec<_> = groups
        .iter()
        .map(|group| {
            vec![
                group.identity.to_string(),
                group.left_nodes.to_string(),
                group.right_nodes.to_string(),
            ]
        })
        .collect();
    writeln!(
        text,
        "\n{}\n",
        bordered_table(&["IDENTITY", "LEFT NODES", "RIGHT NODES"], &rows)
    )
    .expect("String write");
}
