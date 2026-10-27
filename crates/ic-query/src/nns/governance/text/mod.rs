//! Module: nns::governance::text
//!
//! Responsibility: expose human-readable renderers for direct NNS Governance reports.
//! Does not own: live calls, report construction, caching, or process output.
//! Boundary: preserves one text facade while report families own their native-value formatting.

use ic_host_text::escape::terminal_text;

#[cfg(feature = "nns-host")]
mod collection;
#[cfg(feature = "nns-host")]
pub use collection::{nns_governance_cache_status_report_text, nns_governance_refresh_report_text};
mod economics;
mod events;
mod metrics;
use super::{NnsGovernanceReportContext, NnsGovernanceSourceProvenance};

pub use economics::nns_governance_economics_report_text;
pub use events::{
    nns_governance_maturity_modulation_report_text, nns_governance_reward_event_report_text,
};
pub use metrics::nns_governance_metrics_report_text;

pub fn context_lines(context: &NnsGovernanceReportContext) -> Vec<String> {
    let mut lines = vec![
        format!("network: {}", terminal_text(&context.network)),
        format!("governance_canister_id: {}", context.governance_canister_id),
        format!("fetched_at: {}", terminal_text(&context.fetched_at)),
    ];
    lines.extend(source_lines(&context.source));
    lines
}

pub fn collection_context_lines(
    network: &str,
    governance_canister_id: &str,
    started_at: &str,
    updated_at: &str,
    source: &NnsGovernanceSourceProvenance,
) -> Vec<String> {
    let mut lines = vec![
        format!("network: {}", terminal_text(network)),
        format!("governance_canister_id: {governance_canister_id}"),
        format!("collection_started_at: {}", terminal_text(started_at)),
        format!("collection_updated_at: {}", terminal_text(updated_at)),
    ];
    lines.extend(source_lines(source));
    lines
}

fn source_lines(source: &NnsGovernanceSourceProvenance) -> Vec<String> {
    let mut lines = Vec::new();
    match source {
        NnsGovernanceSourceProvenance::ReplicaQuery {
            endpoint,
            fetched_by,
        } => {
            lines.push("source_transport: replica_query".to_string());
            lines.push(format!("source_endpoint: {}", terminal_text(endpoint)));
            lines.push(format!("fetched_by: {}", terminal_text(fetched_by)));
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
        source.execution_assurance().as_str()
    ));
    lines
}
