//! Module: sns::report::text::reward_diff
//!
//! Responsibility: render one local SNS reward diff for humans.
//! Does not own: checkpoint parsing, reconciliation, or process output.
//! Boundary: keeps large joined rows in JSON and bounds invalid-reason text.

use crate::{sns::report::SnsRewardDiffReport, text_value::sanitize_text};
use std::fmt::Write as _;

const MAX_TEXT_REASONS: usize = 20;

/// Render one SNS reward diff as bounded human-readable text.
#[must_use]
pub fn sns_reward_diff_report_text(report: &SnsRewardDiffReport) -> String {
    let mut text = format!(
        "network: {}\nroot_canister_id: {}\ngovernance_canister_id: {}\nallocation_status: {}\ncheckpoint_content_authenticated: {}\nbefore_event_end_timestamp_seconds: {}\nafter_event_end_timestamp_seconds: {}\nafter_event_actual_timestamp_seconds: {}\ndistributed_e8s_equivalent: {}\naggregate_maturity_delta_e8s_equivalent: {}\nsummed_neuron_maturity_delta_e8s_equivalent: {}\naggregate_reconciled: {}\nper_neuron_reconciled: {}\nrow_count: {}\ninvalid_reason_count: {}",
        sanitize_text(&report.before.network),
        report.before.root_canister_id,
        report.before.governance_canister_id,
        report.allocation_status.as_str(),
        report.checkpoint_content_authenticated,
        optional_u64(report.before.reward_event_end_timestamp_seconds),
        optional_u64(report.after.reward_event_end_timestamp_seconds),
        report.after.reward_event_actual_timestamp_seconds,
        report.distributed_e8s_equivalent,
        optional_i128(report.aggregate_maturity_delta_e8s_equivalent),
        optional_i128(report.summed_neuron_maturity_delta_e8s_equivalent),
        report.aggregate_reconciled,
        report.per_neuron_reconciled,
        report.rows.len(),
        report.invalid_reasons.len(),
    );
    for invalid in report.invalid_reasons.iter().take(MAX_TEXT_REASONS) {
        let neuron = invalid
            .neuron_id
            .as_deref()
            .map_or(String::new(), |id| format!(" neuron={id}"));
        let _ = write!(
            text,
            "\ninvalid: {}{neuron} {}",
            invalid.kind.as_str(),
            sanitize_text(&invalid.detail)
        );
    }
    if report.invalid_reasons.len() > MAX_TEXT_REASONS {
        let _ = write!(
            text,
            "\ninvalid_reasons_omitted: {}",
            report.invalid_reasons.len() - MAX_TEXT_REASONS
        );
    }
    text
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_i128(value: Option<i128>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}
