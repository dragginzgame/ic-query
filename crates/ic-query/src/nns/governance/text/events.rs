//! Module: nns::governance::text::events
//!
//! Responsibility: render NNS Governance reward-event and maturity-modulation reports.
//! Does not own: economics, metrics, live calls, caching, or process output.
//! Boundary: formats the two bounded Governance point-value report families without changing data.

use super::{
    super::{NnsGovernanceMaturityModulationReport, NnsGovernanceRewardEventReport},
    context_lines,
};
use crate::text_value::optional_u64_text;

/// Render one latest NNS Governance reward-event report.
#[must_use]
pub fn nns_governance_reward_event_report_text(report: &NnsGovernanceRewardEventReport) -> String {
    let event = &report.reward_event;
    let mut lines = context_lines(&report.context);
    lines.extend([
        format!(
            "rounds_since_last_distribution: {}",
            optional_u64_text(event.rounds_since_last_distribution)
        ),
        format!("day_after_genesis: {}", event.day_after_genesis),
        format!(
            "actual_timestamp_seconds: {}",
            event.actual_timestamp_seconds
        ),
        format!(
            "total_available_e8s_equivalent: {}",
            event.total_available_e8s_equivalent
        ),
        format!(
            "latest_round_available_e8s_equivalent: {}",
            optional_u64_text(event.latest_round_available_e8s_equivalent)
        ),
        format!(
            "distributed_e8s_equivalent: {}",
            event.distributed_e8s_equivalent
        ),
        format!(
            "settled_proposals: {}",
            if event.settled_proposals.is_empty() {
                "-".to_string()
            } else {
                event
                    .settled_proposals
                    .iter()
                    .map(|proposal| proposal.id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ),
    ]);
    lines.join("\n")
}

/// Render one NNS Governance maturity-modulation report.
#[must_use]
pub fn nns_governance_maturity_modulation_report_text(
    report: &NnsGovernanceMaturityModulationReport,
) -> String {
    let mut lines = context_lines(&report.context);
    if let Some(modulation) = report.maturity_modulation.as_ref() {
        lines.extend([
            format!(
                "current_value_permyriad: {}",
                modulation
                    .current_value_permyriad
                    .map_or_else(|| "-".to_string(), |value| value.to_string())
            ),
            format!(
                "updated_at_timestamp_seconds: {}",
                optional_u64_text(modulation.updated_at_timestamp_seconds)
            ),
        ]);
    } else {
        lines.push("maturity_modulation: -".to_string());
    }
    lines.join("\n")
}
