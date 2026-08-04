//! Module: system::cmc::text
//!
//! Responsibility: render compact human-facing CMC reports.
//! Does not own: report construction, JSON output, live calls, or process output.
//! Boundary: keeps raw machine values in report models while formatting XDR only for text.

use super::{CmcCertification, CmcCyclesReport, CmcReportContext, CmcXdrReport};
use crate::{
    human_quantity::{byte_count_text, cycle_count_text},
    text_value::sanitize_text,
};

/// Render one certified CMC ICP/XDR report.
#[must_use]
pub fn cmc_xdr_report_text(report: &CmcXdrReport) -> String {
    let mut lines = context_lines(&report.context);
    lines.extend([
        format!("timestamp_seconds: {}", report.rate.timestamp_seconds),
        format!(
            "xdr_permyriad_per_icp: {}",
            report.rate.xdr_permyriad_per_icp
        ),
        format!(
            "xdr_per_icp: {}",
            permyriad_text(report.rate.xdr_permyriad_per_icp)
        ),
    ]);
    lines.extend(certification_lines(&report.certification));
    lines.join("\n")
}

/// Render one cycles conversion report derived from a certified CMC rate.
#[must_use]
pub fn cmc_cycles_report_text(report: &CmcCyclesReport) -> String {
    let mut lines = context_lines(&report.context);
    lines.extend([
        format!("timestamp_seconds: {}", report.rate.timestamp_seconds),
        format!(
            "xdr_permyriad_per_icp: {}",
            report.rate.xdr_permyriad_per_icp
        ),
        format!(
            "xdr_per_icp: {}",
            permyriad_text(report.rate.xdr_permyriad_per_icp)
        ),
        format!(
            "cycles_per_xdr: {}",
            cycle_count_text(report.cycles_per_xdr)
        ),
        format!("cycles_per_xdr_source: {}", report.cycles_per_xdr_source),
        format!(
            "cycles_per_icp: {}",
            cycle_count_text(report.cycles_per_icp)
        ),
        format!("cycles_per_icp_formula: {}", report.cycles_per_icp_formula),
    ]);
    lines.extend(certification_lines(&report.certification));
    lines.join("\n")
}

fn context_lines(context: &CmcReportContext) -> Vec<String> {
    vec![
        format!("network: {}", sanitize_text(&context.network)),
        format!("cmc_canister_id: {}", context.cmc_canister_id),
        format!("fetched_at: {}", sanitize_text(&context.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&context.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&context.fetched_by)),
    ]
}

fn certification_lines(certification: &CmcCertification) -> [String; 3] {
    [
        format!(
            "certificate_verified: {}",
            certification.certificate_verified
        ),
        format!(
            "certificate_bytes: {}",
            byte_count_text(certification.certificate_bytes as u128)
        ),
        format!(
            "hash_tree_bytes: {}",
            byte_count_text(certification.hash_tree_bytes as u128)
        ),
    ]
}

fn permyriad_text(value: u64) -> String {
    format!("{}.{:04}", value / 10_000, value % 10_000)
}
