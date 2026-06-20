//! Module: nns::proposals::report::text
//!
//! Responsibility: render NNS proposal reports as text.
//! Does not own: live governance calls, JSON output, or report assembly.
//! Boundary: formats NNS proposal rows for human CLI output.

use super::{
    cache::{
        NnsProposalCacheListReport, NnsProposalCacheStatusReport, NnsProposalRefreshAttemptStatus,
        NnsProposalRefreshReport,
    },
    model::{NnsProposalBallotRow, NnsProposalListReport, NnsProposalReport, NnsProposalRow},
};
use crate::{
    nns::render::yes_no,
    table::{ColumnAlign, render_table},
    token_amount::e8s_decimal_text,
};

const NNS_PROPOSAL_DETAIL_TEXT_LIMIT: usize = 240;

#[must_use]
pub(in crate::nns::proposals) fn nns_proposal_list_report_text(
    report: &NnsProposalListReport,
) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "before_proposal_id: {}",
            optional_u64_text(report.before_proposal_id)
        ),
        format!("status_filter: {}", report.status_filter),
        format!("reward_status_filter: {}", report.reward_status_filter),
        format!("topic_filter: {}", report.topic_filter),
        format!("sort: {}", report.sort),
        format!("sort_direction: {}", report.sort_direction),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("proposal_count: {}", report.proposal_count),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
        format!("data_source: {}", report.data_source),
    ];
    if let Some(cache_path) = report.cache_path.as_ref() {
        lines.push(format!("cache_path: {cache_path}"));
    }
    if let Some(cache_complete) = report.cache_complete {
        lines.push(format!("cache_complete: {}", yes_no(cache_complete)));
    }
    if !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["ID", "TOPIC", "STATUS", "PROPOSED_AT", "TITLE"],
            &report
                .proposals
                .iter()
                .map(|proposal| {
                    [
                        optional_u64_text(proposal.proposal_id),
                        proposal.topic_text.clone(),
                        proposal.status_text.clone(),
                        proposal.proposed_at.clone(),
                        proposal_title(proposal),
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
            lines.extend(proposal_detail_lines(proposal, None));
        }
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::nns::proposals) fn nns_proposal_report_text(report: &NnsProposalReport) -> String {
    let proposal = &report.proposal;
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("proposal_id: {}", report.proposal_id),
        format!("show_ballots: {}", yes_no(report.show_ballots)),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
        format!("data_source: {}", report.data_source),
    ];
    if let Some(cache_path) = report.cache_path.as_ref() {
        lines.push(format!("cache_path: {cache_path}"));
    }
    if let Some(cache_complete) = report.cache_complete {
        lines.push(format!("cache_complete: {}", yes_no(cache_complete)));
    }
    lines.push(String::new());
    let detail_limit = if report.verbose {
        None
    } else {
        Some(NNS_PROPOSAL_DETAIL_TEXT_LIMIT)
    };
    lines.extend(proposal_detail_lines(proposal, detail_limit));
    if report.show_ballots {
        lines.push(String::new());
        lines.push("ballots:".to_string());
        if let Some(table) = proposal_ballot_table(&proposal.ballots) {
            lines.push(table);
        } else {
            lines.push("-".to_string());
        }
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::nns::proposals) fn nns_proposal_refresh_report_text(
    report: &NnsProposalRefreshReport,
) -> String {
    [
        format!("network: {}", report.network),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("proposal_count: {}", report.proposal_count),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("complete: {}", yes_no(report.complete)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
        format!("cache_path: {}", report.cache_path),
        format!("refresh_attempt_path: {}", report.refresh_attempt_path),
        format!("refresh_lock_path: {}", report.refresh_lock_path),
    ]
    .join("\n")
}

#[must_use]
pub(in crate::nns::proposals) fn nns_proposal_cache_list_report_text(
    report: &NnsProposalCacheListReport,
) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("cache_root: {}", report.cache_root),
        format!("cache_count: {}", report.cache_count),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["GOVERNANCE", "ROWS", "PAGES", "FETCHED_AT"],
            &report
                .caches
                .iter()
                .map(|cache| {
                    [
                        cache.governance_canister_id.clone(),
                        cache.row_count.to_string(),
                        cache.page_count.to_string(),
                        cache.fetched_at.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub(in crate::nns::proposals) fn nns_proposal_cache_status_report_text(
    report: &NnsProposalCacheStatusReport,
) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("cache_root: {}", report.cache_root),
        format!("found: {}", yes_no(report.found)),
        format!("expected_cache_path: {}", report.expected_cache_path),
        format!("refresh_attempt_path: {}", report.refresh_attempt_path),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("governance_canister_id: {}", cache.governance_canister_id),
            format!("complete: {}", yes_no(cache.complete)),
            format!("row_count: {}", cache.row_count),
            format!("page_count: {}", cache.page_count),
            format!("page_size: {}", cache.page_size),
            format!("fetched_at: {}", cache.fetched_at),
            format!("source_endpoint: {}", cache.source_endpoint),
            format!("cache_path: {}", cache.cache_path),
        ]);
    } else {
        lines.push("refresh_hint: icq nns proposal refresh".to_string());
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.extend(attempt_lines(attempt));
    }
    lines.join("\n")
}

fn proposal_detail_lines(proposal: &NnsProposalRow, summary_limit: Option<usize>) -> Vec<String> {
    let tally = proposal.latest_tally.as_ref();
    vec![
        format!("proposal_id: {}", optional_u64_text(proposal.proposal_id)),
        format!(
            "proposer_neuron_id: {}",
            optional_u64_text(proposal.proposer_neuron_id)
        ),
        format!("topic: {} ({})", proposal.topic_text, proposal.topic),
        format!("status: {} ({})", proposal.status_text, proposal.status),
        format!(
            "reward_status: {} ({})",
            proposal.reward_status_text, proposal.reward_status
        ),
        format!("action: {}", proposal.action_text.as_deref().unwrap_or("-")),
        format!("title: {}", proposal_title(proposal)),
        format!("url: {}", empty_text(&proposal.url)),
        format!(
            "reject_cost: {}",
            e8s_decimal_text(proposal.reject_cost_e8s)
        ),
        format!("proposed_at: {}", proposal.proposed_at),
        format!(
            "deadline_at: {}",
            proposal.deadline_at.as_deref().unwrap_or("-")
        ),
        format!(
            "decided_at: {}",
            proposal.decided_at.as_deref().unwrap_or("-")
        ),
        format!(
            "executed_at: {}",
            proposal.executed_at.as_deref().unwrap_or("-")
        ),
        format!(
            "failed_at: {}",
            proposal.failed_at.as_deref().unwrap_or("-")
        ),
        format!("reward_event_round: {}", proposal.reward_event_round),
        format!(
            "total_potential_voting_power: {}",
            optional_u64_text(proposal.total_potential_voting_power)
        ),
        format!("ballot_count: {}", proposal.ballot_count),
        format!(
            "latest_tally_yes: {}",
            tally.map_or_else(|| "-".to_string(), |tally| tally.yes.to_string())
        ),
        format!(
            "latest_tally_no: {}",
            tally.map_or_else(|| "-".to_string(), |tally| tally.no.to_string())
        ),
        format!(
            "summary: {}",
            proposal_detail_text(&proposal.summary, summary_limit)
        ),
    ]
}

fn proposal_title(proposal: &NnsProposalRow) -> String {
    proposal
        .title
        .as_ref()
        .filter(|title| !title.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "-".to_string())
}

fn empty_text(value: &str) -> &str {
    if value.trim().is_empty() { "-" } else { value }
}

fn proposal_detail_text(value: &str, limit: Option<usize>) -> String {
    let value = empty_text(value);
    if value == "-" {
        return value.to_string();
    }
    limit.map_or_else(
        || value.to_string(),
        |limit| truncate_text_value(value, limit),
    )
}

fn truncate_text_value(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let truncated = value.chars().take(limit).collect::<String>();
    format!("{truncated}...")
}

fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn proposal_ballot_table(ballots: &[NnsProposalBallotRow]) -> Option<String> {
    if ballots.is_empty() {
        return None;
    }
    Some(render_table(
        &["NEURON_ID", "VOTE", "VOTING_POWER"],
        &ballots
            .iter()
            .map(|ballot| {
                [
                    ballot.neuron_id.to_string(),
                    ballot.vote_text.clone(),
                    e8s_decimal_text(ballot.voting_power),
                ]
            })
            .collect::<Vec<_>>(),
        &[ColumnAlign::Right, ColumnAlign::Left, ColumnAlign::Right],
    ))
}

fn attempt_lines(attempt: &NnsProposalRefreshAttemptStatus) -> [String; 9] {
    [
        "latest_attempt:".to_string(),
        format!("  status: {}", attempt.status),
        format!("  started_at: {}", attempt.started_at),
        format!("  updated_at: {}", attempt.updated_at),
        format!("  page_size: {}", attempt.page_size),
        format!("  pages_fetched: {}", attempt.pages_fetched),
        format!("  rows_fetched: {}", attempt.rows_fetched),
        format!(
            "  last_cursor: {}",
            attempt.last_cursor.as_deref().unwrap_or("-")
        ),
        format!(
            "  last_error: {}",
            attempt.last_error.as_deref().unwrap_or("-")
        ),
    ]
}
