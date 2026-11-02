//! Shared NNS text-rendering primitives for inventory and refresh reports.

use ic_host_text::escape::terminal_text;

#[cfg(feature = "nns-host")]
use crate::text_value::yes_no;

#[must_use]
pub fn compact_text(value: &str, chars: usize) -> String {
    crate::text_value::compact_text(value, chars)
}

#[must_use]
pub fn text_or_dash(value: Option<&str>) -> String {
    crate::text_value::text_or_dash(value.unwrap_or_default())
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
pub fn nns_inventory_refresh_report_text(
    report: &ic_query::nns::NnsInventoryRefreshReport,
) -> String {
    let [
        registry_id,
        version,
        network,
        fetched_at,
        endpoint,
        fetched_by,
    ] = registry_provenance_lines(&report.provenance);
    let mut lines = vec![
        network,
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
        registry_id,
        version,
        fetched_at,
        endpoint,
        fetched_by,
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
    provenance: &ic_query::nns::registry::NnsRegistryInventoryProvenance,
) -> [String; 6] {
    [
        format!("registry_canister_id: {}", provenance.registry_canister_id),
        format!("registry_version: {}", provenance.registry_version),
        format!("network: {}", terminal_text(&provenance.network)),
        format!("fetched_at: {}", terminal_text(&provenance.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&provenance.source_endpoint)
        ),
        format!("fetched_by: {}", terminal_text(&provenance.fetched_by)),
    ]
}
