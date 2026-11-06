//! Module: nns::proposals
//!
//! Responsibility: render NNS proposal reports as human-readable text.
//! Does not own: live governance calls, JSON output, or report assembly.
//! Boundary: returns formatted strings without selecting or writing to a process output sink.

use crate::text_value::field;
use crate::text_value::push_titled_table_section;
use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;
use ic_host_text::timestamp::format_utc_timestamp_secs;

use crate::nns::governance::collection_context_lines as governance_collection_context_lines;
use crate::nns::governance::context_lines as governance_context_lines;
use crate::text_value::optional_u64_text;
use crate::text_value::truncate_text;
use crate::text_value::yes_no;
use ic_query::nns::proposals::NnsProposalActivityReport;
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
    let mut lines = governance_collection_context_lines(&report.context);
    lines.extend(crate::text_value::report_fields!(report;
        collection_page_count = report.context.collection_page_count,
        collected_proposal_count,
        point_in_time_guaranteed = yes_no(report.context.point_in_time_guaranteed),
        from_proposal_timestamp_seconds = optional_u64_text(report.from_proposal_timestamp_seconds),
        until_proposal_timestamp_seconds = optional_u64_text(report.until_proposal_timestamp_seconds),
        included_proposal_count,
        excluded_before_from_count,
        excluded_at_or_after_until_count,
        earliest_included_proposal_timestamp_seconds = optional_u64_text(report.earliest_included_proposal_timestamp_seconds),
        latest_included_proposal_timestamp_seconds = optional_u64_text(report.latest_included_proposal_timestamp_seconds),
    ));
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
        table(
            &headers,
            &rows,
            &[Alignment::Right, Alignment::Left, Alignment::Right],
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
        table(
            &["DAY_START_UTC", "DAY_START_SECONDS", "PROPOSALS"],
            &rows,
            &[Alignment::Left, Alignment::Right, Alignment::Right],
        )
    })
}

/// Render proposal rows and collection provenance as human-facing text.
#[must_use]
pub fn nns_proposal_list_report_text(
    report: &NnsProposalListReport,
    options: crate::TextOptions,
) -> String {
    let mut lines = governance_context_lines(&report.context);
    lines.extend(crate::text_value::report_fields!(report;
        requested_limit,
        before_proposal_id = optional_u64_text(report.before_proposal_id),
        status_filter,
        reward_status_filter,
        topic_filter,
        proposer_filter = optional_u64_text(report.proposer_filter),
        query_filter = report.query_filter.as_deref().unwrap_or("-"),
        sort,
        sort_direction,
        result_scope,
        verbose = yes_no(options.verbose),
        proposal_count = report.proposals.len(),
    ));
    crate::report::push_report_provenance_lines(&mut lines, &report.provenance);
    if !report.proposals.is_empty() {
        lines.push(String::new());
        lines.push(table(
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
        ));
    }

    lines.join("\n")
}

/// Render one proposal summary; exhaustive fields and ballots remain in JSON.
#[must_use]
pub fn nns_proposal_report_text(report: &NnsProposalReport, options: crate::TextOptions) -> String {
    let proposal = &report.proposal;
    let mut lines = governance_context_lines(&report.context);
    lines.extend([
        field("proposal_id", &(report.proposal_id)),
        field("verbose", &(yes_no(options.verbose))),
    ]);
    crate::report::push_report_provenance_lines(&mut lines, &report.provenance);
    lines.push(String::new());
    let detail_limit = if options.verbose {
        None
    } else {
        Some(NNS_PROPOSAL_DETAIL_TEXT_LIMIT)
    };
    lines.extend(proposal_detail_lines(proposal, detail_limit));

    lines.join("\n")
}

fn proposal_detail_lines(proposal: &NnsProposalRow, summary_limit: Option<usize>) -> Vec<String> {
    vec![
        field("proposal_id", &(optional_u64_text(proposal.proposal_id))),
        field(
            "topic",
            &(NnsProposalTopic::from_code(proposal.topic).as_str()),
        ),
        field(
            "status",
            &(NnsProposalStatus::from_code(proposal.status).as_str()),
        ),
        field("title", &(proposal_title(proposal))),
        field(
            "proposed_at",
            &(format_utc_timestamp_secs(proposal.proposal_timestamp_seconds)),
        ),
        field(
            "summary",
            &(proposal_detail_text(&proposal.summary, summary_limit)),
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
