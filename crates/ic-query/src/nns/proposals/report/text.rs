//! Module: nns::proposals::report::text
//!
//! Responsibility: render NNS proposal reports as human-readable text.
//! Does not own: live governance calls, JSON output, or report assembly.
//! Boundary: returns formatted strings without selecting or writing to a process output sink.

use super::NnsProposalActivityReport;
#[cfg(feature = "nns-host")]
use super::cache::{
    NnsProposalCacheListReport, NnsProposalCacheStatusReport, NnsProposalRefreshReport,
};
use super::model::{
    NnsProposalBallotRow, NnsProposalListReport, NnsProposalReport, NnsProposalRow,
};
#[cfg(feature = "nns-host")]
use crate::nns::NnsGovernanceRefreshAttemptStatus;
use crate::{
    nns::governance::{governance_context_lines, governance_source_lines},
    subnet_catalog::format_utc_timestamp_secs,
    table::{ColumnAlign, render_table},
    text_value::{optional_u64_text, sanitize_text, truncate_text, yes_no},
    token_amount::e8s_decimal_text,
};

const NNS_PROPOSAL_DETAIL_TEXT_LIMIT: usize = 240;

/// Render a portable NNS proposal activity report without selecting a process output sink.
#[must_use]
pub fn nns_proposal_activity_report_text(report: &NnsProposalActivityReport) -> String {
    let mut lines = activity_preamble(report);
    push_activity_section(&mut lines, "topics:", topic_activity_table(report));
    push_activity_section(&mut lines, "statuses:", status_activity_table(report));
    push_activity_section(
        &mut lines,
        "reward_statuses:",
        reward_status_activity_table(report),
    );
    push_activity_section(&mut lines, "daily_activity:", daily_activity_table(report));
    lines.join("\n")
}

fn activity_preamble(report: &NnsProposalActivityReport) -> Vec<String> {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!(
            "collection_started_at: {}",
            sanitize_text(&report.collection_started_at)
        ),
        format!(
            "collection_updated_at: {}",
            sanitize_text(&report.collection_updated_at)
        ),
    ];
    lines.extend(governance_source_lines(&report.source));
    lines.extend([
        format!("collection_page_count: {}", report.collection_page_count),
        format!(
            "collected_proposal_count: {}",
            report.collected_proposal_count
        ),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!(
            "from_proposal_timestamp_seconds: {}",
            optional_u64_text(report.from_proposal_timestamp_seconds)
        ),
        format!(
            "until_proposal_timestamp_seconds: {}",
            optional_u64_text(report.until_proposal_timestamp_seconds)
        ),
        format!(
            "included_proposal_count: {}",
            report.included_proposal_count
        ),
        format!(
            "excluded_before_from_count: {}",
            report.excluded_before_from_count
        ),
        format!(
            "excluded_at_or_after_until_count: {}",
            report.excluded_at_or_after_until_count
        ),
        format!(
            "earliest_included_proposal_timestamp_seconds: {}",
            optional_u64_text(report.earliest_included_proposal_timestamp_seconds)
        ),
        format!(
            "latest_included_proposal_timestamp_seconds: {}",
            optional_u64_text(report.latest_included_proposal_timestamp_seconds)
        ),
    ]);
    lines
}

fn topic_activity_table(report: &NnsProposalActivityReport) -> Option<String> {
    (!report.topic_counts.is_empty()).then(|| {
        let rows = report
            .topic_counts
            .iter()
            .map(|row| {
                [
                    row.topic.to_string(),
                    row.topic_text.as_str().to_string(),
                    row.proposal_count.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        render_table(
            &["TOPIC_CODE", "TOPIC", "PROPOSALS"],
            &rows,
            &[ColumnAlign::Right, ColumnAlign::Left, ColumnAlign::Right],
        )
    })
}

fn status_activity_table(report: &NnsProposalActivityReport) -> Option<String> {
    (!report.status_counts.is_empty()).then(|| {
        let rows = report
            .status_counts
            .iter()
            .map(|row| {
                [
                    row.status.to_string(),
                    row.status_text.as_str().to_string(),
                    row.proposal_count.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        render_table(
            &["STATUS_CODE", "STATUS", "PROPOSALS"],
            &rows,
            &[ColumnAlign::Right, ColumnAlign::Left, ColumnAlign::Right],
        )
    })
}

fn reward_status_activity_table(report: &NnsProposalActivityReport) -> Option<String> {
    (!report.reward_status_counts.is_empty()).then(|| {
        let rows = report
            .reward_status_counts
            .iter()
            .map(|row| {
                [
                    row.reward_status.to_string(),
                    row.reward_status_text.as_str().to_string(),
                    row.proposal_count.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        render_table(
            &["REWARD_STATUS_CODE", "REWARD_STATUS", "PROPOSALS"],
            &rows,
            &[ColumnAlign::Right, ColumnAlign::Left, ColumnAlign::Right],
        )
    })
}

fn daily_activity_table(report: &NnsProposalActivityReport) -> Option<String> {
    (!report.day_counts.is_empty()).then(|| {
        let rows = report
            .day_counts
            .iter()
            .map(|row| {
                [
                    format_utc_timestamp_secs(row.day_start_timestamp_seconds),
                    row.day_start_timestamp_seconds.to_string(),
                    row.proposal_count.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        render_table(
            &["DAY_START_UTC", "DAY_START_SECONDS", "PROPOSALS"],
            &rows,
            &[ColumnAlign::Left, ColumnAlign::Right, ColumnAlign::Right],
        )
    })
}

#[must_use]
pub fn nns_proposal_list_report_text(report: &NnsProposalListReport) -> String {
    let mut lines = governance_context_lines(&report.context);
    lines.extend([
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "before_proposal_id: {}",
            optional_u64_text(report.before_proposal_id)
        ),
        format!("status_filter: {}", report.status_filter),
        format!("reward_status_filter: {}", report.reward_status_filter),
        format!("topic_filter: {}", report.topic_filter),
        format!(
            "proposer_filter: {}",
            optional_u64_text(report.proposer_filter)
        ),
        format!(
            "query_filter: {}",
            sanitize_text(report.query_filter.as_deref().unwrap_or("-"))
        ),
        format!("sort: {}", report.sort),
        format!("sort_direction: {}", report.sort_direction),
        format!("result_scope: {}", report.result_scope),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("proposal_count: {}", report.proposal_count),
        format!("data_source: {}", report.data_source),
    ]);
    if let Some(cache_path) = report.cache_path.as_ref() {
        lines.push(format!("cache_path: {}", sanitize_text(cache_path)));
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
                        proposal.topic_text.as_str().to_string(),
                        proposal.status_text.as_str().to_string(),
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
pub fn nns_proposal_report_text(report: &NnsProposalReport) -> String {
    let proposal = &report.proposal;
    let mut lines = governance_context_lines(&report.context);
    lines.extend([
        format!("proposal_id: {}", report.proposal_id),
        format!("show_ballots: {}", yes_no(report.show_ballots)),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("data_source: {}", report.data_source),
    ]);
    if let Some(cache_path) = report.cache_path.as_ref() {
        lines.push(format!("cache_path: {}", sanitize_text(cache_path)));
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
#[cfg(feature = "nns-host")]
pub fn nns_proposal_refresh_report_text(report: &NnsProposalRefreshReport) -> String {
    [
        format!("network: {}", sanitize_text(&report.network)),
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
        format!(
            "attempt_finalized: {}",
            yes_no(report.attempt_finalization_error.is_none())
        ),
        format!(
            "attempt_finalization_error: {}",
            sanitize_text(report.attempt_finalization_error.as_deref().unwrap_or("-"))
        ),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
        format!("cache_path: {}", sanitize_text(&report.cache_path)),
        format!(
            "refresh_attempt_path: {}",
            sanitize_text(&report.refresh_attempt_path)
        ),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
    ]
    .join("\n")
}

#[must_use]
#[cfg(feature = "nns-host")]
pub fn nns_proposal_cache_list_report_text(report: &NnsProposalCacheListReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("cache_root: {}", sanitize_text(&report.cache_root)),
        format!("cache_count: {}", report.cache_count),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["STATUS", "GOVERNANCE", "ROWS", "PAGES", "FETCHED_AT"],
            &report
                .caches
                .iter()
                .map(|cache| {
                    [
                        cache.cache_status.to_string(),
                        cache.governance_canister_id.clone(),
                        cache.row_count.to_string(),
                        cache.page_count.to_string(),
                        cache.fetched_at.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
        for cache in &report.caches {
            if let Some(error) = cache.cache_error.as_ref() {
                lines.push(format!(
                    "cache_error: {}: {}",
                    sanitize_text(&cache.cache_path),
                    sanitize_text(error)
                ));
            }
        }
    }
    lines.join("\n")
}

#[must_use]
#[cfg(feature = "nns-host")]
pub fn nns_proposal_cache_status_report_text(report: &NnsProposalCacheStatusReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("cache_root: {}", sanitize_text(&report.cache_root)),
        format!("found: {}", yes_no(report.found)),
        format!(
            "expected_cache_path: {}",
            sanitize_text(&report.expected_cache_path)
        ),
        format!(
            "refresh_attempt_path: {}",
            sanitize_text(&report.refresh_attempt_path)
        ),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("governance_canister_id: {}", cache.governance_canister_id),
            format!("cache_status: {}", cache.cache_status),
            format!("complete: {}", yes_no(cache.complete)),
            format!("row_count: {}", cache.row_count),
            format!("page_count: {}", cache.page_count),
            format!("page_size: {}", cache.page_size),
            format!("fetched_at: {}", sanitize_text(&cache.fetched_at)),
            format!("source_endpoint: {}", sanitize_text(&cache.source_endpoint)),
            format!("cache_path: {}", sanitize_text(&cache.cache_path)),
        ]);
        if let Some(error) = cache.cache_error.as_ref() {
            lines.push(format!("cache_error: {}", sanitize_text(error)));
        }
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
        format!(
            "topic: {} ({})",
            sanitize_text(proposal.topic_text.as_str()),
            proposal.topic
        ),
        format!(
            "status: {} ({})",
            sanitize_text(proposal.status_text.as_str()),
            proposal.status
        ),
        format!(
            "reward_status: {} ({})",
            sanitize_text(proposal.reward_status_text.as_str()),
            proposal.reward_status
        ),
        format!(
            "action: {}",
            sanitize_text(proposal.action_text.as_deref().unwrap_or("-"))
        ),
        format!("title: {}", proposal_title(proposal)),
        format!("url: {}", sanitize_text(empty_text(&proposal.url))),
        format!(
            "reject_cost: {}",
            e8s_decimal_text(proposal.reject_cost_e8s)
        ),
        format!("proposed_at: {}", sanitize_text(&proposal.proposed_at)),
        format!(
            "deadline_at: {}",
            sanitize_text(proposal.deadline_at.as_deref().unwrap_or("-"))
        ),
        format!(
            "decided_at: {}",
            sanitize_text(proposal.decided_at.as_deref().unwrap_or("-"))
        ),
        format!(
            "executed_at: {}",
            sanitize_text(proposal.executed_at.as_deref().unwrap_or("-"))
        ),
        format!(
            "failed_at: {}",
            sanitize_text(proposal.failed_at.as_deref().unwrap_or("-"))
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
    let title = proposal
        .title
        .as_ref()
        .filter(|title| !title.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "-".to_string());
    sanitize_text(&title)
}

fn empty_text(value: &str) -> &str {
    if value.trim().is_empty() { "-" } else { value }
}

fn proposal_detail_text(value: &str, limit: Option<usize>) -> String {
    let value = empty_text(value);
    if value == "-" {
        return value.to_string();
    }
    limit.map_or_else(|| sanitize_text(value), |limit| truncate_text(value, limit))
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
                    ballot.vote_text.as_str().to_string(),
                    e8s_decimal_text(ballot.voting_power),
                ]
            })
            .collect::<Vec<_>>(),
        &[ColumnAlign::Right, ColumnAlign::Left, ColumnAlign::Right],
    ))
}

fn push_activity_section(lines: &mut Vec<String>, title: &str, table: Option<String>) {
    lines.push(String::new());
    lines.push(title.to_string());
    lines.push(table.unwrap_or_else(|| "-".to_string()));
}

#[cfg(feature = "nns-host")]
fn attempt_lines(attempt: &NnsGovernanceRefreshAttemptStatus) -> [String; 9] {
    [
        "latest_attempt:".to_string(),
        format!("  status: {}", attempt.status),
        format!("  started_at: {}", sanitize_text(&attempt.started_at)),
        format!("  updated_at: {}", sanitize_text(&attempt.updated_at)),
        format!("  page_size: {}", attempt.page_size),
        format!("  pages_fetched: {}", attempt.pages_fetched),
        format!("  rows_fetched: {}", attempt.rows_fetched),
        format!(
            "  last_cursor: {}",
            sanitize_text(attempt.last_cursor.as_deref().unwrap_or("-"))
        ),
        format!(
            "  last_error: {}",
            sanitize_text(attempt.last_error.as_deref().unwrap_or("-"))
        ),
    ]
}
