mod info;
mod list;
mod principal;
mod refresh;

pub use info::subnet_catalog_info_report_text;
pub use list::{subnet_catalog_list_report_text, subnet_catalog_list_report_verbose_text};
pub use refresh::subnet_catalog_refresh_report_text;

// These fields have identical meaning on the list and info report contracts.
macro_rules! append_catalog_authority {
    ($lines:expr, $report:expr) => {{
        let lines = &mut *$lines;
        let report = $report;
        lines.push(format!("assurance: {}", report.assurance.as_str()));
        lines.push(format!(
            "cache_disposition: {}",
            report.cache_disposition.as_str()
        ));
        lines.push(format!(
            "catalog_digest: {}",
            ic_host_text::escape::terminal_text(&report.catalog_digest)
        ));
        lines.push(format!(
            "source_endpoints: {}",
            report
                .source_endpoints
                .iter()
                .map(|endpoint| ic_host_text::escape::terminal_text(endpoint))
                .collect::<Vec<_>>()
                .join(",")
        ));
        lines.push(format!(
            "agreement_digest: {}",
            report.agreement_digest.as_deref().unwrap_or("-")
        ));
        lines.push(format!(
            "registry_query_call_count: {}",
            report.registry_query_call_count
        ));
        lines.push(format!(
            "routing_source: {}",
            report.routing_source.as_str()
        ));
        lines.push(format!(
            "registry_record_count: {}",
            report.registry_records.len()
        ));
    }};
}
pub(super) use append_catalog_authority;

// These fields have identical meaning on the list and info report contracts.
macro_rules! append_catalog_policy {
    ($lines:expr, $report:expr) => {{
        let lines = &mut *$lines;
        let report = $report;
        lines.push(format!(
            "collector_version: {}",
            ic_host_text::escape::terminal_text(&report.collector_version)
        ));
        lines.push(format!(
            "classification_schema_version: {}",
            report.classification_schema_version
        ));
        lines.push(format!(
            "classification_policy_digest: {}",
            ic_host_text::escape::terminal_text(&report.classification_policy_digest)
        ));
        lines.push(format!(
            "resolver_schema_version: {}",
            report.resolver_schema_version
        ));
        lines.push(format!(
            "resolver_backend: {}",
            ic_host_text::escape::terminal_text(&report.resolver_backend)
        ));
    }};
}
pub(super) use append_catalog_policy;
