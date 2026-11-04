//! Module: system::cmc
//!
//! Responsibility: render compact human-facing CMC reports.
//! Does not own: report construction, JSON output, live calls, or process output.
//! Boundary: keeps raw machine values in report models while formatting XDR only for text.

use crate::text_value::field;

use crate::human_quantity::byte_count_text;
use crate::human_quantity::cycle_count_text;
use ic_query::system::cmc::CmcCertification;
use ic_query::system::cmc::CmcCyclesReport;
use ic_query::system::cmc::CmcIcpXdrConversionRate;
use ic_query::system::cmc::CmcReportContext;
use ic_query::system::cmc::CmcXdrReport;

/// Render one certified CMC ICP/XDR report.
#[must_use]
pub fn cmc_xdr_report_text(report: &CmcXdrReport) -> String {
    let mut lines = context_lines(&report.context);
    lines.extend(rate_lines(&report.rate));
    lines.extend(certification_lines(&report.certification));
    lines.join("\n")
}

/// Render one cycles conversion report derived from a certified CMC rate.
#[must_use]
pub fn cmc_cycles_report_text(report: &CmcCyclesReport) -> String {
    let mut lines = context_lines(&report.xdr.context);
    lines.extend(rate_lines(&report.xdr.rate));
    lines.extend([
        field("cycles_per_xdr", &(cycle_count_text(report.cycles_per_xdr))),
        field("cycles_per_xdr_source", &(report.cycles_per_xdr_source)),
        field("cycles_per_icp", &(cycle_count_text(report.cycles_per_icp))),
        field("cycles_per_icp_formula", &(report.cycles_per_icp_formula)),
    ]);
    lines.extend(certification_lines(&report.xdr.certification));
    lines.join("\n")
}

fn rate_lines(rate: &CmcIcpXdrConversionRate) -> [String; 3] {
    [
        field("timestamp_seconds", &(rate.timestamp_seconds)),
        field("xdr_permyriad_per_icp", &(rate.xdr_permyriad_per_icp)),
        field("xdr_per_icp", &(permyriad_text(rate.xdr_permyriad_per_icp))),
    ]
}

fn context_lines(context: &CmcReportContext) -> Vec<String> {
    vec![
        field("network", &context.network),
        field("cmc_canister_id", &(context.cmc_canister_id)),
        field("fetched_at", &context.fetched_at),
        field("source_endpoint", &context.source_endpoint),
        field("fetched_by", &context.fetched_by),
    ]
}

fn certification_lines(certification: &CmcCertification) -> [String; 3] {
    [
        field(
            "certificate_verified",
            &(certification.certificate_verified),
        ),
        field(
            "certificate_bytes",
            &(byte_count_text(certification.certificate_bytes as u128)),
        ),
        field(
            "hash_tree_bytes",
            &(byte_count_text(certification.hash_tree_bytes as u128)),
        ),
    ]
}

fn permyriad_text(value: u64) -> String {
    format!("{}.{:04}", value / 10_000, value % 10_000)
}
