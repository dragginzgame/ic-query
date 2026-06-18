//! Module: nns::topology::report::text::capacity
//!
//! Responsibility: render NNS topology capacity reports as text.
//! Does not own: capacity calculation, source reads, or JSON output.
//! Boundary: combines summary and attention capacity tables for humans.

mod attention;
mod summary;

use crate::nns::topology::report::NnsTopologyCapacityReport;

#[must_use]
pub fn nns_topology_capacity_report_text(report: &NnsTopologyCapacityReport) -> String {
    let lines = [
        summary::render_capacity_summary_table(report),
        String::new(),
        attention::render_capacity_attention_table(report),
    ];
    lines.join("\n")
}
