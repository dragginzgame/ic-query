use super::*;

pub(super) const SUBNET_A: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
pub(super) const SUBNET_B: &str = "aaaaa-aa";
pub(super) const CANISTER_A: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

pub(super) fn list_request(root: &Path) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest {
        cache: cache_request(root),
        source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_300,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
        filters: SubnetCatalogFilters::default(),
        show_ranges: true,
        range_limit: 1,
        range_offset: 0,
    }
}

pub(super) fn info_request(root: &Path, input: &str) -> SubnetCatalogInfoRequest {
    SubnetCatalogInfoRequest {
        cache: cache_request(root),
        source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        input: input.to_string(),
        forced: None,
        now_unix_secs: 1_780_531_300,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
    }
}

pub(super) fn cache_request(root: &Path) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest {
        icp_root: root.to_path_buf(),
        network: MAINNET_NETWORK.to_string(),
    }
}

pub(super) fn write_catalog(root: &Path, catalog: SubnetCatalog) {
    let path = subnet_catalog_path(root, MAINNET_NETWORK);
    fs::create_dir_all(path.parent().expect("catalog parent")).expect("create parent");
    fs::write(
        path,
        serde_json::to_vec_pretty(&catalog).expect("serialize catalog"),
    )
    .expect("write catalog");
}

pub(super) fn refresh_request(root: &Path) -> SubnetCatalogRefreshRequest {
    SubnetCatalogRefreshRequest {
        cache: cache_request(root),
        source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        lock_stale_after_seconds: DEFAULT_REFRESH_LOCK_STALE_SECONDS,
        dry_run: false,
        output_path: None,
    }
}

pub(super) fn write_refresh_lock_for_test(
    lock_path: &Path,
    request: &SubnetCatalogRefreshRequest,
    started_at_unix_ms: u64,
) {
    fs::create_dir_all(lock_path.parent().expect("lock parent")).expect("create parent");
    let lock = serde_json::json!({
        "schema_version": 1,
        "network": request.cache.network.clone(),
        "pid": 12345,
        "started_at_unix_ms": started_at_unix_ms,
        "target_path": subnet_catalog_path(&request.cache.icp_root, &request.cache.network)
            .display()
            .to_string(),
    });
    fs::write(
        lock_path,
        serde_json::to_vec_pretty(&lock).expect("serialize lock"),
    )
    .expect("write lock");
}

///
/// FixtureRefreshSource
///
pub(super) struct FixtureRefreshSource {
    catalog: Option<SubnetCatalog>,
    fail: bool,
}

impl FixtureRefreshSource {
    pub(super) const fn ok(catalog: SubnetCatalog) -> Self {
        Self {
            catalog: Some(catalog),
            fail: false,
        }
    }

    pub(super) const fn err() -> Self {
        Self {
            catalog: None,
            fail: true,
        }
    }
}

impl SubnetCatalogSource for FixtureRefreshSource {
    fn fetch_catalog(
        &self,
        _request: &SubnetCatalogSourceRequest,
    ) -> Result<SubnetCatalog, SubnetCatalogHostError> {
        if self.fail {
            return Err(SubnetCatalogHostError::InvalidStaleDuration {
                value: "fixture".to_string(),
            });
        }
        Ok(self.catalog.clone().expect("fixture catalog"))
    }
}

pub(super) fn fixture_catalog() -> SubnetCatalog {
    SubnetCatalog {
        catalog_schema_version: CATALOG_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 123_456,
        fetched_at: "2026-06-04T00:00:00Z".to_string(),
        fetched_by: "fixture".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        subnets: vec![
            SubnetInfo {
                subnet_principal: SUBNET_A.to_string(),
                subnet_kind: SubnetKind::Application,
                subnet_kind_source: ClassificationSource::Registry,
                subnet_specialization: SubnetSpecialization::Fiduciary,
                subnet_specialization_source: ClassificationSource::Curated,
                geographic_scope: GeographicScope::Global,
                geographic_scope_source: ClassificationSource::Curated,
                subnet_label: "fiduciary".to_string(),
                subnet_label_source: ClassificationSource::Curated,
                node_count: Some(34),
                charges_apply_by_default: true,
            },
            SubnetInfo {
                subnet_principal: SUBNET_B.to_string(),
                subnet_kind: SubnetKind::System,
                subnet_kind_source: ClassificationSource::Registry,
                subnet_specialization: SubnetSpecialization::None,
                subnet_specialization_source: ClassificationSource::Curated,
                geographic_scope: GeographicScope::Global,
                geographic_scope_source: ClassificationSource::Curated,
                subnet_label: "system".to_string(),
                subnet_label_source: ClassificationSource::Curated,
                node_count: Some(13),
                charges_apply_by_default: false,
            },
        ],
        routing_ranges: vec![
            RoutingRange {
                start_canister_id: CANISTER_A.to_string(),
                end_canister_id: CANISTER_A.to_string(),
                subnet_principal: SUBNET_A.to_string(),
            },
            RoutingRange {
                start_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
                end_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
                subnet_principal: SUBNET_A.to_string(),
            },
            RoutingRange {
                start_canister_id: "r7inp-6aaaa-aaaaa-aaabq-cai".to_string(),
                end_canister_id: "r7inp-6aaaa-aaaaa-aaabq-cai".to_string(),
                subnet_principal: SUBNET_B.to_string(),
            },
        ],
    }
}
