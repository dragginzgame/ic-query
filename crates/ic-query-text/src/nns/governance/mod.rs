//! Module: nns::governance
//!
//! Responsibility: expose human-readable renderers for direct NNS Governance reports.
//! Does not own: live calls, report construction, caching, or process output.
//! Boundary: preserves one text facade while report families own their native-value formatting.

use crate::text_value::field;

#[cfg(feature = "nns-host")]
mod collection;
#[cfg(feature = "nns-host")]
pub use crate::nns::governance::collection::nns_governance_cache_status_report_text;
#[cfg(feature = "nns-host")]
pub use crate::nns::governance::collection::nns_governance_refresh_report_text;
mod economics;
mod events;
mod metrics;
use ic_query::nns::governance::NnsGovernanceReportContext;
use ic_query::nns::governance::NnsGovernanceSourceProvenance;

pub use crate::nns::governance::economics::nns_governance_economics_report_text;
pub use crate::nns::governance::events::nns_governance_maturity_modulation_report_text;
pub use crate::nns::governance::events::nns_governance_reward_event_report_text;
pub use crate::nns::governance::metrics::nns_governance_metrics_report_text;

pub(super) fn context_lines(context: &NnsGovernanceReportContext) -> Vec<String> {
    let mut lines = vec![
        field("network", &context.network),
        field("governance_canister_id", &(context.governance_canister_id)),
        field("fetched_at", &context.fetched_at),
    ];
    lines.extend(source_lines(&context.source));
    lines
}

pub(super) fn collection_context_lines(
    network: &str,
    governance_canister_id: &str,
    started_at: &str,
    updated_at: &str,
    source: &NnsGovernanceSourceProvenance,
) -> Vec<String> {
    let mut lines = vec![
        field("network", &(network)),
        field("governance_canister_id", &(governance_canister_id)),
        field("collection_started_at", &(started_at)),
        field("collection_updated_at", &(updated_at)),
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
            lines.push(field("source_endpoint", &(endpoint)));
            lines.push(field("fetched_by", &(fetched_by)));
        }
        NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
            collector_canister_id,
        } => {
            lines.push("source_transport: replicated_inter_canister_call".to_string());
            lines.push(field("collector_canister_id", &(collector_canister_id)));
        }
    }
    lines.push(field(
        "execution_assurance",
        &(source.execution_assurance().as_str()),
    ));
    lines
}
