use super::super::{SnsProposalReport, SnsProposalRow, SnsProposalsReport};
use super::common::{optional_text, optional_u64_text, truncate_text_value};
use crate::{
    nns::render::yes_no,
    table::{ColumnAlign, render_table},
    token_amount::e8s_decimal_text,
};

const SNS_PROPOSAL_TITLE_TEXT_LIMIT: usize = 96;
const SNS_PROPOSAL_DETAIL_TEXT_LIMIT: usize = 240;

#[must_use]
pub fn sns_proposal_report_text(report: &SnsProposalReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("proposal_id: {}", report.proposal_id),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        String::new(),
        "proposal:".to_string(),
    ];
    let detail_limit = (!report.verbose).then_some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT);
    lines.extend(sns_proposal_detail_lines(&report.proposal, detail_limit));
    lines.join("\n")
}

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
            lines.extend(sns_proposal_detail_lines(
                proposal,
                Some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT),
            ));
        }
    }
    lines.join("\n")
}

fn proposal_title_for_list(proposal: &SnsProposalRow, verbose: bool) -> String {
    if verbose {
        proposal.title.clone()
    } else {
        truncate_text_value(&proposal.title, SNS_PROPOSAL_TITLE_TEXT_LIMIT)
    }
}

fn sns_proposal_detail_lines(
    proposal: &SnsProposalRow,
    detail_limit: Option<usize>,
) -> Vec<String> {
    let mut lines = vec![
        format!("- proposal_id: {}", optional_u64_text(proposal.proposal_id)),
        format!("  action_id: {}", proposal.action_id),
        format!("  action: {}", proposal.action),
        format!("  decision_state: {}", proposal.decision_state),
        format!("  title: {}", proposal.title),
        format!("  url: {}", optional_text(proposal.url.as_ref())),
        format!(
            "  proposer_neuron_id: {}",
            optional_text(proposal.proposer_neuron_id.as_ref())
        ),
        format!(
            "  reject_cost: {}",
            e8s_decimal_text(proposal.reject_cost_e8s)
        ),
        format!("  created_at: {}", proposal.created_at),
        format!(
            "  decided_at: {}",
            optional_text(proposal.decided_at.as_ref())
        ),
        format!(
            "  executed_at: {}",
            optional_text(proposal.executed_at.as_ref())
        ),
        format!(
            "  failed_at: {}",
            optional_text(proposal.failed_at.as_ref())
        ),
        format!("  reward_event_round: {}", proposal.reward_event_round),
        format!(
            "  is_eligible_for_rewards: {}",
            yes_no(proposal.is_eligible_for_rewards)
        ),
        format!("  ballot_count: {}", proposal.ballot_count),
    ];
    if let Some(tally) = proposal.latest_tally.as_ref() {
        lines.extend([
            format!("  tally_yes: {}", tally.yes),
            format!("  tally_no: {}", tally.no),
            format!("  tally_total: {}", tally.total),
        ]);
    }
    if let Some(reason) = proposal.failure_reason.as_ref() {
        lines.extend([
            format!("  failure_error_type: {}", reason.error_type),
            format!("  failure_error_message: {}", reason.error_message),
        ]);
    }
    if !proposal.summary.is_empty() {
        lines.push(format!(
            "  summary: {}",
            proposal_detail_text(&proposal.summary, detail_limit)
        ));
    }
    if let Some(rendering) = proposal.payload_text_rendering.as_ref() {
        lines.push(format!(
            "  payload_text_rendering: {}",
            proposal_detail_text(rendering, detail_limit)
        ));
    }
    lines
}

fn proposal_detail_text(value: &str, detail_limit: Option<usize>) -> String {
    detail_limit.map_or_else(
        || value.to_string(),
        |limit| truncate_text_value(value, limit),
    )
}
