use super::{
    SNS_PROPOSAL_DETAIL_TEXT_LIMIT,
    detail::{proposal_detail_lines, proposal_title_for_list},
};
use crate::{
    nns::render::yes_no,
    sns::report::{SnsProposalsReport, text::common::optional_u64_text},
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn sns_proposals_report_text(report: &SnsProposalsReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "before_proposal_id: {}",
            optional_u64_text(report.before_proposal_id)
        ),
        format!("status_filter: {}", report.status_filter),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("proposal_count: {}", report.proposal_count),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["ID", "ACTION", "DECISION", "CREATED_AT", "TITLE"],
            &report
                .proposals
                .iter()
                .map(|proposal| {
                    [
                        optional_u64_text(proposal.proposal_id),
                        proposal.action.clone(),
                        proposal.decision_state.clone(),
                        proposal.created_at.clone(),
                        proposal_title_for_list(proposal, report.verbose),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }
    if report.verbose && !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push("proposal_details:".to_string());
        for proposal in &report.proposals {
            lines.extend(proposal_detail_lines(
                proposal,
                Some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT),
            ));
        }
    }
    lines.join("\n")
}
