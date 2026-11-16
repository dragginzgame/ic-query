//! Two state-time observations, retaining source/freshness distinctions.
use ic_host_text::bordered_table;
use ic_topology::subnet::progress::{LaterObservation, StateTimeChange, SubnetProgressReport};
use std::fmt::Write;
pub(crate) fn render(report: &SubnetProgressReport) -> String {
    let mut rows = Vec::new();
    let mut append = |label: &str, o: &ic_topology::subnet::progress::StateTimeObservation| {
        rows.push(vec![
            label.into(),
            o.certified_time_nanoseconds.to_string(),
            o.delegation_time_nanoseconds
                .map_or_else(|| "none (root)".into(), |n| n.to_string()),
            o.observed_at_unix_nanoseconds.to_string(),
        ])
    };
    append("Before", &report.before);
    let change = match &report.after {
        LaterObservation::Read {
            observation,
            change,
        } => {
            append("After", observation);
            match change {
                StateTimeChange::Advanced { by_nanoseconds } => {
                    format!("advanced by {by_nanoseconds} nanoseconds")
                }
                StateTimeChange::Unchanged => "unchanged (possibly repeated state)".into(),
                StateTimeChange::Regressed => "regressed (earlier state served second)".into(),
            }
        }
        LaterObservation::Failed { .. } => {
            rows.push(vec![
                "After".into(),
                "unavailable".into(),
                "unavailable".into(),
                "unavailable".into(),
            ]);
            "unavailable; second read failed".into()
        }
    };
    let mut text = format!(
        "Subnet: {}\nState time: {}\nCollection: {} | wait: {}s | elapsed: {:.2}s\n\n{}\n",
        report.subnet_id,
        change,
        if report.collection_complete {
            "complete"
        } else {
            "partial"
        },
        report.interval_seconds,
        report.elapsed.as_secs_f64(),
        bordered_table(
            &[
                "READ",
                "CERTIFIED UNIX NS",
                "DELEGATION UNIX NS",
                "LOCAL OBSERVED UNIX NS"
            ],
            &rows
        )
    );
    let e = &report.before;
    writeln!(text,"Source: {} {} | leaf {}\nFreshness: maximum age {} ns, future skew {} ns (both certificate timestamps)\nCertificate and selected-subnet authority verified. Original encoding bytes are not retained.\nTwo samples do not establish continuous uptime, running software or maintenance completion. Use --json for raw evidence and failure category.",e.endpoint,e.source_path,e.state_tree_path,e.maximum_age_nanoseconds,e.maximum_future_skew_nanoseconds).expect("String write");
    text
}
