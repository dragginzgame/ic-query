//! Selected-engine relationships with compact source and membership evidence.

use ic_host_text::{bordered_table, escape};
use ic_query_topology::cloud_engine::engine::{EngineOutcome, EngineReport, MembershipComparison};
use std::{collections::BTreeSet, fmt::Write};

pub(crate) fn cloud_engine(report: &EngineReport) -> String {
    let mut text = format!(
        "Cloud-engine topology\n\nEngine: {}\n",
        escape(&format!("{:?}", report.requested_engine_id.as_str()))
    );
    match &report.outcome {
        EngineOutcome::NotReturned => {
            text.push_str("Not returned: the ID may be unknown or hidden from anonymous callers.\n")
        }
        EngineOutcome::Read { record } => {
            writeln!(
                text,
                "Subnet: {}\nEngine operator: {}",
                record
                    .subnet_id
                    .map_or_else(|| "not bound".into(), |id| id.to_string()),
                record
                    .operator_canister_id
                    .map_or_else(|| "not bound".into(), |id| id.to_string()),
            )
            .expect("String write");
            if let Some(deleted) = record.deleted_at_unix_nanoseconds {
                writeln!(
                    text,
                    "Deletion timestamp recorded: {deleted} Unix nanoseconds"
                )
                .expect("String write");
            }
            let snapshot: BTreeSet<_> = record.record_node_ids.iter().copied().collect();
            let copy = record.registry_copy_subnet.as_ref();
            let copied: BTreeSet<_> = copy
                .into_iter()
                .flat_map(|subnet| &subnet.node_ids)
                .copied()
                .collect();
            match &record.membership {
                MembershipComparison::Unavailable => text
                    .push_str("Registry-copy subnet unavailable; membership comparison unknown.\n"),
                MembershipComparison::Compared {
                    record_only,
                    registry_copy_only,
                } => {
                    writeln!(text, "Members: {} engine record / {} Registry copy | differences: {} record-only / {} copy-only", snapshot.len(), copied.len(), record_only.len(), registry_copy_only.len()).expect("String write");
                }
            }
            if let Some(subnet) = copy {
                let version =
                    escape(&format!("{:?}", subnet.configured_replica_version)).into_owned();
                writeln!(text, "Configured replica version: {version}{}\nRegistered halt flag: {}\nMaximum state size: {}",
                    if subnet.configured_replica_version.is_empty() { " (standard engine version)" } else { "" },
                    subnet.halted,
                    subnet.maximum_state_size_bytes.map_or_else(|| "not supplied".into(), |bytes| format!("{bytes} bytes")),
                ).expect("String write");
            }
            let rows: Vec<_> = snapshot
                .union(&copied)
                .map(|id| {
                    vec![
                        id.to_string(),
                        if snapshot.contains(id) { "yes" } else { "no" }.into(),
                        if copy.is_none() {
                            "unknown"
                        } else if copied.contains(id) {
                            "yes"
                        } else {
                            "no"
                        }
                        .into(),
                    ]
                })
                .collect();
            text.push('\n');
            text.push_str(&bordered_table(
                &["NODE", "ENGINE RECORD", "REGISTRY COPY"],
                &rows,
            ));
            text.push('\n');
            if rows.is_empty() {
                text.push_str("(no node identities returned)\n");
            }
        }
    }
    let evidence = &report.evidence;
    writeln!(text, "\nSource: {} / {} / {}\nObserved at: {} Unix seconds | {:.2}s\nQuery signature verified: {} | Consensus-certified: {}",
        evidence.endpoint, evidence.source_canister, evidence.source_method,
        evidence.observed_at_unix_seconds, report.elapsed.as_secs_f64(),
        evidence.query_signature_verified, evidence.result_certified,
    ).expect("String write");
    text.push_str("Engine-record nodes and Registry-copy membership are separate declarations.\nCopy version/freshness is not exposed; this is not an atomic IC inventory or proof of liveness.\nUse --json for raw ordering, optional values and membership differences.\n");

    text
}
