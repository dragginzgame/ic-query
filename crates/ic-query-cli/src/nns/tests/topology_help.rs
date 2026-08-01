use super::*;

#[test]
fn topology_help_is_advertised_under_nns() {
    let nns = usage();
    let topology = topology_usage();
    let summary = topology_summary_usage();
    let coverage = topology_coverage_usage();
    let versions = topology_versions_usage();
    let health = topology_health_usage();
    let gaps = topology_gaps_usage();
    let capacity = topology_capacity_usage();
    let regions = topology_regions_usage();
    let providers = topology_providers_usage();
    let refresh = topology_refresh_usage();

    assert!(nns.contains("topology"));
    assert!(topology.contains("Summarize cached mainnet NNS topology reports"));
    assert!(topology.contains("Show cached mainnet NNS topology join coverage"));
    assert!(topology.contains("Show cached mainnet NNS topology component registry versions"));
    assert!(topology.contains("Check cached mainnet NNS topology cache health"));
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
    assert!(health.contains("icq nns topology health"));
    assert!(health.contains("--json"));
    assert!(health.contains("--source-endpoint"));
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
