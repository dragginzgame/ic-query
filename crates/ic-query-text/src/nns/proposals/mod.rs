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
        field(
            "collection_page_count",
            &(report.context.collection_page_count),
        ),
        field(
            "collected_proposal_count",
            &(report.collected_proposal_count),
        ),
        field(
            "point_in_time_guaranteed",
            &(yes_no(report.context.point_in_time_guaranteed)),
        ),
        field(
            "from_proposal_timestamp_seconds",
            &(optional_u64_text(report.from_proposal_timestamp_seconds)),
        ),
        field(
            "until_proposal_timestamp_seconds",
            &(optional_u64_text(report.until_proposal_timestamp_seconds)),
        ),
        field("included_proposal_count", &(report.included_proposal_count)),
        field(
            "excluded_before_from_count",
            &(report.excluded_before_from_count),
        ),
        field(
            "excluded_at_or_after_until_count",
            &(report.excluded_at_or_after_until_count),
        ),
        field(
            "earliest_included_proposal_timestamp_seconds",
            &(optional_u64_text(report.earliest_included_proposal_timestamp_seconds)),
        ),
        field(
            "latest_included_proposal_timestamp_seconds",
            &(optional_u64_text(report.latest_included_proposal_timestamp_seconds)),
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
    lines.extend([
        field("requested_limit", &(report.requested_limit)),
        field(
            "before_proposal_id",
            &(optional_u64_text(report.before_proposal_id)),
        ),
        field("status_filter", &(report.status_filter)),
        field("reward_status_filter", &(report.reward_status_filter)),
        field("topic_filter", &(report.topic_filter)),
        field(
            "proposer_filter",
            &(optional_u64_text(report.proposer_filter)),
        ),
        field(
            "query_filter",
            &(report.query_filter.as_deref().unwrap_or("-")),
        ),
        field("sort", &(report.sort)),
        field("sort_direction", &(report.sort_direction)),
        field("result_scope", &(report.result_scope)),
        field("verbose", &(yes_no(options.verbose))),
        field("proposal_count", &(report.proposals.len())),
    ]);
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

/// Render the discovered local proposal snapshots as a table.
#[must_use]
#[cfg(feature = "nns-host")]
pub fn nns_proposal_cache_list_report_text(report: &NnsProposalCacheListReport) -> String {
    let mut lines = vec![
        field("network", &report.network),
        field("cache_root", &report.cache_root),
        field("cache_count", &(report.caches.len())),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(table(
            &["STATUS", "GOVERNANCE", "ROWS", "PAGES", "FETCHED_AT"],
            &report
                .caches
                .iter()
                .map(|cache| {
                    cache.valid().map_or_else(
                        || ["invalid", "-", "-", "-", "-"].map(str::to_string),
                        |data| {
                            [
                                "ok".to_string(),
                                data.governance_canister_id.clone(),
                                data.row_count.to_string(),
                                data.page_count.to_string(),
                                data.fetched_at.clone(),
                            ]
                        },
                    )
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
            ],
        ));
        for cache in &report.caches {
            if let Some(error) = cache.cache_error() {
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
