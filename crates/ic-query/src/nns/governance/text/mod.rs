//! Module: nns::governance::text
//!
//! Responsibility: expose human-readable renderers for direct NNS Governance reports.
//! Does not own: live calls, report construction, caching, or process output.
//! Boundary: preserves one text facade while report families own their native-value formatting.

mod economics;
mod events;
mod metrics;

use super::{NnsGovernanceReportContext, NnsGovernanceSourceProvenance};
use crate::text_value::sanitize_text;

pub use economics::nns_governance_economics_report_text;
pub use events::{
    nns_governance_maturity_modulation_report_text, nns_governance_reward_event_report_text,
};
pub use metrics::nns_governance_metrics_report_text;

pub fn context_lines(context: &NnsGovernanceReportContext) -> Vec<String> {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&context.network)),
        format!("governance_canister_id: {}", context.governance_canister_id),
        format!("fetched_at: {}", sanitize_text(&context.fetched_at)),
    ];
    match &context.source {
        NnsGovernanceSourceProvenance::ReplicaQuery {
            endpoint,
            fetched_by,
        } => {
            lines.push("source_transport: replica_query".to_string());
            lines.push(format!("source_endpoint: {}", sanitize_text(endpoint)));
            lines.push(format!("fetched_by: {}", sanitize_text(fetched_by)));
        }
        NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
            collector_canister_id,
        } => {
            lines.push("source_transport: replicated_inter_canister_call".to_string());
            lines.push(format!("collector_canister_id: {collector_canister_id}"));
        }
    }
    lines.push(format!(
        "execution_assurance: {}",
        context.source.execution_assurance().as_str()
    ));
    lines
}
