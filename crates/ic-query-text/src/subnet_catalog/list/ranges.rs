use ic_host_text::escape::terminal_text;

use crate::subnet_catalog::compact_principal;
use ic_query::subnet_catalog::SubnetCatalogListReport;

pub(super) fn append_range_lines(
    report: &SubnetCatalogListReport,
    lines: &mut Vec<String>,
    verbose: bool,
) {
    let principal = |value: &str| {
        if verbose {
            terminal_text(value).into_owned()
        } else {
            compact_principal(value)
        }
    };
    for subnet in &report.subnets {
        if subnet.ranges.is_empty() {
            continue;
        }
        lines.push(String::new());
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
