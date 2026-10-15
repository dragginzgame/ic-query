//! Module: cloud_engine::provider::text
//!
//! Responsibility: render CloudEngine provider reports as compact human-facing text.
//! Does not own: JSON serialization, source validation, live calls, or process output.
//! Boundary: separates Dashboard provenance from provider and location tables.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use super::{
    CloudEngineProviderInfoReport, CloudEngineProviderListReport, CloudEngineProviderLocation,
};
use crate::{
    ic::dashboard_provenance_lines,
    text_value::{optional_text, yes_no},
};

/// Render the complete CloudEngine-bearing provider list.
#[must_use]
pub fn cloud_engine_provider_list_report_text(report: &CloudEngineProviderListReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend([
        format!(
            "source_node_provider_count: {}",
            report.source_node_provider_count
        ),
        format!(
            "cloud_engine_provider_count: {}",
            report.cloud_engine_provider_count
        ),
    ]);
    if !report.providers.is_empty() {
        let rows = report
            .providers
            .iter()
            .map(|provider| {
                [
                    provider.principal_id.clone(),
                    terminal_text(&provider.display_name),
                    provider.total_cloud_engines.to_string(),
                    provider.total_cloud_engine_nodes.to_string(),
                    provider.total_cloud_engine_unassigned_nodes.to_string(),
                    provider.cloud_engine_location_count.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        lines.push(String::new());
        lines.push("CloudEngine providers".to_string());
        lines.push(render(
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
            REPORT_TABLE_OPTIONS,
        ));
    }
    lines.join("\n")
}

/// Render one exact Dashboard provider record and its CloudEngine location evidence.
#[must_use]
pub fn cloud_engine_provider_info_report_text(report: &CloudEngineProviderInfoReport) -> String {
    let provider = &report.provider;
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend([
        format!(
            "cloud_engine_evidence_present: {}",
            yes_no(report.cloud_engine_evidence_present)
        ),
        format!("node_provider_id: {}", provider.principal_id),
        format!("display_name: {}", terminal_text(&provider.display_name)),
        format!("website: {}", optional_text(provider.website.as_ref())),
        format!("logo_url: {}", optional_text(provider.logo_url.as_ref())),
        format!("total_cloud_engines: {}", provider.total_cloud_engines),
        format!(
            "total_cloud_engine_nodes: {}",
            provider.total_cloud_engine_nodes
        ),
        format!(
            "total_cloud_engine_unassigned_nodes: {}",
            provider.total_cloud_engine_unassigned_nodes
        ),
        format!(
            "cloud_engine_location_count: {}",
            provider.cloud_engine_location_count
        ),
        format!("total_nodes: {}", provider.total_nodes),
        format!(
            "total_unassigned_nodes: {}",
            provider.total_unassigned_nodes
        ),
        format!(
            "total_rewardable_nodes: {}",
            provider.total_rewardable_nodes
        ),
        format!("total_node_allowance: {}", provider.total_node_allowance),
        format!("total_subnets: {}", provider.total_subnets),
        format!("location_count: {}", provider.location_count),
    ]);
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
                terminal_text(&location.display_name),
                terminal_text(location.owner.trim()),
                terminal_text(&location.region),
                location.latitude.to_string(),
                location.longitude.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    render(
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
        REPORT_TABLE_OPTIONS,
    )
}
