//! Compact declaration coverage, without a selected or computed deployment price.

use ic_host_text::bordered_table;
use ic_query_topology::cloud_engine::{
    candidates::{CandidatePricing, CandidatesReport},
    prices::FeeOutcome,
};
use std::fmt::Write;

pub(crate) fn cloud_engine_candidates(report: &CandidatesReport) -> String {
    let available = &report.availability;
    let nodes: usize = available.pages.iter().map(|page| page.nodes.len()).sum();
    let mut text = format!(
        "Cloud-engine candidate price declarations\n\n{nodes} observed nodes | {} pages | {:.2}s | collection {}\nAvailability pagination: {} | budget: {} pages\n",
        available.pages.len(),
        report.elapsed.as_secs_f64(),
        if report.collection_complete {
            "complete"
        } else {
            "partial (see --json)"
        },
        if available.pagination_exhausted {
            "exhausted"
        } else {
            "not exhausted (see --json for stop reason)"
        },
        available.page_limit,
    );
    match &report.pricing {
        CandidatePricing::Failed { .. } => {
            text.push_str("Prices unavailable; matching coverage unknown (see --json).\n")
        }
        CandidatePricing::Read {
            report: prices,
            matches,
        } => {
            let zero = matches
                .iter()
                .filter(|node| node.price_indices.is_empty())
                .count();
            let one = matches
                .iter()
                .filter(|node| node.price_indices.len() == 1)
                .count();
            let many = matches.len() - zero - one;
            writeln!(
                text,
                "Declaration matches: {zero} none | {one} single | {many} multiple"
            )
            .expect("String write");
            let mut counts = vec![0usize; prices.prices.entries.len()];
            for node in matches {
                for &index in &node.price_indices {
                    counts[index] += 1;
                }
            }
            let rows: Vec<_> = prices
                .prices
                .entries
                .iter()
                .zip(counts)
                .filter(|(_, count)| *count > 0)
                .map(|(price, count)| {
                    vec![
                        format!("{:?}", price.key),
                        count.to_string(),
                        price.net_cycles_per_node_per_day.clone(),
                        price.gross_cycles_per_node_per_day.clone(),
                    ]
                })
                .collect();
            text.push('\n');
            text.push_str(&bordered_table(
                &[
                    "DECLARATION",
                    "MATCHING NODES",
                    "NET CYCLES/NODE/DAY",
                    "GROSS CYCLES/NODE/DAY",
                ],
                &rows,
            ));
            text.push('\n');
            if rows.is_empty() {
                text.push_str("(no matching public declarations)\n");
            }
            writeln!(
                text,
                "\nPrices observed at: {} Unix seconds",
                prices.prices.evidence.observed_at_unix_seconds
            )
            .expect("String write");
            if !matches!(prices.network_fee, FeeOutcome::Read { .. }) {
                text.push_str(
                    "Independent network fee unavailable; supplied gross amounts retained.\n",
                );
            }
        }
    }
    if let (Some(first), Some(last)) = (available.pages.first(), available.pages.last()) {
        writeln!(
            text,
            "Pages observed at: {}–{} Unix seconds | Registry version: {}",
            first.observed_at_unix_seconds, last.observed_at_unix_seconds, first.registry_version
        )
        .expect("String write");
    }
    writeln!(text, "\nSource: {} / {}\nMethods: registryCopyStatus + listAvailableNodesPage + listMarketplacePrices + getNetworkFee", available.endpoint, available.source_canister).expect("String write");
    text.push_str("Matching uses exact class/provider/data-center scopes. Counts overlap when declarations overlap.\nNo tariff selected: precedence and customer discounts are unobserved; even one match is not a quote.\nIndependent signature-verified queries, not consensus-certified or an atomic inventory.\nEligibility does not establish live or reserved capacity. Use --json for nodes, matches and source evidence.\n");

    text
}
