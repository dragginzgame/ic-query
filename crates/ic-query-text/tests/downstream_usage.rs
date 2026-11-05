#[cfg(feature = "ledger-evidence-host")]
use ic_host_tools::ledger_evidence::LedgerTip;
#[cfg(feature = "certificate-host")]
use ic_platform::certification::cbor::strict::ParseError;
#[cfg(feature = "ledger-evidence-host")]
use ic_query::certificate::CertificateEvidenceLimits;
#[cfg(feature = "certificate-host")]
use ic_query::certificate::CertificateVerificationError;
#[cfg(feature = "certificate-host")]
use ic_query::certificate::authenticate;
#[cfg(feature = "ledger-evidence-host")]
use ic_query::certificate::authenticate_ledger_tip;
use ic_query::nns::registry::NnsRegistryCertification;
use ic_query::nns::registry::NnsRegistryVersionReport;
use ic_query::nns::registry::NnsRegistryVersionRequest;
use ic_query_text::nns::registry::nns_registry_version_report_text;

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
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            network: request.network,
            registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
            registry_version: 42,
            fetched_at: "2023-11-14T22:13:20Z".to_string(),
            source_endpoint: request.source_endpoint,
            fetched_by: "my-tool".to_string(),
        },
        certification: registry_certification(),
    };

    nns_registry_version_report_text(&report)
}

fn registry_certification() -> NnsRegistryCertification {
    NnsRegistryCertification {
        certificate_verified: true,
        certificate_time_nanos: 1_700_000_000_000_000_000,

        root_key_digest: "ab".repeat(32),
        certificate_hex: "cd".repeat(8),

        hash_tree_hex: "ef".repeat(4),
    }
}

#[cfg(feature = "host")]
mod host {
    use ic_query::QueryProgress;
    use ic_query::QueryProgressEvent;
    use ic_query::QueryProgressState;
    use ic_query::icrc::DEFAULT_ICRC_SOURCE_ENDPOINT;
    use ic_query::icrc::IcrcError;
    use ic_query::icrc::IcrcLedgerRequest;
    use ic_query::icrc::build_icrc_token_report;
    use ic_query::nns::NnsInventoryCacheRequest;
    use ic_query::nns::NnsInventoryError;
    use ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;
    use ic_query::nns::node::NnsNodeListRequest;
    use ic_query::nns::node::build_nns_node_list_report;
    use ic_query::nns::registry::NnsRegistryHostError;
    use ic_query::nns::registry::NnsRegistrySource;
    use ic_query::nns::registry::NnsRegistryVersionRequest;
    use ic_query::nns::registry::build_nns_registry_version_report_with_source;
    use ic_query::nns::topology::CachedNnsSubnetTopologyReport;
    use ic_query::nns::topology::DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT;
    use ic_query::nns::topology::NnsSubnetTopologyCacheRequest;
    use ic_query::nns::topology::NnsSubnetTopologyHostError;
    use ic_query::nns::topology::NnsSubnetTopologyRefreshRequest;
    use ic_query::nns::topology::refresh_nns_subnet_topology;
    use ic_query::report::CollectionSourceRequest;
    use ic_query::subnet_catalog::DEFAULT_STALE_AFTER_SECONDS;
    use ic_query::subnet_catalog::DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT;
    use ic_query::subnet_catalog::ResolveAs;
    use ic_query::subnet_catalog::SubnetCatalogCacheRequest;
    use ic_query::subnet_catalog::SubnetCatalogHostError;
    use ic_query::subnet_catalog::SubnetCatalogInfoRequest;
    use ic_query::subnet_catalog::SubnetKind;
    use ic_query::subnet_catalog::build_subnet_catalog_info_report;
    use ic_query_text::icrc::icrc_token_report_text;
    use ic_query_text::nns::node::nns_node_list_report_text;
    use ic_query_text::nns::registry::nns_registry_version_report_text;
    use ic_query_text::subnet_catalog::subnet_catalog_info_report_text;
    use std::path::Path;

    #[tokio::test(flavor = "current_thread")]
    async fn downstream_host_examples_typecheck_public_builders() {
        accepts_subnet_example(render_subnet_info);
        accepts_nns_node_example(render_application_nodes);
        accepts_icrc_example(render_token);
        accepts_custom_source_example(render_registry_version_with_source);
        accepts_subnet_topology_example(refresh_subnet_topology);
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
        let mut request = SubnetCatalogInfoRequest::new(
            cache,
            DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            canister_or_subnet,
            now_unix_secs,
            DEFAULT_STALE_AFTER_SECONDS,
        );
        request.forced = Some(ResolveAs::Canister);

        let report = build_subnet_catalog_info_report(&request).await?;
        Ok(subnet_catalog_info_report_text(&report))
    }

    fn assert_progress_api_is_public() {
        let mut events = Vec::new();
        let mut progress = |event| events.push(event);
        progress.report(QueryProgressEvent::PagedRefresh {
            progress: ic_query::PagedQueryProgress {
                collection: ic_query::QueryProgressCollection::NnsProposals,
                target: None,
                pages_fetched: 0,
                rows_fetched: 0,
            },
            state: QueryProgressState::Running,
        });
        assert_eq!(events.len(), 1);
    }

    async fn render_application_nodes(
        cache_root: &Path,
        now_unix_secs: u64,
    ) -> Result<String, NnsInventoryError> {
        let cache = NnsInventoryCacheRequest::new(cache_root, "ic");
        let request =
            NnsNodeListRequest::new(cache, DEFAULT_NNS_NODE_SOURCE_ENDPOINT, now_unix_secs)
                .with_subnet_kind(SubnetKind::Application);

        let report = build_nns_node_list_report(&request).await?;
        Ok(nns_node_list_report_text(
            &report,
            ic_query_text::TextOptions::default(),
        ))
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

    struct FixtureRegistrySource;

    impl NnsRegistrySource for FixtureRegistrySource {
        fn fetch_registry_version<'a>(
            &'a self,
            request: &'a CollectionSourceRequest,
        ) -> ic_query::report::SourceFuture<
            'a,
            Result<ic_query::nns::registry::NnsRegistryVersionData, NnsRegistryHostError>,
        > {
            Box::pin(async move {
                Ok(ic_query::nns::registry::NnsRegistryVersionData {
                    provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
                        network: "ic".to_string(),
                        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
                        registry_version: 42,
                        fetched_at: request.fetched_at.clone(),
                        source_endpoint: request.endpoint.clone(),
                        fetched_by: request.fetched_by.clone(),
                    },
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
        _example: impl AsyncFn(&Path, u64) -> Result<String, NnsInventoryError>,
    ) {
    }

    fn accepts_icrc_example(_example: impl AsyncFn(&str, u64) -> Result<String, IcrcError>) {}

    fn accepts_custom_source_example(
        _example: impl AsyncFn(&dyn NnsRegistrySource, u64) -> Result<String, NnsRegistryHostError>,
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
