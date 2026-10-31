#[cfg(feature = "certificate-host")]
use ic_host_tools::certificate::ParseError;
#[cfg(feature = "ledger-evidence-host")]
use ic_host_tools::ledger_evidence::LedgerTip;
#[cfg(feature = "ledger-evidence-host")]
use ic_query::certificate::{CertificateEvidenceLimits, authenticate_ledger_tip};
#[cfg(feature = "certificate-host")]
use ic_query::certificate::{CertificateVerificationError, authenticate};
use ic_query::nns::registry::{
    NnsRegistryCertification, NnsRegistryVersionReport, NnsRegistryVersionRequest,
    nns_registry_version_report_text,
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
        certification: registry_certification(),
    };

    nns_registry_version_report_text(&report)
}

fn registry_certification() -> NnsRegistryCertification {
    NnsRegistryCertification {
        certificate_verified: true,
        certificate_time_nanos: 1_700_000_000_000_000_000,
        certificate_time: "2023-11-14T22:13:20Z".to_string(),
        root_key_digest: "ab".repeat(32),
        certificate_hex: "cd".repeat(8),
        certificate_bytes: 8,
        hash_tree_hex: "ef".repeat(4),
        hash_tree_bytes: 4,
    }
}

#[cfg(feature = "host")]
mod host {
    use ic_query::report::{CollectionSourceRequest, SortDirection};
    use ic_query::{
        QueryProgress, QueryProgressEvent, QueryProgressState,
        icrc::{
            DEFAULT_ICRC_SOURCE_ENDPOINT, IcrcError, IcrcLedgerRequest, build_icrc_token_report,
            icrc_token_report_text,
        },
        nns::node::{
            DEFAULT_NNS_NODE_SOURCE_ENDPOINT, NnsNodeHostError, NnsNodeListRequest,
            build_nns_node_list_report, nns_node_list_report_text,
        },
        nns::proposals::{NnsProposalHostError, refresh_nns_proposal_cache_with_progress},
        nns::registry::{
            NnsRegistryHostError, NnsRegistrySource, NnsRegistryVersionData,
            NnsRegistryVersionRequest, build_nns_registry_version_report_with_source,
            nns_registry_version_report_text,
        },
        nns::topology::{
            CachedNnsSubnetTopologyReport, DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT,
            NnsSubnetTopologyCacheRequest, NnsSubnetTopologyHostError,
            NnsSubnetTopologyRefreshRequest, refresh_nns_subnet_topology,
        },
        nns::{NnsGovernanceRefreshReport, NnsGovernanceRefreshRequest, NnsInventoryCacheRequest},
        sns::{
            DEFAULT_SNS_SOURCE_ENDPOINT, SnsCacheStatusRequest, SnsHostError, SnsMetricsRequest,
            SnsNeuronsRequest, SnsNeuronsSort, SnsProposalsReport, SnsProposalsRequest,
            SnsProposalsSort, SnsSnapshotRefreshReport, SnsSnapshotRefreshRequest,
            build_sns_metrics_report, build_sns_neurons_cache_status_report,
            build_sns_neurons_report, build_sns_proposals_cache_status_report,
            build_sns_proposals_report, build_sns_proposals_report_with_progress,
            refresh_sns_neurons_cache_with_progress, refresh_sns_proposals_cache_with_progress,
            sns_metrics_report_text, sns_neurons_cache_status_report_text, sns_neurons_report_text,
            sns_proposals_cache_status_report_text, sns_proposals_report_text,
        },
        subnet_catalog::{
            DEFAULT_STALE_AFTER_SECONDS, DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, ResolveAs,
            SubnetCatalogCacheRequest, SubnetCatalogHostError, SubnetCatalogInfoRequest,
            SubnetKind, build_subnet_catalog_info_report, subnet_catalog_info_report_text,
        },
    };
    use std::path::Path;

    #[tokio::test(flavor = "current_thread")]
    async fn downstream_host_examples_typecheck_public_builders() {
        accepts_subnet_example(render_subnet_info);
        accepts_nns_node_example(render_application_nodes);
        accepts_icrc_example(render_token);
        accepts_custom_source_example(render_registry_version_with_source);
        accepts_subnet_topology_example(refresh_subnet_topology);
        accepts_sns_metrics_example(render_sns_metrics);
        accepts_sns_proposals_example(render_recent_sns_proposals);
        accepts_sns_neurons_example(render_cached_sns_neurons);
        accepts_sns_cache_status_example(render_sns_cache_status);
        assert_progress_api_is_public();

        let text = render_registry_version_with_source(&FixtureRegistrySource, 1_700_000_000)
            .await
            .expect("custom registry source");
        assert!(text.contains("registry_version: 42"));
    }

    async fn render_subnet_info(
        cache_root: &Path,
        canister_or_subnet: &str,
        now_unix_secs: u64,
    ) -> Result<String, SubnetCatalogHostError> {
        let cache = SubnetCatalogCacheRequest::new(cache_root, "ic");
        let request = SubnetCatalogInfoRequest::new(
            cache,
            DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            canister_or_subnet,
            now_unix_secs,
            DEFAULT_STALE_AFTER_SECONDS,
        )
        .with_forced(ResolveAs::Canister);

        let report = build_subnet_catalog_info_report(&request).await?;
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

        let _: for<'a> fn(
            &'a NnsGovernanceRefreshRequest,
            &'a mut dyn QueryProgress,
        ) -> ic_query::report::SourceFuture<
            'a,
            Result<NnsGovernanceRefreshReport, NnsProposalHostError>,
        > = |arg0, arg1| Box::pin(refresh_nns_proposal_cache_with_progress(arg0, arg1));
        let _: for<'a> fn(
            &'a SnsSnapshotRefreshRequest,
            &'a mut dyn QueryProgress,
        ) -> ic_query::report::SourceFuture<
            'a,
            Result<SnsSnapshotRefreshReport, SnsHostError>,
        > = |arg0, arg1| Box::pin(refresh_sns_neurons_cache_with_progress(arg0, arg1));
        let _: for<'a> fn(
            &'a SnsSnapshotRefreshRequest,
            &'a mut dyn QueryProgress,
        ) -> ic_query::report::SourceFuture<
            'a,
            Result<SnsSnapshotRefreshReport, SnsHostError>,
        > = |arg0, arg1| Box::pin(refresh_sns_proposals_cache_with_progress(arg0, arg1));
        let _: for<'a> fn(
            &'a SnsProposalsRequest,
            &'a mut dyn QueryProgress,
        ) -> ic_query::report::SourceFuture<
            'a,
            Result<SnsProposalsReport, SnsHostError>,
        > = |arg0, arg1| Box::pin(build_sns_proposals_report_with_progress(arg0, arg1));
    }

    async fn render_application_nodes(
        cache_root: &Path,
        now_unix_secs: u64,
    ) -> Result<String, NnsNodeHostError> {
        let cache = NnsInventoryCacheRequest::new(cache_root, "ic");
        let request =
            NnsNodeListRequest::new(cache, DEFAULT_NNS_NODE_SOURCE_ENDPOINT, now_unix_secs)
                .with_subnet_kind(SubnetKind::Application);

        let report = build_nns_node_list_report(&request).await?;
        Ok(nns_node_list_report_text(&report))
    }

    async fn render_token(
        ledger_canister_id: &str,
        now_unix_secs: u64,
    ) -> Result<String, IcrcError> {
        let request = IcrcLedgerRequest::new(
            DEFAULT_ICRC_SOURCE_ENDPOINT,
            now_unix_secs,
            ledger_canister_id,
        );
        let report = build_icrc_token_report(&request).await?;
        Ok(icrc_token_report_text(&report))
    }

    async fn render_registry_version_with_source(
        source: &dyn NnsRegistrySource,
        now_unix_secs: u64,
    ) -> Result<String, NnsRegistryHostError> {
        let request = NnsRegistryVersionRequest::new("ic", "https://mirror.example", now_unix_secs);
        let report = build_nns_registry_version_report_with_source(&request, source).await?;
        Ok(nns_registry_version_report_text(&report))
    }

    async fn render_recent_sns_proposals(
        cache_root: &Path,
        sns_input: &str,
        now_unix_secs: u64,
    ) -> Result<String, SnsHostError> {
        let request = SnsProposalsRequest::new(
            ic_query::sns::SnsLookupRequest::new(
                "ic",
                DEFAULT_SNS_SOURCE_ENDPOINT,
                now_unix_secs,
                sns_input,
            ),
            25,
        )
        .with_cache_root(cache_root)
        .with_sort(SnsProposalsSort::Created)
        .with_sort_direction(SortDirection::Desc);

        let report = build_sns_proposals_report(&request).await?;
        Ok(sns_proposals_report_text(&report))
    }

    async fn render_sns_metrics(
        sns_input: &str,
        now_unix_secs: u64,
    ) -> Result<String, SnsHostError> {
        let request = SnsMetricsRequest::new(ic_query::sns::SnsLookupRequest::new(
            "ic",
            DEFAULT_SNS_SOURCE_ENDPOINT,
            now_unix_secs,
            sns_input,
        ))
        .with_time_window_seconds(90 * 24 * 60 * 60);

        let report = build_sns_metrics_report(&request).await?;
        Ok(sns_metrics_report_text(&report))
    }

    async fn refresh_subnet_topology(
        cache_root: &Path,
        now_unix_secs: u64,
    ) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
        let request = NnsSubnetTopologyRefreshRequest::new(
            NnsSubnetTopologyCacheRequest::new(cache_root, "ic"),
            DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT,
            now_unix_secs,
        );
        refresh_nns_subnet_topology(&request).await
    }

    async fn render_cached_sns_neurons(
        cache_root: &Path,
        sns_input: &str,
        now_unix_secs: u64,
    ) -> Result<String, SnsHostError> {
        let request = SnsNeuronsRequest::new(
            ic_query::sns::SnsLookupRequest::new(
                "ic",
                DEFAULT_SNS_SOURCE_ENDPOINT,
                now_unix_secs,
                sns_input,
            ),
            500,
        )
        .with_cache_root(cache_root)
        .with_sort(SnsNeuronsSort::Stake);

        let report = build_sns_neurons_report(&request).await?;
        Ok(sns_neurons_report_text(&report))
    }

    fn render_sns_cache_status(cache_root: &Path, sns_input: &str) -> Result<String, SnsHostError> {
        let proposals = SnsCacheStatusRequest::new(cache_root, "ic", sns_input);
        let proposals_report = build_sns_proposals_cache_status_report(&proposals)?;

        let neurons = SnsCacheStatusRequest::new(cache_root, "ic", sns_input);
        let neurons_report = build_sns_neurons_cache_status_report(&neurons)?;

        Ok(format!(
            "{}\n{}",
            sns_proposals_cache_status_report_text(&proposals_report),
            sns_neurons_cache_status_report_text(&neurons_report)
        ))
    }

    struct FixtureRegistrySource;

    impl NnsRegistrySource for FixtureRegistrySource {
        fn fetch_registry_version<'a>(
            &'a self,
            request: &'a CollectionSourceRequest,
        ) -> ic_query::report::SourceFuture<'a, Result<NnsRegistryVersionData, NnsRegistryHostError>>
        {
            Box::pin(async move {
                Ok(NnsRegistryVersionData {
                    network: "ic".to_string(),
                    registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
                    registry_version: 42,
                    fetched_at: request.fetched_at.clone(),
                    fetched_by: request.fetched_by.clone(),
                    source_endpoint: request.endpoint.clone(),
                    certification: super::registry_certification(),
                })
            })
        }
    }

    fn accepts_subnet_example(
        _example: impl AsyncFn(&Path, &str, u64) -> Result<String, SubnetCatalogHostError>,
    ) {
    }

    fn accepts_nns_node_example(
        _example: impl AsyncFn(&Path, u64) -> Result<String, NnsNodeHostError>,
    ) {
    }

    fn accepts_icrc_example(_example: impl AsyncFn(&str, u64) -> Result<String, IcrcError>) {}

    fn accepts_custom_source_example(
        _example: impl AsyncFn(&dyn NnsRegistrySource, u64) -> Result<String, NnsRegistryHostError>,
    ) {
    }

    fn accepts_sns_proposals_example(
        _example: impl AsyncFn(&Path, &str, u64) -> Result<String, SnsHostError>,
    ) {
    }

    fn accepts_sns_metrics_example(
        _example: impl AsyncFn(&str, u64) -> Result<String, SnsHostError>,
    ) {
    }

    fn accepts_subnet_topology_example(
        _example: impl AsyncFn(
            &Path,
            u64,
        )
            -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError>,
    ) {
    }

    fn accepts_sns_neurons_example(
        _example: impl AsyncFn(&Path, &str, u64) -> Result<String, SnsHostError>,
    ) {
    }

    fn accepts_sns_cache_status_example(_example: fn(&Path, &str) -> Result<String, SnsHostError>) {
    }
}

#[cfg(feature = "certificate-host")]
#[test]
fn downstream_certificate_failure_exposes_shared_parse_source() {
    let agent = ic_platform::agent::Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let error = authenticate(
        &agent,
        ic_platform::candid::contract::Principal::anonymous(),
        &[0xff],
        1,
    )
    .unwrap_err();
    let CertificateVerificationError::Decode(source) = error else {
        panic!("expected a CBOR parse failure before authentication");
    };
    let source: ParseError = source;
    assert!(std::error::Error::source(&source).is_some());
}

#[cfg(feature = "ledger-evidence-host")]
#[test]
fn downstream_authenticated_ledger_tip_uses_host_type_and_limits() {
    let agent = ic_platform::agent::Agent::builder()
        .with_url("https://icp-api.io")
        .build()
        .unwrap();
    let result: Result<LedgerTip, CertificateVerificationError> = authenticate_ledger_tip(
        &agent,
        ic_platform::candid::contract::Principal::anonymous(),
        &[0xff],
        &[],
        CertificateEvidenceLimits { bytes: 0 },
    );
    assert!(matches!(
        result,
        Err(CertificateVerificationError::BudgetExceeded)
    ));
}
