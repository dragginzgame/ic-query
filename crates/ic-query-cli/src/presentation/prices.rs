//! Compact daily prices; raw report values remain available through JSON.

use ic_host_text::table::render_bordered;
use ic_topology::cloud_engine::prices::{CloudEnginePricesReport, FeeOutcome};
use std::fmt::Write;

pub(crate) fn cloud_engine_prices(report: &CloudEnginePricesReport) -> String {
    let prices = &report.prices;
    let mut text = format!(
        "Cloud-engine marketplace\n\n{} price declarations | {:.2}s | {}\nWhole cycles per node per day\n",
        prices.entries.len(),
        report.elapsed.as_secs_f64(),
        if report.complete {
            "complete"
        } else {
            "partial"
        },
    );
    match &report.network_fee {
        FeeOutcome::Read { markup, evidence } => {
            writeln!(text, "Network markup on net: {markup} (gross prices already include their fee)\nFee observed at: {} Unix seconds", evidence.observed_at_unix_seconds).expect("String write");
        }
        FeeOutcome::Failed { .. } => text.push_str(
            "Network markup unavailable; prices retained. See --json for the failure category.\n",
        ),
        FeeOutcome::OverallDeadline => text
            .push_str("Network markup unavailable: overall deadline reached; prices retained.\n"),
    }
    let rows: Vec<_> = prices
        .entries
        .iter()
        .map(|row| {
            vec![
                row.net_cycles_per_node_per_day.clone(),
                row.gross_cycles_per_node_per_day.clone(),
                format!("{:?}", row.key),
            ]
        })
        .collect();
    text.push('\n');
    text.push_str(&render_bordered(
        &["NET / NODE / DAY", "GROSS / NODE / DAY", "MARKETPLACE KEY"],
        &rows,
    ));
    text.push('\n');
    if prices.entries.is_empty() {
        text.push_str("(no declarations returned)\n");
    }
    writeln!(text, "\nSource: {} / {}\nMethods: {} + getNetworkFee\nPrices observed at: {} Unix seconds\nQuery signatures verified: {} | Consensus-certified: {}",
        prices.evidence.endpoint, prices.evidence.source_canister, prices.evidence.source_method,
        prices.evidence.observed_at_unix_seconds, prices.evidence.query_signature_verified,
        prices.evidence.result_certified,
    ).expect("String write");
    text.push_str("Keys identify [provider,]class[,data center]; declarations are not binding quotes or available capacity.\nSeparate observations, not an atomic snapshot. Use --json for raw classes, scopes, timestamps and evidence.\n");
    text
}
