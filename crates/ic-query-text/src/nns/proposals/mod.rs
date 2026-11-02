//! Module: nns::proposals
//!
//! Responsibility: render NNS proposal reports as human-readable text.
//! Does not own: live governance calls, JSON output, or report assembly.
//! Boundary: returns formatted strings without selecting or writing to a process output sink.

use crate::text_value::REPORT_TABLE_OPTIONS;
use crate::text_value::push_titled_table_section;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;
use ic_host_text::table::render;
use ic_host_text::timestamp::format_utc_timestamp_secs;

use crate::nns::governance::collection_context_lines as governance_collection_context_lines;
use crate::nns::governance::context_lines as governance_context_lines;
use crate::text_value::optional_u64_text;
use crate::text_value::truncate_text;
use crate::text_value::yes_no;
use crate::token_amount::e8s_decimal_text;
use ic_query::nns::governance::NnsGovernanceVote;
use ic_query::nns::proposals::NnsProposalActivityReport;
use ic_query::nns::proposals::NnsProposalBallotRow;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::NnsProposalCacheListReport;
use ic_query::nns::proposals::NnsProposalListReport;
use ic_query::nns::proposals::NnsProposalReport;
use ic_query::nns::proposals::NnsProposalRewardStatus;
use ic_query::nns::proposals::NnsProposalRow;
use ic_query::nns::proposals::NnsProposalStatus;
use ic_query::nns::proposals::NnsProposalTopic;

const NNS_PROPOSAL_DETAIL_TEXT_LIMIT: usize = 240;

/// Render a portable NNS proposal activity report without selecting a process output sink.
#[must_use]
pub fn nns_proposal_activity_report_text(report: &NnsProposalActivityReport) -> String {
    let mut lines = activity_preamble(report);
    push_titled_table_section(&mut lines, "topics:", topic_activity_table(report));
    push_titled_table_section(&mut lines, "statuses:", status_activity_table(report));
    push_titled_table_section(
        &mut lines,
        "reward_statuses:",
        reward_status_activity_table(report),
    );
    push_titled_table_section(&mut lines, "daily_activity:", daily_activity_table(report));
    lines.join("\n")
}

fn activity_preamble(report: &NnsProposalActivityReport) -> Vec<String> {
    let mut lines = governance_collection_context_lines(
        &report.context.network,
        &report.context.governance_canister_id,
        &report.context.collection_started_at,
        &report.context.collection_updated_at,
        &report.context.source,
    );
    lines.extend([
        format!(
            "collection_page_count: {}",
            report.context.collection_page_count
        ),
        format!(
            "collected_proposal_count: {}",
            report.collected_proposal_count
        ),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.context.point_in_time_guaranteed)
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
    activity_count_table(
        ["TOPIC_CODE", "TOPIC", "PROPOSALS"],
        report.topic_counts.iter().map(|row| {
            [
                row.topic.to_string(),
                NnsProposalTopic::from_code(row.topic).as_str().to_string(),
                row.proposal_count.to_string(),
            ]
        }),
    )
}

fn status_activity_table(report: &NnsProposalActivityReport) -> Option<String> {
    activity_count_table(
        ["STATUS_CODE", "STATUS", "PROPOSALS"],
        report.status_counts.iter().map(|row| {
            [
                row.status.to_string(),
                NnsProposalStatus::from_code(row.status)
                    .as_str()
                    .to_string(),
                row.proposal_count.to_string(),
            ]
        }),
    )
}

fn reward_status_activity_table(report: &NnsProposalActivityReport) -> Option<String> {
    activity_count_table(
        ["REWARD_STATUS_CODE", "REWARD_STATUS", "PROPOSALS"],
        report.reward_status_counts.iter().map(|row| {
            [
                row.reward_status.to_string(),
                NnsProposalRewardStatus::from_code(row.reward_status)
                    .as_str()
                    .to_string(),
                row.proposal_count.to_string(),
            ]
        }),
    )
}

fn activity_count_table(
    headers: [&str; 3],
    rows: impl Iterator<Item = [String; 3]>,
) -> Option<String> {
    let rows = rows.collect::<Vec<_>>();
    (!rows.is_empty()).then(|| {
        render(
            &headers,
            &rows,
            &[Alignment::Right, Alignment::Left, Alignment::Right],
            REPORT_TABLE_OPTIONS,
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
        render(
            &["DAY_START_UTC", "DAY_START_SECONDS", "PROPOSALS"],
            &rows,
            &[Alignment::Left, Alignment::Right, Alignment::Right],
            REPORT_TABLE_OPTIONS,
        )
    })
}

/// Render proposal rows and collection provenance as human-facing text.
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
            terminal_text(report.query_filter.as_deref().unwrap_or("-"))
        ),
        format!("sort: {}", report.sort),
        format!("sort_direction: {}", report.sort_direction),
        format!("result_scope: {}", report.result_scope),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("proposal_count: {}", report.proposal_count),
    ]);
    crate::report::push_report_provenance_lines(&mut lines, &report.provenance);
    if !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push(render(
            &["ID", "TOPIC", "STATUS", "PROPOSED_AT", "TITLE"],
            &report
                .proposals
                .iter()
                .map(|proposal| {
                    [
                        optional_u64_text(proposal.proposal_id),
                        NnsProposalTopic::from_code(proposal.topic)
                            .as_str()
                            .to_string(),
                        NnsProposalStatus::from_code(proposal.status)
                            .as_str()
                            .to_string(),
                        format_utc_timestamp_secs(proposal.proposal_timestamp_seconds),
                        proposal_title(proposal),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Right,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
            ],
            REPORT_TABLE_OPTIONS,
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

/// Render one proposal, including requested details and ballots, as human-facing text.
#[must_use]
pub fn nns_proposal_report_text(report: &NnsProposalReport) -> String {
    let proposal = &report.proposal;
    let mut lines = governance_context_lines(&report.context);
    lines.extend([
        format!("proposal_id: {}", report.proposal_id),
        format!("show_ballots: {}", yes_no(report.show_ballots)),
        format!("verbose: {}", yes_no(report.verbose)),
    ]);
    crate::report::push_report_provenance_lines(&mut lines, &report.provenance);
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

/// Render the discovered local proposal snapshots as a table.
#[must_use]
#[cfg(feature = "nns-host")]
pub fn nns_proposal_cache_list_report_text(report: &NnsProposalCacheListReport) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("cache_root: {}", terminal_text(&report.cache_root)),
        format!("cache_count: {}", report.cache_count),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render(
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
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
            ],
            REPORT_TABLE_OPTIONS,
        ));
        for cache in &report.caches {
            if let Some(error) = cache.cache_error.as_ref() {
                lines.push(format!(
                    "cache_error: {}: {}",
                    terminal_text(&cache.cache_path),
                    terminal_text(error)
                ));
            }
        }
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
            terminal_text(NnsProposalTopic::from_code(proposal.topic).as_str()),
            proposal.topic
        ),
        format!(
            "status: {} ({})",
            terminal_text(NnsProposalStatus::from_code(proposal.status).as_str()),
            proposal.status
        ),
        format!(
            "reward_status: {} ({})",
            terminal_text(NnsProposalRewardStatus::from_code(proposal.reward_status).as_str()),
            proposal.reward_status
        ),
        format!(
            "action: {}",
            terminal_text(proposal.action_text.as_deref().unwrap_or("-"))
        ),
        format!("title: {}", proposal_title(proposal)),
        format!("url: {}", terminal_text(empty_text(&proposal.url))),
        format!(
            "reject_cost: {}",
            e8s_decimal_text(proposal.reject_cost_e8s)
        ),
        format!(
            "proposed_at: {}",
            format_utc_timestamp_secs(proposal.proposal_timestamp_seconds)
        ),
        format!(
            "deadline_at: {}",
            proposal
                .deadline_timestamp_seconds
                .map_or_else(|| "-".to_string(), format_utc_timestamp_secs)
        ),
        format!(
            "decided_at: {}",
            nonzero_timestamp_text(proposal.decided_timestamp_seconds)
        ),
        format!(
            "executed_at: {}",
            nonzero_timestamp_text(proposal.executed_timestamp_seconds)
        ),
        format!(
            "failed_at: {}",
            nonzero_timestamp_text(proposal.failed_timestamp_seconds)
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
    terminal_text(&title).into_owned()
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
        || terminal_text(value).into_owned(),
        |limit| truncate_text(value, limit),
    )
}

fn proposal_ballot_table(ballots: &[NnsProposalBallotRow]) -> Option<String> {
    if ballots.is_empty() {
        return None;
    }
    Some(render(
        &["NEURON_ID", "VOTE", "VOTING_POWER"],
        &ballots
            .iter()
            .map(|ballot| {
                [
                    ballot.neuron_id.to_string(),
                    NnsGovernanceVote::from_code(ballot.vote).to_string(),
                    e8s_decimal_text(ballot.voting_power),
                ]
            })
            .collect::<Vec<_>>(),
        &[Alignment::Right, Alignment::Left, Alignment::Right],
        REPORT_TABLE_OPTIONS,
    ))
}

fn nonzero_timestamp_text(timestamp_seconds: u64) -> String {
    if timestamp_seconds == 0 {
        "-".to_string()
    } else {
        format_utc_timestamp_secs(timestamp_seconds)
    }
}
