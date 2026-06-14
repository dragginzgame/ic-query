use super::{NnsNodeProviderInfoReport, NnsNodeProviderListReport, NnsNodeProviderRefreshReport};
use crate::{
    nns::render::{
        NnsLeafRefreshText, compact_text, nns_leaf_refresh_report_text, optional_node_count_text,
        text_or_dash,
    },
    table::{ColumnAlign, render_table},
};

const COMPACT_PRINCIPAL_CHARS: usize = 5;

#[must_use]
pub fn nns_node_provider_list_report_text(report: &NnsNodeProviderListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "node_providers: {} count {} fetched_at {}",
        report.network, report.node_provider_count, report.fetched_at
    ));
    if report.node_providers.is_empty() {
        lines.push("node providers: none".to_string());
        return lines.join("\n");
    }

    let headers = ["NODE_PROVIDER", "NODES"];
    let rows = report
        .node_providers
        .iter()
        .map(|provider| {
            [
                compact_text(&provider.node_provider_principal, COMPACT_PRINCIPAL_CHARS),
                optional_node_count_text(provider.node_count),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [ColumnAlign::Left, ColumnAlign::Right];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

#[must_use]
pub fn nns_node_provider_list_report_verbose_text(report: &NnsNodeProviderListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("fetched_by: {}", report.fetched_by));
    if report.node_providers.is_empty() {
        lines.push("node providers: none".to_string());
        return lines.join("\n");
    }

    let headers = [
        "NODE_PROVIDER",
        "NODES",
        "REWARD_ACCOUNT",
        "REGISTRY_VERSION",
        "FETCHED_AT",
    ];
    let rows = report
        .node_providers
        .iter()
        .map(|provider| {
            [
                provider.node_provider_principal.clone(),
                optional_node_count_text(provider.node_count),
                text_or_dash(provider.reward_account_hex.as_deref()).to_string(),
                report.registry_version.to_string(),
                report.fetched_at.clone(),
            ]
        })
        .collect::<Vec<_>>();
    let alignments = [
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
        ColumnAlign::Right,
        ColumnAlign::Left,
    ];
    lines.push(render_table(&headers, &rows, &alignments));
    lines.join("\n")
}

#[must_use]
pub fn nns_node_provider_info_report_text(report: &NnsNodeProviderInfoReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("input: {}", report.input));
    lines.push(format!("resolved_from: {}", report.resolved_from));
    lines.push(format!(
        "node_provider_principal: {}",
        report.node_provider_principal
    ));
    lines.push(format!(
        "node_count: {}",
        optional_node_count_text(report.node_count)
    ));
    lines.push(format!(
        "reward_account_hex: {}",
        text_or_dash(report.reward_account_hex.as_deref())
    ));
    lines.push(format!(
        "governance_canister_id: {}",
        report.governance_canister_id
    ));
    lines.push(format!(
        "registry_canister_id: {}",
        report.registry_canister_id
    ));
    lines.push(format!("registry_version: {}", report.registry_version));
    lines.push(format!("network: {}", report.network));
    lines.push(format!("fetched_at: {}", report.fetched_at));
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("fetched_by: {}", report.fetched_by));
    lines.join("\n")
}

#[must_use]
pub fn nns_node_provider_refresh_report_text(report: &NnsNodeProviderRefreshReport) -> String {
    nns_leaf_refresh_report_text(NnsLeafRefreshText {
        network: &report.network,
        cache_path: &report.cache_path,
        refresh_lock_path: &report.refresh_lock_path,
        governance_canister_id: Some(&report.governance_canister_id),
        registry_canister_id: &report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: &report.fetched_at,
        source_endpoint: &report.source_endpoint,
        fetched_by: &report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        count_label: "node_provider_count",
        count: report.node_provider_count,
    })
}
