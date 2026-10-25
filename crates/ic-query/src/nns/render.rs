//! Shared NNS text-rendering primitives for inventory and refresh reports.

use ic_host_text::escape::terminal_text;

#[cfg(feature = "nns-host")]
use crate::text_value::yes_no;

#[must_use]
pub fn compact_text(value: &str, chars: usize) -> String {
    terminal_text(value).chars().take(chars).collect()
}

#[must_use]
pub fn text_or_dash(value: Option<&str>) -> String {
    value.filter(|text| !text.is_empty()).map_or_else(
        || "-".to_string(),
        |value| terminal_text(value).into_owned(),
    )
}

#[must_use]
pub fn optional_f32_text(value: Option<f32>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

#[must_use]
pub fn optional_node_count_text(value: Option<u32>) -> String {
    value.map_or_else(|| "unknown".to_string(), |count| count.to_string())
}

#[cfg(feature = "nns-host")]
#[must_use]
/// Render a Registry inventory refresh publication outcome.
pub fn nns_inventory_refresh_report_text(report: &crate::nns::NnsInventoryRefreshReport) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("cache_path: {}", terminal_text(&report.cache_path)),
        format!(
            "refresh_lock_path: {}",
            terminal_text(&report.refresh_lock_path)
        ),
    ];
    if let Some(governance_canister_id) = &report.governance_canister_id {
        lines.push(format!(
            "governance_canister_id: {}",
            terminal_text(governance_canister_id)
        ));
    }
    lines.extend([
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", terminal_text(&report.fetched_by)),
        format!("dry_run: {}", yes_no(report.dry_run)),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("collection: {}", terminal_text(&report.collection)),
        format!("row_count: {}", report.row_count),
    ]);
    lines.join("\n")
}

#[must_use]
pub(super) fn registry_provenance_lines(
    registry_canister_id: &str,
    registry_version: u64,
    network: &str,
    fetched_at: &str,
    source_endpoint: &str,
    fetched_by: &str,
) -> [String; 6] {
    [
        format!("registry_canister_id: {registry_canister_id}"),
        format!("registry_version: {registry_version}"),
        format!("network: {}", terminal_text(network)),
        format!("fetched_at: {}", terminal_text(fetched_at)),
        format!("source_endpoint: {}", terminal_text(source_endpoint)),
        format!("fetched_by: {}", terminal_text(fetched_by)),
    ]
}
