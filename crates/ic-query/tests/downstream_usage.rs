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
        icrc::{
            DEFAULT_ICRC_SOURCE_ENDPOINT, IcrcError, IcrcTokenRequest, build_icrc_token_report,
            icrc_token_report_text,
        },
        nns::node::{
            DEFAULT_NNS_NODE_SOURCE_ENDPOINT, NNS_NODE_SUBNET_KIND_APPLICATION,
            NnsNodeCacheRequest, NnsNodeHostError, NnsNodeListRequest, build_nns_node_list_report,
            nns_node_list_report_text,
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

    fn accepts_subnet_example(
        _example: fn(&Path, &str, u64) -> Result<String, SubnetCatalogHostError>,
    ) {
    }

    fn accepts_nns_node_example(_example: fn(&Path, u64) -> Result<String, NnsNodeHostError>) {}

    fn accepts_icrc_example(_example: fn(&str, u64) -> Result<String, IcrcError>) {}
}
