use crate::text_value::{sanitize_text, yes_no};

#[must_use]
pub fn compact_text(value: &str, chars: usize) -> String {
    sanitize_text(value).chars().take(chars).collect()
}

#[must_use]
pub fn text_or_dash(value: Option<&str>) -> String {
    value
        .filter(|text| !text.is_empty())
        .map_or_else(|| "-".to_string(), sanitize_text)
}

#[must_use]
pub fn optional_f32_text(value: Option<f32>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

#[must_use]
pub fn optional_node_count_text(value: Option<u32>) -> String {
    value.map_or_else(|| "unknown".to_string(), |count| count.to_string())
}

///
/// NnsLeafRefreshText
///
/// Shared fields rendered for an NNS inventory refresh result.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NnsLeafRefreshText<'a> {
    pub network: &'a str,
    pub cache_path: &'a str,
    pub refresh_lock_path: &'a str,
    pub governance_canister_id: Option<&'a str>,
    pub registry_canister_id: &'a str,
    pub registry_version: u64,
    pub fetched_at: &'a str,
    pub source_endpoint: &'a str,
    pub fetched_by: &'a str,
    pub dry_run: bool,
    pub wrote_cache: bool,
    pub replaced_existing_cache: bool,
    pub count_label: &'a str,
    pub count: usize,
}

#[must_use]
pub fn nns_leaf_refresh_report_text(report: NnsLeafRefreshText<'_>) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(report.network)),
        format!("cache_path: {}", sanitize_text(report.cache_path)),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(report.refresh_lock_path)
        ),
    ];
    if let Some(governance_canister_id) = report.governance_canister_id {
        lines.push(format!(
            "governance_canister_id: {}",
            sanitize_text(governance_canister_id)
        ));
    }
    lines.extend([
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("fetched_at: {}", sanitize_text(report.fetched_at)),
        format!("source_endpoint: {}", sanitize_text(report.source_endpoint)),
        format!("fetched_by: {}", sanitize_text(report.fetched_by)),
        format!("dry_run: {}", yes_no(report.dry_run)),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("{}: {}", sanitize_text(report.count_label), report.count),
    ]);
    lines.join("\n")
}
