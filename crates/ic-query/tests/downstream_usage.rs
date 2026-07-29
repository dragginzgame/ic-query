use ic_query::nns::registry::{
    NnsRegistryVersionReport, NnsRegistryVersionRequest, nns_registry_version_report_text,
};

#[test]
fn downstream_no_default_rendering_example_uses_public_api() {
    let text = render_registry_version();

    assert!(text.contains("network: ic"));
    assert!(text.contains("registry_version: 42"));
}

fn render_registry_version() -> String {
    let request = NnsRegistryVersionRequest::new("ic", "https://icp-api.io", 1_700_000_000);

    let report = NnsRegistryVersionReport {
        schema_version: 1,
        network: request.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "my-tool".to_string(),
    };

    nns_registry_version_report_text(&report)
}

#[cfg(feature = "host")]
mod host {
    use ic_query::{
        QueryProgress, QueryProgressEvent, QueryProgressState,
        icrc::{
            DEFAULT_ICRC_SOURCE_ENDPOINT, IcrcError, IcrcTokenRequest, build_icrc_token_report,
            icrc_token_report_text,
        },
        nns::NnsSourceRequest,
        nns::node::{
            DEFAULT_NNS_NODE_SOURCE_ENDPOINT, NNS_NODE_SUBNET_KIND_APPLICATION,
            NnsNodeCacheRequest, NnsNodeHostError, NnsNodeListRequest, build_nns_node_list_report,
            nns_node_list_report_text,
        },
        nns::proposals::{
            NnsProposalHostError, NnsProposalRefreshReport, NnsProposalRefreshRequest,
            refresh_nns_proposal_cache_with_progress,
        },
        nns::registry::{
            NnsRegistryHostError, NnsRegistrySource, NnsRegistryVersionData,
            NnsRegistryVersionRequest, build_nns_registry_version_report_with_source,
            nns_registry_version_report_text,
        },
        nns::topology::{
            CachedNnsSubnetTopologyReport, DEFAULT_NNS_SUBNET_TOPOLOGY_REFRESH_LOCK_STALE_SECONDS,
            DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT, NnsSubnetTopologyCacheRequest,
            NnsSubnetTopologyHostError, NnsSubnetTopologyRefreshRequest,
            refresh_nns_subnet_topology,
        },
        sns::{
            DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsNeuronsCacheStatusRequest,
            SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest, SnsNeuronsRequest, SnsNeuronsSort,
            SnsProposalSortDirection, SnsProposalsCacheStatusRequest, SnsProposalsRefreshReport,
            SnsProposalsRefreshRequest, SnsProposalsReport, SnsProposalsRequest, SnsProposalsSort,
            build_sns_neurons_cache_status_report, build_sns_neurons_report,
            build_sns_proposals_cache_status_report, build_sns_proposals_report,
            build_sns_proposals_report_with_progress, refresh_sns_neurons_cache_with_progress,
            refresh_sns_proposals_cache_with_progress, sns_neurons_cache_status_report_text,
            sns_neurons_report_text, sns_proposals_cache_status_report_text,
            sns_proposals_report_text,
        },
        subnet_catalog::{
            DEFAULT_STALE_AFTER_SECONDS, DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, ResolveAs,
            SubnetCatalogCacheRequest, SubnetCatalogHostError, SubnetCatalogInfoRequest,
            build_subnet_catalog_info_report, subnet_catalog_info_report_text,
        },
    };
    use std::path::Path;

    #[test]
    fn downstream_host_examples_typecheck_public_builders() {
        accepts_subnet_example(render_subnet_info);
        accepts_nns_node_example(render_application_nodes);
        accepts_icrc_example(render_token);
        accepts_custom_source_example(render_registry_version_with_source);
        accepts_subnet_topology_example(refresh_subnet_topology);
        accepts_sns_proposals_example(render_recent_sns_proposals);
        accepts_sns_neurons_example(render_cached_sns_neurons);
        accepts_sns_cache_status_example(render_sns_cache_status);
        assert_progress_api_is_public();

        let text = render_registry_version_with_source(&FixtureRegistrySource, 1_700_000_000)
            .expect("custom registry source");
        assert!(text.contains("registry_version: 42"));
    }

    fn render_subnet_info(
        project_root: &Path,
        canister_or_subnet: &str,
        now_unix_secs: u64,
    ) -> Result<String, SubnetCatalogHostError> {
        let cache = SubnetCatalogCacheRequest::new(project_root, "ic");
        let request = SubnetCatalogInfoRequest::new(
            cache,
            DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            canister_or_subnet,
            now_unix_secs,
            DEFAULT_STALE_AFTER_SECONDS,
        )
        .with_forced(ResolveAs::Canister);

        let report = build_subnet_catalog_info_report(&request)?;
        Ok(subnet_catalog_info_report_text(&report))
    }

    fn assert_progress_api_is_public() {
        let mut events = Vec::new();
        let mut progress = |event| events.push(event);
        progress.report(QueryProgressEvent::PagedRefresh {
            text: "refreshing".to_string(),
            state: QueryProgressState::Running,
        });
        assert_eq!(events.len(), 1);

        let _: fn(
            &NnsProposalRefreshRequest,
            &mut dyn QueryProgress,
        ) -> Result<NnsProposalRefreshReport, NnsProposalHostError> =
            refresh_nns_proposal_cache_with_progress;
        let _: fn(
            &SnsNeuronsRefreshRequest,
            &mut dyn QueryProgress,
        ) -> Result<SnsNeuronsRefreshReport, SnsHostError> =
            refresh_sns_neurons_cache_with_progress;
        let _: fn(
            &SnsProposalsRefreshRequest,
            &mut dyn QueryProgress,
        ) -> Result<SnsProposalsRefreshReport, SnsHostError> =
            refresh_sns_proposals_cache_with_progress;
        let _: fn(
            &SnsProposalsRequest,
            &mut dyn QueryProgress,
        ) -> Result<SnsProposalsReport, SnsHostError> = build_sns_proposals_report_with_progress;
    }

    fn render_application_nodes(
        project_root: &Path,
        now_unix_secs: u64,
    ) -> Result<String, NnsNodeHostError> {
        let cache = NnsNodeCacheRequest::new(project_root, "ic");
        let request =
            NnsNodeListRequest::new(cache, DEFAULT_NNS_NODE_SOURCE_ENDPOINT, now_unix_secs)
                .with_subnet_kind(NNS_NODE_SUBNET_KIND_APPLICATION);

        let report = build_nns_node_list_report(&request)?;
        Ok(nns_node_list_report_text(&report))
    }

    fn render_token(ledger_canister_id: &str, now_unix_secs: u64) -> Result<String, IcrcError> {
        let request = IcrcTokenRequest::new(
            DEFAULT_ICRC_SOURCE_ENDPOINT,
            now_unix_secs,
            ledger_canister_id,
        );
        let report = build_icrc_token_report(&request)?;
        Ok(icrc_token_report_text(&report))
    }

    fn render_registry_version_with_source(
        source: &dyn NnsRegistrySource,
        now_unix_secs: u64,
    ) -> Result<String, NnsRegistryHostError> {
        let request = NnsRegistryVersionRequest::new("ic", "https://mirror.example", now_unix_secs);
        let report = build_nns_registry_version_report_with_source(&request, source)?;
        Ok(nns_registry_version_report_text(&report))
    }

    fn render_recent_sns_proposals(
        project_root: &Path,
        sns_input: &str,
        now_unix_secs: u64,
    ) -> Result<String, SnsHostError> {
        let request = SnsProposalsRequest::new(
            "ic",
            DEFAULT_SNS_SOURCE_ENDPOINT,
            now_unix_secs,
            sns_input,
            25,
        )
        .with_icp_root(project_root)
        .with_sort(SnsProposalsSort::Created)
        .with_sort_direction(SnsProposalSortDirection::Desc);

        let report = build_sns_proposals_report(&request)?;
        Ok(sns_proposals_report_text(&report))
    }

    fn refresh_subnet_topology(
        project_root: &Path,
        now_unix_secs: u64,
    ) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
        let request = NnsSubnetTopologyRefreshRequest::new(
            NnsSubnetTopologyCacheRequest::new(project_root, "ic"),
            DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT,
            now_unix_secs,
            DEFAULT_NNS_SUBNET_TOPOLOGY_REFRESH_LOCK_STALE_SECONDS,
        );
        refresh_nns_subnet_topology(&request)
    }

    fn render_cached_sns_neurons(
        project_root: &Path,
        sns_input: &str,
        now_unix_secs: u64,
    ) -> Result<String, SnsHostError> {
        let request = SnsNeuronsRequest::new(
            "ic",
            DEFAULT_SNS_SOURCE_ENDPOINT,
            now_unix_secs,
            sns_input,
            500,
        )
        .with_icp_root(project_root)
        .with_sort(SnsNeuronsSort::Stake);

        let report = build_sns_neurons_report(&request)?;
        Ok(sns_neurons_report_text(&report))
    }

    fn render_sns_cache_status(
        project_root: &Path,
        sns_input: &str,
    ) -> Result<String, SnsHostError> {
        let proposals = SnsProposalsCacheStatusRequest::new(project_root, "ic", sns_input);
        let proposals_report = build_sns_proposals_cache_status_report(&proposals)?;

        let neurons = SnsNeuronsCacheStatusRequest::new(project_root, "ic", sns_input);
        let neurons_report = build_sns_neurons_cache_status_report(&neurons)?;

        Ok(format!(
            "{}\n{}",
            sns_proposals_cache_status_report_text(&proposals_report),
            sns_neurons_cache_status_report_text(&neurons_report)
        ))
    }

    struct FixtureRegistrySource;

    impl NnsRegistrySource for FixtureRegistrySource {
        fn fetch_registry_version(
            &self,
            request: &NnsSourceRequest,
        ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
            Ok(NnsRegistryVersionData {
                network: "ic".to_string(),
                registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
                registry_version: 42,
                fetched_at: request.fetched_at.clone(),
                fetched_by: request.fetched_by.clone(),
                source_endpoint: request.endpoint.clone(),
            })
        }
    }

    fn accepts_subnet_example(
        _example: fn(&Path, &str, u64) -> Result<String, SubnetCatalogHostError>,
    ) {
    }

    fn accepts_nns_node_example(_example: fn(&Path, u64) -> Result<String, NnsNodeHostError>) {}

    fn accepts_icrc_example(_example: fn(&str, u64) -> Result<String, IcrcError>) {}

    fn accepts_custom_source_example(
        _example: fn(&dyn NnsRegistrySource, u64) -> Result<String, NnsRegistryHostError>,
    ) {
    }

    fn accepts_sns_proposals_example(
        _example: fn(&Path, &str, u64) -> Result<String, SnsHostError>,
    ) {
    }

    fn accepts_subnet_topology_example(
        _example: fn(
            &Path,
            u64,
        ) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError>,
    ) {
    }

    fn accepts_sns_neurons_example(_example: fn(&Path, &str, u64) -> Result<String, SnsHostError>) {
    }

    fn accepts_sns_cache_status_example(_example: fn(&Path, &str) -> Result<String, SnsHostError>) {
    }
}
