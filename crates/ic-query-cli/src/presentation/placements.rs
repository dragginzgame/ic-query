//! Selected placement rows; no inferred fleet ownership or subnet suitability.
use ic_host_text::table::render_bordered;
use ic_topology::placement::collection::{PlacementCollectionReport, PlacementOutcome};

pub(crate) fn render(report: &PlacementCollectionReport) -> String {
    let rows: Vec<_> = report
        .canisters
        .iter()
        .map(|row| {
            let (subnet, status, observed) = match &row.outcome {
                PlacementOutcome::Read { report } => (
                    report.subnet_id.to_string(),
                    "read".to_owned(),
                    report.evidence.observed_at_unix_seconds.to_string(),
                ),
                PlacementOutcome::Failed { error } => {
                    ("unavailable".into(), error.to_string(), "—".into())
                }
                PlacementOutcome::IdentityMismatch { .. } => {
                    ("unavailable".into(), "identity mismatch".into(), "—".into())
                }
                PlacementOutcome::OverallDeadline => (
                    "unavailable".into(),
                    "cancelled: collection deadline".into(),
                    "—".into(),
                ),
                PlacementOutcome::NotStarted => {
                    ("unavailable".into(), "not started".into(), "—".into())
                }
            };
            vec![row.canister_id.to_string(), subnet, status, observed]
        })
        .collect();
    format!(
        "Selected canister placements\n\nCollection: {} | selected: {} | elapsed: {:.2}s\n\n{}\n\nIndependent mainnet Registry routing observations; no common Registry version or atomic snapshot.\nQuery signatures are verified for successful reads; results are not consensus-certified.\nRouting does not establish existence, liveness, fleet ownership or deployment authority.\nUse --json for each lookup's source evidence and failure category.\n",
        if report.collection_complete {
            "complete"
        } else {
            "partial"
        },
        report.canisters.len(),
        report.elapsed.as_secs_f64(),
        render_bordered(
            &["CANISTER", "SUBNET", "OUTCOME", "OBSERVED (UNIX S)"],
            &rows
        )
    )
}
