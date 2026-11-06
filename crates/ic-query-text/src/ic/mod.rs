//! Module: ic
//!
//! Responsibility: render IC Dashboard reports as human-facing text.
//! Does not own: REST calls, report construction, JSON serialization, or command output.
//! Boundary: keeps raw values intact in JSON while making nullable text fields readable.

use crate::text_value::table;
use crate::text_value::{field, json_scalar_text};
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::human_quantity::decimal_cycle_rate_text;
use crate::text_value::optional_text;
use crate::text_value::optional_u64_text;
use crate::text_value::text_or_dash;
use crate::text_value::truncate_text;
use crate::text_value::yes_no;
use ic_query::ic::IcBoundaryNodeDataCentersReport;
use ic_query::ic::IcCanisterCountReport;
use ic_query::ic::IcCanisterFilters;
use ic_query::ic::IcCanisterPageReport;
use ic_query::ic::IcCanisterReport;
use ic_query::ic::IcDailyStatsReport;
use ic_query::ic::IcDashboardReportProvenance;
use ic_query::ic::IcIcrcAccountInfoReport;
use ic_query::ic::IcIcrcAccountListReport;
use ic_query::ic::IcIcrcHolderListReport;
use ic_query::ic::IcIcrcIndexedCountReport;
use ic_query::ic::IcIcrcTokenValueReport;
use ic_query::ic::IcIcrcTotalSupplyReport;
use ic_query::ic::IcMetricKind;
use ic_query::ic::IcMetricReport;
use ic_query::ic::IcNodeProviderRewardHistoryReport;
use ic_query::ic::IcNodeProviderRewardInfoReport;
use ic_query::ic::IcNodeProviderRewardListReport;
use ic_query::ic::IcReplicaVersionInfoReport;
use ic_query::ic::IcReplicaVersionListReport;

const IC_REPLICA_VERSION_TITLE_TEXT_LIMIT: usize = 80;
const IC_REPLICA_VERSION_SUMMARY_TEXT_LIMIT: usize = 240;

pub(crate) fn dashboard_provenance_lines(provenance: &IcDashboardReportProvenance) -> Vec<String> {
    crate::text_value::report_fields!(provenance;
        schema_version,
        network,
        authority,
        source_endpoint,
        fetched_at,
        fetched_by,
        certified = yes_no(provenance.certified),
        point_in_time_guaranteed = yes_no(provenance.point_in_time_guaranteed),
    )
    .into()
}

fn query_window_lines(start_unix_secs: u64, end_unix_secs: u64) -> [String; 2] {
    [
        field("start_unix_secs", &(start_unix_secs)),
        field("end_unix_secs", &(end_unix_secs)),
    ]
}

fn series_query_lines(start_unix_secs: u64, end_unix_secs: u64, step_secs: u32) -> [String; 3] {
    let [start, end] = query_window_lines(start_unix_secs, end_unix_secs);
    [start, end, field("step_secs", &(step_secs))]
}

/// Render one official boundary-node data-center report as human-facing text.
#[must_use]
pub fn ic_boundary_node_data_centers_report_text(
    report: &IcBoundaryNodeDataCentersReport,
) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        data_center_count = report.rows.len(),
        total_node_count,
    ));

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("boundary_node_data_centers:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  name={}  owner={}  region={}  latitude={}  longitude={}  nodes={}",
                terminal_text(&row.dc_id),
                terminal_text(&row.name),
                terminal_text(&row.owner),
                terminal_text(&row.region),
                terminal_text(&row.latitude),
                terminal_text(&row.longitude),
                terminal_text(&row.total_nodes),
            )
        }));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard daily-statistics report as human-facing text.
#[must_use]
pub fn ic_daily_stats_report_text(report: &IcDailyStatsReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(query_window_lines(
        report.query.start_unix_secs,
        report.query.end_unix_secs,
    ));
    lines.extend([field("returned_day_count", &(report.rows.len()))]);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("daily_stats:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  timestamp={}  avg_total={}  avg_update={}  avg_query={}  max_total={}  max_update={}  max_query={}  blocks_avg={}",
                terminal_text(&row.day),
                row.timestamp_unix_secs,
                terminal_text(&row.average_transactions_per_second),
                terminal_text(&row.average_update_transactions_per_second),
                terminal_text(&row.average_query_transactions_per_second),
                terminal_text(&row.max_total_transactions_per_second),
                terminal_text(&row.max_update_transactions_per_second),
                terminal_text(&row.max_query_transactions_per_second),
                terminal_text(&row.blocks_per_second_average),
            )
        }));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard node-provider reward page as human-facing text.
#[must_use]
pub fn ic_node_provider_reward_list_report_text(report: &IcNodeProviderRewardListReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        limit = report.query.limit,
        offset = report.query.offset,
        requested_max_reward_index = optional_u64_text(report.query.max_reward_index),
        resolved_max_reward_index,
        total_reward_records,
        returned_count = report.rows.len(),
        next_offset_hint = optional_u64_text(report.next_offset_hint),
        pages_may_overlap = yes_no(report.pages_may_overlap),
    ));

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push(table(
            &[
                "ID",
                "PROVIDER",
                "AMOUNT_E8S",
                "MODE",
                "REWARDED_AT",
                "PROPOSAL",
            ],
            &report
                .rows
                .iter()
                .map(|row| {
                    [
                        row.reward_id.to_string(),
                        row.node_provider_id.clone(),
                        row.amount_e8s.to_string(),
                        terminal_text(&row.reward_mode).into_owned(),
                        row.reward_timestamp_unix_secs.to_string(),
                        optional_u64_text(row.proposal_id),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Right,
                Alignment::Left,
                Alignment::Right,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
            ],
        ));
    }
    lines.join("\n")
}

/// Render one exact official Dashboard node-provider reward report as human-facing text.
#[must_use]
pub fn ic_node_provider_reward_info_report_text(report: &IcNodeProviderRewardInfoReport) -> String {
    let reward = &report.reward;
    let details = serde_json::to_string(&reward.details).unwrap_or_else(|_| "{}".to_string());
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(reward;
        reward_id,
        node_provider_id,
        amount_e8s,
        reward_mode,
        reward_timestamp_unix_secs,
        dashboard_updated_at,
        proposal_id = optional_u64_text(reward.proposal_id),
        registry_version = optional_u64_text(reward.registry_version),
        maximum_node_provider_rewards_e8s = optional_u64_text(reward.maximum_node_provider_rewards_e8s),
        minimum_xdr_permyriad_per_icp = optional_u64_text(reward.minimum_xdr_permyriad_per_icp),
        xdr_timestamp_unix_secs = optional_u64_text(reward.xdr_conversion_rate.timestamp_unix_secs),
        xdr_permyriad_per_icp = optional_u64_text(reward.xdr_conversion_rate.xdr_permyriad_per_icp),
        details = details,
    ));
    lines.join("\n")
}

/// Render one bounded official Dashboard node-provider reward history as human-facing text.
#[must_use]
pub fn ic_node_provider_reward_history_report_text(
    report: &IcNodeProviderRewardHistoryReport,
) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(series_query_lines(
        report.query.start_unix_secs,
        report.query.end_unix_secs,
        report.query.step_secs,
    ));
    lines.extend(crate::text_value::report_fields!(report;
        requested_observation_limit,
        returned_observation_count = report.observations.len(),
    ));

    if !report.observations.is_empty() {
        lines.push(String::new());
        lines.push(table(
            &["TIMESTAMP", "AMOUNT_E8S"],
            &report
                .observations
                .iter()
                .map(|observation| {
                    [
                        observation.timestamp_unix_secs.to_string(),
                        observation.amount_e8s.to_string(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[Alignment::Right; 2],
        ));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard replica-version page as human-facing text.
#[must_use]
pub fn ic_replica_version_list_report_text(report: &IcReplicaVersionListReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        limit = report.query.limit,
        offset = report.query.offset,
        requested_max_proposal_index = optional_u64_text(report.query.max_proposal_index),
        resolved_max_proposal_index,
        total_proposals,
        returned_count = report.rows.len(),
        next_offset = optional_u64_text(report.next_offset),
    ));

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push(table(
            &[
                "VERSION",
                "STATUS",
                "PROPOSAL",
                "EXECUTED_AT",
                "SUBNETS",
                "TITLE",
            ],
            &report
                .rows
                .iter()
                .map(|row| {
                    [
                        row.replica_version_id.clone(),
                        row.status.to_string(),
                        row.proposal_id.to_string(),
                        row.executed_timestamp_seconds.to_string(),
                        row.subnets.len().to_string(),
                        truncate_text(&row.title, IC_REPLICA_VERSION_TITLE_TEXT_LIMIT),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
            ],
        ));
    }
    lines.join("\n")
}

/// Render one exact official Dashboard replica-version report as human-facing text.
#[must_use]
pub fn ic_replica_version_info_report_text(report: &IcReplicaVersionInfoReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        replica_version_id = report.release.replica_version_id,
        proposal_id = report.release.proposal_id,
        executed_timestamp_seconds = report.release.executed_timestamp_seconds,
        title = text_or_dash(&report.release.title),
        url = text_or_dash(&report.release.url),
        subnet_count = report.release.subnets.len(),
        summary = truncate_text(
                &report.release.summary,
                IC_REPLICA_VERSION_SUMMARY_TEXT_LIMIT,
            ),
    ));

    if !report.release.subnets.is_empty() {
        lines.push(String::new());
        lines.push("subnet_rollouts:".to_string());
        lines.extend(report.release.subnets.iter().map(|subnet| {
            format!(
                "  {}  proposal={}  executed_at={}",
                subnet.subnet_id, subnet.proposal_id, subnet.executed_timestamp_seconds
            )
        }));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard metric report as human-facing text.
#[must_use]
pub fn ic_metric_report_text(report: &IcMetricReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend([field("metric", &(report.query.metric))]);
    lines.extend(series_query_lines(
        report.query.start_unix_secs,
        report.query.end_unix_secs,
        report.query.step_secs,
    ));
    lines.extend(crate::text_value::report_fields!(report;
        returned_series_count = report.series.len(),
        returned_observation_count = report.series.iter().map(|series| series.observations.len()).sum::<usize>(),
    ));

    for series in &report.series {
        lines.push(String::new());
        lines.push(format!("{}:", terminal_text(&series.name)));
        lines.extend(series.observations.iter().map(|observation| {
            format!(
                "  {}  {}",
                observation.timestamp_unix_secs,
                metric_observation_text(report.query.metric, &observation.value)
            )
        }));
    }
    lines.join("\n")
}

/// Render one official ICRC indexed-count report as human-facing text.
#[must_use]
pub fn icrc_indexed_count_report_text(report: &IcIcrcIndexedCountReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        ledger_canister_id,
        kind,
        total,
    ));
    lines.join("\n")
}

/// Render one bounded official ICRC account-index page as human-facing text.
#[must_use]
pub fn icrc_account_list_report_text(report: &IcIcrcAccountListReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        ledger_canister_id,
        owner = optional_text(report.query.owner.as_ref()),
    ));
    append_cursor_page_lines(
        &mut lines,
        report.query.sort_by.as_api_value(),
        report.query.limit,
        report.rows.len(),
        report.previous_cursor.as_ref(),
        report.next_cursor.as_ref(),
    );

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("accounts:".to_string());
        let rows = report
            .rows
            .iter()
            .map(|row| {
                [
                    row.account_id.clone(),
                    row.owner.clone(),
                    row.balance_base_units.clone(),
                    row.total_transactions.to_string(),
                    row.created_at_unix_secs.to_string(),
                    row.latest_transaction_index.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        lines.push(table(
            &[
                "ACCOUNT ID",
                "OWNER",
                "BALANCE",
                "TXS",
                "CREATED",
                "LATEST TX",
            ],
            &rows,
            &[
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
            ],
        ));
    }
    lines.join("\n")
}

/// Render one exact official ICRC account record as human-facing text.
#[must_use]
pub fn icrc_account_info_report_text(report: &IcIcrcAccountInfoReport) -> String {
    let account = &report.account;
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(account;
        account_id,
        owner,
        subaccount,
        balance_base_units,
        total_transactions,
        created_at_unix_secs,
        created_at_subsec_nanos,
        ledger_canister_id,
        latest_transaction_index,
        dashboard_updated_at,
        active_fee_collector = yes_no(account.active_fee_collector),
        fee_collector_block_range_count = account.fee_collector_block_ranges.len(),
    ));
    lines.join("\n")
}

/// Render one bounded official ICRC holder-index page as human-facing text.
#[must_use]
pub fn icrc_holder_list_report_text(report: &IcIcrcHolderListReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend([field("ledger_canister_id", &report.ledger_canister_id)]);
    append_cursor_page_lines(
        &mut lines,
        report.query.sort_by.as_api_value(),
        report.query.limit,
        report.rows.len(),
        report.previous_cursor.as_ref(),
        report.next_cursor.as_ref(),
    );

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("holders:".to_string());
        let rows = report
            .rows
            .iter()
            .map(|row| {
                [
                    row.principal.clone(),
                    row.balance_base_units.clone(),
                    row.total_transactions.to_string(),
                    json_scalar_text(&row.percentage),
                    json_scalar_text(&row.value_usd),
                    row.created_at_unix_secs.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        lines.push(table(
            &[
                "PRINCIPAL",
                "BALANCE",
                "TXS",
                "PERCENT",
                "VALUE USD",
                "CREATED",
            ],
            &rows,
            &[
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
                Alignment::Right,
            ],
        ));
    }
    lines.join("\n")
}

/// Render one bounded official ICRC token-value series as human-facing text.
#[must_use]
pub fn icrc_token_value_report_text(report: &IcIcrcTokenValueReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend([field("ledger_canister_id", &report.ledger_canister_id)]);
    lines.extend(query_window_lines(
        report.query.start_unix_secs,
        report.query.end_unix_secs,
    ));
    lines.extend(crate::text_value::report_fields!(report;
        limit = report.query.limit,
        returned_row_count = report.rows.len(),
        limit_reached = yes_no(report.limit_reached),
    ));

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("token_values:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  price={}  volume_24h={}  price_usd={}  volume_24h_usd={}  source={}  source_url={}",
                row.timestamp_unix_secs,
                optional_text(row.price.as_ref()),
                optional_text(row.volume_24h.as_ref()),
                optional_text(row.price_usd.as_ref()),
                optional_text(row.volume_24h_usd.as_ref()),
                optional_text(row.source.as_ref()),
                optional_text(row.source_url.as_ref()),
            )
        }));
    }
    lines.join("\n")
}

/// Render one bounded official ICRC total-supply series as human-facing text.
#[must_use]
pub fn icrc_total_supply_report_text(report: &IcIcrcTotalSupplyReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend([field("ledger_canister_id", &report.ledger_canister_id)]);
    lines.extend(series_query_lines(
        report.query.start_unix_secs,
        report.query.end_unix_secs,
        report.query.step_secs,
    ));
    lines.extend(crate::text_value::report_fields!(report;
        requested_observation_limit,
        returned_observation_count = report.observations.len(),
    ));

    if !report.observations.is_empty() {
        lines.push(String::new());
        lines.push("total_supply_base_units:".to_string());
        lines.extend(report.observations.iter().map(|observation| {
            format!(
                "  {}  {}",
                observation.timestamp_unix_secs,
                terminal_text(&observation.total_supply_base_units)
            )
        }));
    }
    lines.join("\n")
}

fn metric_observation_text(metric: IcMetricKind, value: &str) -> String {
    if metric == IcMetricKind::CycleBurnRate {
        decimal_cycle_rate_text(value)
    } else {
        terminal_text(value).into_owned()
    }
}

/// Render one official Dashboard canister report as human-facing text.
#[must_use]
pub fn ic_canister_report_text(report: &IcCanisterReport) -> String {
    let data = &report.data;
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(data;
        canister_id,
        dashboard_id,
        name = text_or_dash(&data.name),
        canister_type = data.canister_type.as_deref().map_or_else(|| "-".to_string(), text_or_dash),
        subnet_id,
        controller_count = data.controllers.len(),
        language = text_or_dash(&data.language),
        module_hash = text_or_dash(&data.module_hash),
        dashboard_updated_at,
        upgrade_history_available = yes_no(data.upgrades.is_some()),
        upgrade_count = data.upgrades.as_ref().map(Vec::len).map_or_else(|| "-".to_string(), |count| count.to_string()),
    ));

    if !data.controllers.is_empty() {
        lines.push(String::new());
        lines.push("controllers:".to_string());
        lines.extend(
            data.controllers
                .iter()
                .map(|controller| format!("  {controller}")),
        );
    }

    if let Some(latest) = data.upgrades.as_ref().and_then(|upgrades| upgrades.first()) {
        lines.push(String::new());
        lines.push("latest_upgrade:".to_string());
        lines.push(field("  proposal_id", &(latest.proposal_id)));
        lines.push(field(
            "  executed_timestamp_seconds",
            &(latest.executed_timestamp_seconds),
        ));
        lines.push(field("  module_hash", &(latest.module_hash)));
    }

    lines.join("\n")
}

/// Render one official Dashboard canister-count report as human-facing text.
#[must_use]
pub fn ic_canister_count_report_text(report: &IcCanisterCountReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.push(field("total", &(report.total)));
    append_filters(&mut lines, &report.filters);
    lines.join("\n")
}

/// Render one bounded official Dashboard canister page as human-facing text.
#[must_use]
pub fn ic_canister_page_report_text(report: &IcCanisterPageReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        returned_count = report.rows.len(),
        requested_limit,
        after = report.after.as_deref().map_or("-", |value| value),
        before = report.before.as_deref().map_or("-", |value| value),
        previous_cursor = report.previous_cursor.as_deref().map_or("-", |value| value),
        next_cursor = report.next_cursor.as_deref().map_or("-", |value| value),
    ));
    append_filters(&mut lines, &report.filters);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("canisters:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  name={}  type={}  subnet={}  controllers={}  language={}  updated={}",
                row.canister_id,
                text_or_dash(&row.name),
                row.canister_type
                    .as_deref()
                    .map_or_else(|| "-".to_string(), text_or_dash),
                row.subnet_id,
                row.controllers.len(),
                text_or_dash(&row.language),
                terminal_text(&row.dashboard_updated_at),
            )
        }));
    }
    lines.join("\n")
}

fn append_filters(lines: &mut Vec<String>, filters: &IcCanisterFilters) {
    if let Some(has_name) = filters.has_name {
        lines.push(field("filter.has_name", &(yes_no(has_name))));
    }
    if let Some(subnet_id) = filters.subnet_id.as_deref() {
        lines.push(field("filter.subnet_id", &(subnet_id)));
    }
    if let Some(controller_id) = filters.controller_id.as_deref() {
        lines.push(field("filter.controller_id", &(controller_id)));
    }
    if !filters.languages.is_empty() {
        lines.push(field(
            "filter.languages",
            &(filters
                .languages
                .iter()
                .map(|value| terminal_text(value))
                .collect::<Vec<_>>()
                .join(",")),
        ));
    }
    if !filters.canister_types.is_empty() {
        lines.push(field(
            "filter.canister_types",
            &(filters
                .canister_types
                .iter()
                .map(|value| terminal_text(value))
                .collect::<Vec<_>>()
                .join(",")),
        ));
    }
    if let Some(query) = filters.query.as_deref() {
        lines.push(field("filter.query", &(query)));
    }
}

fn append_cursor_page_lines(
    lines: &mut Vec<String>,
    sort_by: &str,
    limit: u16,
    returned_count: usize,
    previous_cursor: Option<&String>,
    next_cursor: Option<&String>,
) {
    lines.extend([
        field("sort_by", &(sort_by)),
        field("limit", &(limit)),
        field("returned_count", &(returned_count)),
        field("previous_cursor", &(optional_text(previous_cursor))),
        field("next_cursor", &(optional_text(next_cursor))),
    ]);
}

mod api_boundary_node;
pub use api_boundary_node::*;

mod node_status;
pub use node_status::*;
