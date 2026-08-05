use super::*;
use crate::cli::clap::render_help;

#[test]
fn topology_help_is_advertised_under_nns() {
    let nns = render_help(command());
    let topology = render_help(topology_command());
    let summary = render_help(topology_summary_command());
    let coverage = render_help(topology_coverage_command());
    let versions = render_help(topology_versions_command());
    let check = render_help(topology_check_command());
    let gaps = render_help(topology_gaps_command());
    let capacity = render_help(topology_capacity_command());
    let regions = render_help(topology_regions_command());
    let providers = render_help(topology_providers_command());
    let refresh = render_help(topology_refresh_command());

    assert!(nns.contains("topology"));
    assert!(topology.contains("Summarize cached mainnet NNS topology reports"));
    assert!(topology.contains("Show cached mainnet NNS topology join coverage"));
    assert!(topology.contains("Show cached mainnet NNS topology component registry versions"));
    assert!(topology.contains("Check cached mainnet NNS topology consistency"));
    assert!(topology.contains("List cached mainnet NNS topology join gaps"));
    assert!(topology.contains("Show cached mainnet NNS node-operator capacity"));
    assert!(topology.contains("Summarize cached mainnet NNS topology by region"));
    assert!(topology.contains("Summarize cached mainnet NNS topology by node provider"));
    assert!(topology.contains("Refresh cached mainnet NNS topology component reports"));
    assert!(summary.contains("icq nns topology summary"));
    assert!(summary.contains("Collection mode: Cache-backed read"));
    assert!(summary.contains("--json"));
    assert!(summary.contains("--source-endpoint"));
    assert!(coverage.contains("icq nns topology coverage"));
    assert!(coverage.contains("--json"));
    assert!(coverage.contains("--source-endpoint"));
    assert!(versions.contains("icq nns topology versions"));
    assert!(versions.contains("--json"));
    assert!(versions.contains("--source-endpoint"));
    assert!(check.contains("icq nns topology check"));
    assert!(check.contains("--json"));
    assert!(check.contains("--source-endpoint"));
    assert!(gaps.contains("icq nns topology gaps"));
    assert!(gaps.contains("--json"));
    assert!(gaps.contains("--source-endpoint"));
    assert!(capacity.contains("icq nns topology capacity"));
    assert!(capacity.contains("--json"));
    assert!(capacity.contains("--source-endpoint"));
    assert!(regions.contains("icq nns topology regions"));
    assert!(regions.contains("--json"));
    assert!(regions.contains("--source-endpoint"));
    assert!(providers.contains("icq nns topology providers"));
    assert!(providers.contains("--json"));
    assert!(providers.contains("--source-endpoint"));
    assert!(refresh.contains("icq nns topology refresh"));
    assert!(refresh.contains("Collection mode: Forced live refresh"));
    assert!(refresh.contains("--json"));
    assert!(refresh.contains("--source-endpoint"));
    assert!(refresh.contains("--lock-stale-after"));
    assert!(refresh.contains("--dry-run"));
}
