//! Compact human presentation of selected membership concentration.

use ic_host_text::bordered_table;
use std::fmt::Write;

use ic_topology::subnet::distribution::{RecordOutcome, SubnetDistributionReport};

pub(crate) fn subnet_distribution(report: &SubnetDistributionReport) -> String {
    let summary = &report.summary;
    let mut text = format!(
        "Subnet {}\n\nRegistry version: {} | {} nodes | {:.2}s | {}\nProviders: {} / {} resolved ({} unknown)\nLocations: {} / {} resolved ({} unknown; {} known references)\nRegion unknown: {} | Owner unknown: {}\n\nProviders\n",
        report.configuration.subnet_id,
        report.requested_registry_version,
        summary.selected_nodes,
        report.elapsed.as_secs_f64(),
        if report.complete {
            "complete"
        } else {
            "partial"
        },
        summary.provider_resolved,
        summary.selected_nodes,
        summary.provider_unknown,
        summary.location_resolved,
        summary.selected_nodes,
        summary.location_unknown,
        summary.data_center_references_resolved,
        summary.region_unknown,
        summary.owner_unknown,
    );
    let providers: Vec<_> = summary
        .providers
        .iter()
        .map(|group| vec![group.value.to_text(), group.nodes.to_string()])
        .collect();
    text.push_str(&bordered_table(&["PROVIDER", "NODES"], &providers));
    text.push('\n');
    if summary.providers.is_empty() {
        text.push_str("  (none resolved)\n");
    }
    text.push_str("\nRegistered data centers\n");
    let mut locations = Vec::new();
    for group in &summary.data_centers {
        let record = report
            .data_centers
            .iter()
            .find(|row| row.id == group.value)
            .and_then(|row| match &row.outcome {
                RecordOutcome::Read { report } => Some(report),
                _ => None,
            });
        if let Some(record) = record {
            locations.push(vec![
                group.nodes.to_string(),
                format!("{:?}", group.value),
                format!("{:?}", record.region),
                format!("{:?}", record.owner),
            ]);
        }
    }
    text.push_str(&bordered_table(
        &["NODES", "ID", "REGION", "OWNER"],
        &locations,
    ));
    text.push('\n');
    if summary.data_centers.is_empty() {
        text.push_str("  (none resolved)\n");
    }
    writeln!(
        text,
        "\nIncomplete reads: nodes {} | operators {} | data centers {}",
        report
            .nodes
            .iter()
            .filter(|row| !matches!(row.outcome, RecordOutcome::Read { .. }))
            .count(),
        report
            .operators
            .iter()
            .filter(|row| !matches!(row.outcome, RecordOutcome::Read { .. }))
            .count(),
        report
            .data_centers
            .iter()
            .filter(|row| !matches!(row.outcome, RecordOutcome::Read { .. }))
            .count(),
    )
    .expect("String write");
    if report.overall_deadline_reached {
        text.push_str("Overall deadline reached; cancelled and unstarted reads remain in JSON.\n");
    }
    writeln!(text, "Source: {} / {} | get_latest_version + pinned get_value\nVersion observed at: {} Unix seconds\nQuery signatures verified: {} | Consensus-certified: {}",
        report.endpoint, report.source_canister, report.version_observed_at_unix_seconds,
        report.query_signature_verified, report.result_certified,
    ).expect("String write");
    text.push_str("Counts cover selected membership. Locations/owners are Registry declarations, not verified failure domains.\nNo liveness, capacity or TEE attestation claim. Use --json for per-node joins, raw evidence and failures.\n");
    text
}
