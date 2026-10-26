use ic_host_text::escape::terminal_text;

use crate::subnet_catalog::{SubnetCatalogListReport, text::principal::compact_principal};

pub(super) fn append_range_lines(report: &SubnetCatalogListReport, lines: &mut Vec<String>) {
    append_formatted_range_lines(report, lines, |value| terminal_text(value).into_owned());
}

pub(super) fn append_compact_range_lines(
    report: &SubnetCatalogListReport,
    lines: &mut Vec<String>,
) {
    append_formatted_range_lines(report, lines, compact_principal);
}

fn append_formatted_range_lines(
    report: &SubnetCatalogListReport,
    lines: &mut Vec<String>,
    principal: impl Fn(&str) -> String,
) {
    for subnet in &report.subnets {
        if subnet.ranges.is_empty() {
            continue;
        }
        lines.push(format!(
            "ranges for {}:",
            principal(&subnet.subnet_principal)
        ));
        for range in &subnet.ranges {
            lines.push(format!(
                "  {}..{}",
                principal(&range.start_canister_id),
                principal(&range.end_canister_id)
            ));
        }
        if subnet.ranges_shown < subnet.range_count {
            lines.push(format!(
                "  showing {} of {} ranges; use --range-limit or --json",
                subnet.ranges_shown, subnet.range_count
            ));
        }
    }
}
