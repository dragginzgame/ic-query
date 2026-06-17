use super::{
    SNS_PROPOSAL_DETAIL_TEXT_LIMIT,
    detail::{proposal_ballot_table, proposal_detail_lines},
};
use crate::{
    nns::render::yes_no,
    sns::report::{SnsProposalReport, text::common::push_report_provenance_lines},
};

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
        format!("show_ballots: {}", yes_no(report.show_ballots)),
    ];
    push_report_provenance_lines(
        &mut lines,
        &report.data_source,
        report.cache_path.as_deref(),
        report.cache_complete,
    );
    lines.extend([
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]);
    lines.push(String::new());
    lines.push("proposal:".to_string());
    let detail_limit = (!report.verbose).then_some(SNS_PROPOSAL_DETAIL_TEXT_LIMIT);
    lines.extend(proposal_detail_lines(&report.proposal, detail_limit));
    if report.show_ballots {
        lines.push(String::new());
        lines.push("ballots:".to_string());
        if let Some(table) = proposal_ballot_table(&report.proposal.ballots, report.verbose) {
            lines.push(table);
        } else {
            lines.push("-".to_string());
        }
    }
    lines.join("\n")
}
