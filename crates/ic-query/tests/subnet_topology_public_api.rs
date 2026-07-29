use candid::Principal;
use ic_query::{
    nns::topology::{
        NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION, NnsSubnetNodeProviderRow,
        NnsSubnetTopologyReport, NnsSubnetTopologyRow, nns_subnet_topology_report_text,
    },
    subnet_catalog::SubnetKind,
};

#[test]
fn public_subnet_topology_model_is_constructible_and_validatable_without_host() {
    let report = fixture_report(42, "2023-11-14T22:13:20Z", "https://icp-api.io");

    report.validate().expect("valid report");
    let text = nns_subnet_topology_report_text(&report);

    assert!(text.contains("registry_version 42"));
    assert!(text.contains("cloud_engine"));
}

#[cfg(feature = "host")]
mod host {
    use super::*;
    use ic_query::nns::topology::{
        CachedNnsSubnetTopologyReport, NnsSubnetTopologyCacheRequest, NnsSubnetTopologyHostError,
        NnsSubnetTopologyRefreshRequest, NnsSubnetTopologySource, load_cached_nns_subnet_topology,
        load_or_refresh_missing_nns_subnet_topology, load_or_refresh_stale_nns_subnet_topology,
        nns_subnet_topology_cache_path, nns_subnet_topology_freshness,
        refresh_nns_subnet_topology_with_source,
    };
    use ic_query::nns::{LiveNnsSource, NnsSourceRequest};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn public_live_source_rejects_non_mainnet_before_agent_construction() {
        let request = NnsSourceRequest::new(
            "local",
            "not a valid replica endpoint",
            "2026-07-29T00:00:00Z",
            "public-api-test",
        );

        let error = LiveNnsSource
            .fetch_subnet_topology_report(&request)
            .expect_err("unsupported network");

        assert!(matches!(
            error,
            NnsSubnetTopologyHostError::UnsupportedNetwork { network }
                if network == "local"
        ));
    }

    #[test]
    fn public_subnet_topology_host_api_accepts_custom_source_and_shared_cache_root() {
        let root = temp_root();
        let cache = NnsSubnetTopologyCacheRequest::new(&root, "ic");
        let request =
            NnsSubnetTopologyRefreshRequest::new(cache.clone(), "https://mirror.example", 100, 30);

        let refreshed = refresh_nns_subnet_topology_with_source(&request, &FixtureSource)
            .expect("refreshed report");
        let loaded = load_cached_nns_subnet_topology(&cache).expect("cached report");

        assert_eq!(refreshed.report.registry_version, 77);
        assert_eq!(loaded.report, refreshed.report);
        assert_eq!(loaded.path, nns_subnet_topology_cache_path(&root, "ic"));
        assert!(!nns_subnet_topology_freshness(&loaded.report, 100, 30).stale);

        let _: fn(
            &NnsSubnetTopologyRefreshRequest,
        ) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> =
            load_or_refresh_missing_nns_subnet_topology;
        let _: fn(
            &NnsSubnetTopologyRefreshRequest,
            u64,
        ) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> =
            load_or_refresh_stale_nns_subnet_topology;

        let _ = fs::remove_dir_all(root);
    }

    struct FixtureSource;

    impl NnsSubnetTopologySource for FixtureSource {
        fn fetch_subnet_topology_report(
            &self,
            request: &NnsSourceRequest,
        ) -> Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
            Ok(fixture_report(77, &request.fetched_at, &request.endpoint))
        }
    }

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "ic-query-subnet-topology-public-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }
}

fn fixture_report(
    registry_version: u64,
    fetched_at: &str,
    source_endpoint: &str,
) -> NnsSubnetTopologyReport {
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let provider = Principal::self_authenticating(b"provider").to_text();
    NnsSubnetTopologyReport {
        schema_version: NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
        network: "ic".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version,
        fetched_at: fetched_at.to_string(),
        source_endpoint: source_endpoint.to_string(),
        fetched_by: "fixture".to_string(),
        subnet_count: 1,
        node_count: 2,
        subnets: vec![NnsSubnetTopologyRow {
            subnet_principal: subnet,
            subnet_kind: SubnetKind::CloudEngine,
            node_count: 2,
            node_providers: vec![NnsSubnetNodeProviderRow {
                node_provider_principal: provider,
                node_count: 2,
            }],
        }],
    }
}
