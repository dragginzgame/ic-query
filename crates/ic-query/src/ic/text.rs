//! Module: ic::text
//!
//! Responsibility: render IC Dashboard reports as human-facing text.
//! Does not own: REST calls, report construction, JSON serialization, or command output.
//! Boundary: keeps raw values intact in JSON while making nullable text fields readable.

use crate::{
    human_quantity::decimal_cycle_rate_text,
    ic::{
        IcBoundaryNodeDataCentersReport, IcCanisterCountReport, IcCanisterFilters,
        IcCanisterPageReport, IcCanisterReport, IcDailyStatsReport, IcDashboardReportProvenance,
        IcIcrcAccountInfoReport, IcIcrcAccountListReport, IcIcrcHolderListReport,
        IcIcrcIndexedCountReport, IcIcrcTokenValueReport, IcIcrcTotalSupplyReport, IcMetricKind,
        IcMetricReport, IcNodeProviderRewardHistoryReport, IcNodeProviderRewardInfoReport,
        IcNodeProviderRewardListReport, IcReplicaVersionInfoReport, IcReplicaVersionListReport,
    },
    table::{ColumnAlign, render_table},
    text_value::{optional_text, optional_u64_text, sanitize_text, truncate_text, yes_no},
};

const IC_REPLICA_VERSION_TITLE_TEXT_LIMIT: usize = 80;
const IC_REPLICA_VERSION_SUMMARY_TEXT_LIMIT: usize = 240;

pub fn dashboard_provenance_lines(provenance: &IcDashboardReportProvenance) -> Vec<String> {
    vec![
        format!("schema_version: {}", provenance.schema_version),
        format!("network: {}", sanitize_text(&provenance.network)),
        format!("authority: {}", sanitize_text(&provenance.authority)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&provenance.source_endpoint)
        ),
        format!("fetched_at: {}", sanitize_text(&provenance.fetched_at)),
        format!("fetched_by: {}", sanitize_text(&provenance.fetched_by)),
        format!("certified: {}", yes_no(provenance.certified)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(provenance.point_in_time_guaranteed)
        ),
    ]
}

/// Render one official boundary-node data-center report as human-facing text.
#[must_use]
pub fn ic_boundary_node_data_centers_report_text(
    report: &IcBoundaryNodeDataCentersReport,
) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("data_center_count: {}", report.data_center_count),
        format!("total_node_count: {}", report.total_node_count),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("boundary_node_data_centers:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  name={}  owner={}  region={}  latitude={}  longitude={}  nodes={}",
                sanitize_text(&row.dc_id),
                sanitize_text(&row.name),
                sanitize_text(&row.owner),
                sanitize_text(&row.region),
                sanitize_text(&row.latitude),
                sanitize_text(&row.longitude),
                sanitize_text(&row.total_nodes),
            )
        }));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard daily-statistics report as human-facing text.
#[must_use]
pub fn ic_daily_stats_report_text(report: &IcDailyStatsReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("start_unix_secs: {}", report.query.start_unix_secs),
        format!("end_unix_secs: {}", report.query.end_unix_secs),
        format!("returned_day_count: {}", report.returned_day_count),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("daily_stats:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  timestamp={}  avg_total={}  avg_update={}  avg_query={}  max_total={}  max_update={}  max_query={}  blocks_avg={}",
                sanitize_text(&row.day),
                row.timestamp_unix_secs,
                sanitize_text(&row.average_transactions_per_second),
                sanitize_text(&row.average_update_transactions_per_second),
                sanitize_text(&row.average_query_transactions_per_second),
                sanitize_text(&row.max_total_transactions_per_second),
                sanitize_text(&row.max_update_transactions_per_second),
                sanitize_text(&row.max_query_transactions_per_second),
                sanitize_text(&row.blocks_per_second_average),
            )
        }));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard node-provider reward page as human-facing text.
#[must_use]
pub fn ic_node_provider_reward_list_report_text(report: &IcNodeProviderRewardListReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("limit: {}", report.query.limit),
        format!("offset: {}", report.query.offset),
        format!(
            "requested_max_reward_index: {}",
            optional_u64_text(report.query.max_reward_index)
        ),
        format!(
            "resolved_max_reward_index: {}",
            report.resolved_max_reward_index
        ),
        format!("total_reward_records: {}", report.total_reward_records),
        format!("returned_count: {}", report.returned_count),
        format!(
            "next_offset_hint: {}",
            optional_u64_text(report.next_offset_hint)
        ),
        format!("pages_may_overlap: {}", yes_no(report.pages_may_overlap)),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
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
                        sanitize_text(&row.reward_mode),
                        row.reward_timestamp_unix_secs.to_string(),
                        optional_u64_text(row.proposal_id),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
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
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("reward_id: {}", reward.reward_id),
        format!("node_provider_id: {}", reward.node_provider_id),
        format!("amount_e8s: {}", reward.amount_e8s),
        format!("reward_mode: {}", sanitize_text(&reward.reward_mode)),
        format!(
            "reward_timestamp_unix_secs: {}",
            reward.reward_timestamp_unix_secs
        ),
        format!(
            "dashboard_updated_at: {}",
            sanitize_text(&reward.dashboard_updated_at)
        ),
        format!("proposal_id: {}", optional_u64_text(reward.proposal_id)),
        format!(
            "registry_version: {}",
            optional_u64_text(reward.registry_version)
        ),
        format!(
            "maximum_node_provider_rewards_e8s: {}",
            optional_u64_text(reward.maximum_node_provider_rewards_e8s)
        ),
        format!(
            "minimum_xdr_permyriad_per_icp: {}",
            optional_u64_text(reward.minimum_xdr_permyriad_per_icp)
        ),
        format!(
            "xdr_timestamp_unix_secs: {}",
            optional_u64_text(reward.xdr_conversion_rate.timestamp_unix_secs)
        ),
        format!(
            "xdr_permyriad_per_icp: {}",
            optional_u64_text(reward.xdr_conversion_rate.xdr_permyriad_per_icp)
        ),
        format!("details: {}", sanitize_text(&details)),
    ]);
    append_report_footer(&mut lines, &report.provenance);
    lines.join("\n")
}

/// Render one bounded official Dashboard node-provider reward history as human-facing text.
#[must_use]
pub fn ic_node_provider_reward_history_report_text(
    report: &IcNodeProviderRewardHistoryReport,
) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("start_unix_secs: {}", report.query.start_unix_secs),
        format!("end_unix_secs: {}", report.query.end_unix_secs),
        format!("step_secs: {}", report.query.step_secs),
        format!(
            "requested_observation_limit: {}",
            report.requested_observation_limit
        ),
        format!(
            "returned_observation_count: {}",
            report.returned_observation_count
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.observations.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
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
            &[ColumnAlign::Right, ColumnAlign::Right],
        ));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard replica-version page as human-facing text.
#[must_use]
pub fn ic_replica_version_list_report_text(report: &IcReplicaVersionListReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("limit: {}", report.query.limit),
        format!("offset: {}", report.query.offset),
        format!(
            "requested_max_proposal_index: {}",
            optional_u64_text(report.query.max_proposal_index)
        ),
        format!(
            "resolved_max_proposal_index: {}",
            report.resolved_max_proposal_index
        ),
        format!("total_proposals: {}", report.total_proposals),
        format!("returned_count: {}", report.returned_count),
        format!("next_offset: {}", optional_u64_text(report.next_offset)),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
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
                        row.subnet_count.to_string(),
                        truncate_text(&row.title, IC_REPLICA_VERSION_TITLE_TEXT_LIMIT),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
    }
    lines.join("\n")
}

/// Render one exact official Dashboard replica-version report as human-facing text.
#[must_use]
pub fn ic_replica_version_info_report_text(report: &IcReplicaVersionInfoReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!(
            "replica_version_id: {}",
            sanitize_text(&report.replica_version_id)
        ),
        format!("proposal_id: {}", report.proposal_id),
        format!(
            "executed_timestamp_seconds: {}",
            report.executed_timestamp_seconds
        ),
        format!("title: {}", text_or_dash(&report.title)),
        format!("url: {}", text_or_dash(&report.url)),
        format!("subnet_count: {}", report.subnet_count),
        format!(
            "summary: {}",
            truncate_text(&report.summary, IC_REPLICA_VERSION_SUMMARY_TEXT_LIMIT)
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.subnets.is_empty() {
        lines.push(String::new());
        lines.push("subnet_rollouts:".to_string());
        lines.extend(report.subnets.iter().map(|subnet| {
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
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("metric: {}", report.query.metric),
        format!("start_unix_secs: {}", report.query.start_unix_secs),
        format!("end_unix_secs: {}", report.query.end_unix_secs),
        format!("step_secs: {}", report.query.step_secs),
        format!("returned_series_count: {}", report.returned_series_count),
        format!(
            "returned_observation_count: {}",
            report.returned_observation_count
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    for series in &report.series {
        lines.push(String::new());
        lines.push(format!("{}:", sanitize_text(&series.name)));
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
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!(
            "ledger_canister_id: {}",
            sanitize_text(&report.ledger_canister_id)
        ),
        format!("kind: {}", report.kind),
        format!("total: {}", report.total),
    ]);
    append_report_footer(&mut lines, &report.provenance);
    lines.join("\n")
}

/// Render one bounded official ICRC account-index page as human-facing text.
#[must_use]
pub fn icrc_account_list_report_text(report: &IcIcrcAccountListReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!(
            "ledger_canister_id: {}",
            sanitize_text(&report.ledger_canister_id)
        ),
        format!("owner: {}", optional_text(report.query.owner.as_ref())),
        format!("sort_by: {}", report.query.sort_by),
        format!("limit: {}", report.query.limit),
        format!("returned_count: {}", report.returned_count),
        format!(
            "previous_cursor: {}",
            optional_text(report.previous_cursor.as_ref())
        ),
        format!(
            "next_cursor: {}",
            optional_text(report.next_cursor.as_ref())
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

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
        lines.push(render_table(
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
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
            ],
        ));
    }
    lines.join("\n")
}

/// Render one exact official ICRC account record as human-facing text.
#[must_use]
pub fn icrc_account_info_report_text(report: &IcIcrcAccountInfoReport) -> String {
    let account = &report.account;
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("account_id: {}", sanitize_text(&account.account_id)),
        format!("owner: {}", sanitize_text(&account.owner)),
        format!("subaccount: {}", sanitize_text(&account.subaccount)),
        format!(
            "balance_base_units: {}",
            sanitize_text(&account.balance_base_units)
        ),
        format!("total_transactions: {}", account.total_transactions),
        format!("created_at_unix_secs: {}", account.created_at_unix_secs),
        format!(
            "created_at_subsec_nanos: {}",
            account.created_at_subsec_nanos
        ),
        format!(
            "ledger_canister_id: {}",
            sanitize_text(&account.ledger_canister_id)
        ),
        format!(
            "latest_transaction_index: {}",
            account.latest_transaction_index
        ),
        format!(
            "dashboard_updated_at: {}",
            sanitize_text(&account.dashboard_updated_at)
        ),
        format!(
            "active_fee_collector: {}",
            yes_no(account.active_fee_collector)
        ),
        format!(
            "fee_collector_block_range_count: {}",
            account.fee_collector_block_ranges.len()
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);
    lines.join("\n")
}

/// Render one bounded official ICRC holder-index page as human-facing text.
#[must_use]
pub fn icrc_holder_list_report_text(report: &IcIcrcHolderListReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!(
            "ledger_canister_id: {}",
            sanitize_text(&report.ledger_canister_id)
        ),
        format!("sort_by: {}", report.query.sort_by),
        format!("limit: {}", report.query.limit),
        format!("returned_count: {}", report.returned_count),
        format!(
            "previous_cursor: {}",
            optional_text(report.previous_cursor.as_ref())
        ),
        format!(
            "next_cursor: {}",
            optional_text(report.next_cursor.as_ref())
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

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
        lines.push(render_table(
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
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
            ],
        ));
    }
    lines.join("\n")
}

/// Render one bounded official ICRC token-value series as human-facing text.
#[must_use]
pub fn icrc_token_value_report_text(report: &IcIcrcTokenValueReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!(
            "ledger_canister_id: {}",
            sanitize_text(&report.ledger_canister_id)
        ),
        format!("start_unix_secs: {}", report.query.start_unix_secs),
        format!("end_unix_secs: {}", report.query.end_unix_secs),
        format!("limit: {}", report.query.limit),
        format!("returned_row_count: {}", report.returned_row_count),
        format!("limit_reached: {}", yes_no(report.limit_reached)),
    ]);
    append_report_footer(&mut lines, &report.provenance);

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
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!(
            "ledger_canister_id: {}",
            sanitize_text(&report.ledger_canister_id)
        ),
        format!("start_unix_secs: {}", report.query.start_unix_secs),
        format!("end_unix_secs: {}", report.query.end_unix_secs),
        format!("step_secs: {}", report.query.step_secs),
        format!(
            "requested_observation_limit: {}",
            report.requested_observation_limit
        ),
        format!(
            "returned_observation_count: {}",
            report.returned_observation_count
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.observations.is_empty() {
        lines.push(String::new());
        lines.push("total_supply_base_units:".to_string());
        lines.extend(report.observations.iter().map(|observation| {
            format!(
                "  {}  {}",
                observation.timestamp_unix_secs,
                sanitize_text(&observation.total_supply_base_units)
            )
        }));
    }
    lines.join("\n")
}

fn metric_observation_text(metric: IcMetricKind, value: &str) -> String {
    if metric == IcMetricKind::CycleBurnRate {
        decimal_cycle_rate_text(value)
    } else {
        sanitize_text(value)
    }
}

/// Render one official Dashboard canister report as human-facing text.
#[must_use]
pub fn ic_canister_report_text(report: &IcCanisterReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("canister_id: {}", report.canister_id),
        format!("dashboard_id: {}", report.dashboard_id),
        format!("name: {}", text_or_dash(&report.name)),
        format!(
            "canister_type: {}",
            report
                .canister_type
                .as_deref()
                .map_or_else(|| "-".to_string(), text_or_dash)
        ),
        format!("subnet_id: {}", report.subnet_id),
        format!("controller_count: {}", report.controllers.len()),
        format!("language: {}", text_or_dash(&report.language)),
        format!("module_hash: {}", text_or_dash(&report.module_hash)),
        format!(
            "dashboard_updated_at: {}",
            sanitize_text(&report.dashboard_updated_at)
        ),
        format!(
            "upgrade_history_available: {}",
            yes_no(report.upgrades.is_some())
        ),
        format!(
            "upgrade_count: {}",
            report
                .upgrade_count
                .map_or_else(|| "-".to_string(), |count| count.to_string())
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.controllers.is_empty() {
        lines.push(String::new());
        lines.push("controllers:".to_string());
        lines.extend(
            report
                .controllers
                .iter()
                .map(|controller| format!("  {controller}")),
        );
    }

    if let Some(latest) = report
        .upgrades
        .as_ref()
        .and_then(|upgrades| upgrades.first())
    {
        lines.push(String::new());
        lines.push("latest_upgrade:".to_string());
        lines.push(format!("  proposal_id: {}", latest.proposal_id));
        lines.push(format!(
            "  executed_timestamp_seconds: {}",
            latest.executed_timestamp_seconds
        ));
        lines.push(format!("  module_hash: {}", latest.module_hash));
    }

    lines.join("\n")
}

/// Render one official Dashboard canister-count report as human-facing text.
#[must_use]
pub fn ic_canister_count_report_text(report: &IcCanisterCountReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.push(format!("total: {}", report.total));
    append_filters(&mut lines, &report.filters);
    append_report_footer(&mut lines, &report.provenance);
    lines.join("\n")
}

/// Render one bounded official Dashboard canister page as human-facing text.
#[must_use]
pub fn ic_canister_page_report_text(report: &IcCanisterPageReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("returned_count: {}", report.returned_count),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "after: {}",
            report.after.as_deref().map_or("-", |value| value)
        ),
        format!(
            "before: {}",
            report.before.as_deref().map_or("-", |value| value)
        ),
        format!(
            "previous_cursor: {}",
            report.previous_cursor.as_deref().map_or("-", |value| value)
        ),
        format!(
            "next_cursor: {}",
            report.next_cursor.as_deref().map_or("-", |value| value)
        ),
    ]);
    append_filters(&mut lines, &report.filters);
    append_report_footer(&mut lines, &report.provenance);

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
                sanitize_text(&row.dashboard_updated_at),
            )
        }));
    }
    lines.join("\n")
}

fn report_header(provenance: &IcDashboardReportProvenance) -> Vec<String> {
    vec![
        format!("network: {}", sanitize_text(&provenance.network)),
        format!("authority: {}", sanitize_text(&provenance.authority)),
    ]
}

fn append_report_footer(lines: &mut Vec<String>, provenance: &IcDashboardReportProvenance) {
    lines.extend([
        format!("certified: {}", yes_no(provenance.certified)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(provenance.point_in_time_guaranteed)
        ),
        format!("fetched_at: {}", sanitize_text(&provenance.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&provenance.source_endpoint)
        ),
    ]);
}

fn append_filters(lines: &mut Vec<String>, filters: &IcCanisterFilters) {
    if let Some(has_name) = filters.has_name {
        lines.push(format!("filter.has_name: {}", yes_no(has_name)));
    }
    if let Some(subnet_id) = filters.subnet_id.as_deref() {
        lines.push(format!("filter.subnet_id: {subnet_id}"));
    }
    if let Some(controller_id) = filters.controller_id.as_deref() {
        lines.push(format!("filter.controller_id: {controller_id}"));
    }
    if !filters.languages.is_empty() {
        lines.push(format!(
            "filter.languages: {}",
            filters
                .languages
                .iter()
                .map(|value| sanitize_text(value))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    if !filters.canister_types.is_empty() {
        lines.push(format!(
            "filter.canister_types: {}",
            filters
                .canister_types
                .iter()
                .map(|value| sanitize_text(value))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    if let Some(query) = filters.query.as_deref() {
        lines.push(format!("filter.query: {}", sanitize_text(query)));
    }
}

fn text_or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        sanitize_text(value)
    }
}

fn json_scalar_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "-".to_string(),
        serde_json::Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}
