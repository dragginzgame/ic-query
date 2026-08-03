//! Module: nns::governance::text
//!
//! Responsibility: expose human-readable renderers for direct NNS Governance reports.
//! Does not own: live calls, report construction, caching, or process output.
//! Boundary: preserves one text facade while report families own their native-value formatting.

mod economics;
mod events;
mod metrics;

use super::NnsGovernanceReportContext;
use crate::text_value::sanitize_text;

pub use economics::nns_governance_economics_report_text;
pub use events::{
    nns_governance_maturity_modulation_report_text, nns_governance_reward_event_report_text,
};
pub use metrics::nns_governance_metrics_report_text;

fn context_lines(context: &NnsGovernanceReportContext) -> Vec<String> {
    vec![
        format!("network: {}", sanitize_text(&context.network)),
        format!("governance_canister_id: {}", context.governance_canister_id),
        format!("fetched_at: {}", sanitize_text(&context.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&context.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&context.fetched_by)),
    ]
}
