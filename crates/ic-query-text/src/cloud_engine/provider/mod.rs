//! Module: cloud_engine::provider
//!
//! Responsibility: render CloudEngine provider reports as compact human-facing text.
//! Does not own: JSON serialization, source validation, live calls, or process output.
//! Boundary: separates Dashboard provenance from provider and location tables.

use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::ic::dashboard_provenance_lines;
use crate::text_value::optional_text;
use crate::text_value::yes_no;
use ic_query::cloud_engine::CloudEngineProviderInfoReport;
use ic_query::cloud_engine::CloudEngineProviderListReport;
use ic_query::cloud_engine::CloudEngineProviderLocation;

/// Render the complete CloudEngine-bearing provider list.
#[must_use]
pub fn cloud_engine_provider_list_report_text(report: &CloudEngineProviderListReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        source_node_provider_count,
        cloud_engine_provider_count = report.providers.len(),
    ));
    if !report.providers.is_empty() {
        let rows = report
            .providers
            .iter()
            .map(|provider| {
                [
                    provider.principal_id.clone(),
                    terminal_text(&provider.display_name).into_owned(),
                    provider.total_cloud_engines.to_string(),
                    provider.total_cloud_engine_nodes.to_string(),
                    provider.total_cloud_engine_unassigned_nodes.to_string(),
                    provider.cloud_engine_location_count.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        lines.push(String::new());
        lines.push("CloudEngine providers".to_string());
        lines.push(table(
            &[
                "Provider",
                "Name",
                "Engines",
                "CE nodes",
                "CE unassigned",
                "CE locations",
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

/// Render one exact Dashboard provider record and its CloudEngine location evidence.
#[must_use]
pub fn cloud_engine_provider_info_report_text(report: &CloudEngineProviderInfoReport) -> String {
    let provider = &report.provider;
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend(crate::text_value::report_fields!(report;
        cloud_engine_evidence_present = yes_no(report.cloud_engine_evidence_present),
        node_provider_id = provider.principal_id,
        display_name = provider.display_name,
        website = optional_text(provider.website.as_ref()),
        logo_url = optional_text(provider.logo_url.as_ref()),
        total_cloud_engines = provider.total_cloud_engines,
        total_cloud_engine_nodes = provider.total_cloud_engine_nodes,
        total_cloud_engine_unassigned_nodes = provider.total_cloud_engine_unassigned_nodes,
        cloud_engine_location_count = provider.cloud_engine_location_count,
        total_nodes = provider.total_nodes,
        total_unassigned_nodes = provider.total_unassigned_nodes,
        total_rewardable_nodes = provider.total_rewardable_nodes,
        total_node_allowance = provider.total_node_allowance,
        total_subnets = provider.total_subnets,
        location_count = provider.location_count,
    ));
    if !provider.cloud_engine_locations.is_empty() {
        lines.push(String::new());
        lines.push("CloudEngine locations".to_string());
        lines.push(location_table(&provider.cloud_engine_locations));
    }
    lines.join("\n")
}

fn location_table(locations: &[CloudEngineProviderLocation]) -> String {
    let rows = locations
        .iter()
        .map(|location| {
            [
                location.dc_key.clone(),
                terminal_text(&location.display_name).into_owned(),
                terminal_text(location.owner.trim()).into_owned(),
                terminal_text(&location.region).into_owned(),
                location.latitude.to_string(),
                location.longitude.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    table(
        &["DC", "Location", "Owner", "Region", "Latitude", "Longitude"],
        &rows,
        &[
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
        ],
    )
}

#[cfg(test)]
mod tests;
