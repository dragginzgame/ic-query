//! Compact availability summaries; full page/node evidence stays in JSON.

use ic_host_text::table::render_bordered;
use ic_topology::cloud_engine::available::{AvailableNodesReport, TraversalStop};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

pub(crate) fn cloud_engine_available(report: &AvailableNodesReport) -> String {
    let mut groups = BTreeMap::new();
    let mut node_count = 0;
    for node in report.pages.iter().flat_map(|page| &page.nodes) {
        // Render the external unit-variant's exact Serde tag, with safe quoting.
        let class = serde_json::to_string(&node.node_type).expect("unit variant serialization");
        let group = groups
            .entry((class, node.data_center.as_deref()))
            .or_insert((0usize, 0usize, BTreeSet::new()));
        group.0 += 1;
        group.1 += usize::from(node.sev_enabled);
        group.2.insert(node.provider_id);
        node_count += 1;
    }
    let stop = match report.stop {
        TraversalStop::Exhausted => "pagination exhausted",
        TraversalStop::PageLimit => "partial: page limit reached",
        TraversalStop::NodeLimit => "partial: node limit reached",
        TraversalStop::Failed { .. } => "partial: page failed or was refused (see --json)",
        TraversalStop::OverallDeadline { .. } => "partial: overall deadline reached",
    };
    let copy = &report.registry_copy;
    let mut text = format!(
        "Cloud-engine candidate nodes\n\n{node_count} nodes | {} admitted pages | {:.2}s | {stop}\nRegistry copy status: {} / {} | caught up: {}\nStatus observed at: {} Unix seconds\n",
        report.pages.len(),
        report.elapsed.as_secs_f64(),
        copy.version,
        copy.latest,
        copy.caught_up,
        copy.observed_at_unix_seconds,
    );
    if let Some(first) = report.pages.first() {
        writeln!(text, "Page Registry version: {}", first.registry_version).expect("String write");
    }
    writeln!(
        text,
        "Collection budget: {} pages / {} nodes / 60 seconds",
        report.page_limit, report.node_limit
    )
    .expect("String write");
    let rows: Vec<_> = groups
        .into_iter()
        .map(|((class, dc), (nodes, chips, providers))| {
            let dc = dc.map_or_else(|| "(absent)".to_owned(), |raw| format!("{raw:?}"));
            vec![
                class,
                nodes.to_string(),
                providers.len().to_string(),
                chips.to_string(),
                dc,
            ]
        })
        .collect();
    text.push('\n');
    text.push_str(&render_bordered(
        &["RAW CLASS", "NODES", "PROVIDERS", "CHIP ID", "DATA CENTER"],
        &rows,
    ));
    text.push('\n');
    if node_count == 0 {
        text.push_str(if report.pagination_exhausted {
            "(no nodes returned)\n"
        } else {
            "(no node pages admitted; availability unknown)\n"
        });
    }
    writeln!(text, "\nSource: {} / {}\nMethods: {} + {}\nQuery signatures verified for received observations: {} | Consensus-certified: {}",
        report.endpoint, report.source_canister, copy.source_method, report.source_method,
        report.query_signature_verified, report.result_certified,
    ).expect("String write");
    text.push_str("New-engine eligibility only. Pages are read afresh; no atomic inventory or reserved capacity.\nCHIP ID counts the source's registration flag, not verified TEE protection or liveness.\nUse --json for node identities, operators/providers, page cursors, timing and stop details.\n");
    text
}
