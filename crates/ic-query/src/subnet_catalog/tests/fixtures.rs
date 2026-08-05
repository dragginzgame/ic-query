use super::*;

pub(super) const SUBNET_A: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
pub(super) const SUBNET_B: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
pub(super) const CANISTER_A: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

pub(super) fn list_request(root: &Path) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest {
        cache: cache_request(root),
        read_policy: CatalogReadPolicy::RefreshMissingOrInvalid {
            source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        },
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
        read_policy: CatalogReadPolicy::RefreshMissingOrInvalid {
            source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        },
        input: input.to_string(),
        forced: None,
        now_unix_secs: 1_780_531_300,
        stale_after_seconds: DEFAULT_STALE_AFTER_SECONDS,
    }
}

pub(super) fn cache_request(root: &Path) -> SubnetCatalogCacheRequest {
    SubnetCatalogCacheRequest {
        cache_root: root.to_path_buf(),
        network: MAINNET_NETWORK.to_string(),
    }
}

pub(super) fn cache_only_load_request(root: &Path) -> SubnetCatalogLoadRequest {
    SubnetCatalogLoadRequest::cache_only(cache_request(root), 1_780_531_300)
}

pub(super) fn write_catalog(root: &Path, catalog: RawSubnetCatalog) {
    let path = subnet_catalog_path(root, MAINNET_NETWORK);
    crate::cache_file::write_managed_text_atomically(
        root,
        &path,
        &serde_json::to_string_pretty(&catalog).expect("serialize catalog"),
    )
    .expect("write catalog");
}

pub(super) fn refresh_request(root: &Path) -> SubnetCatalogRefreshRequest {
    SubnetCatalogRefreshRequest {
        cache: cache_request(root),
        source_endpoint: DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        lock_stale_after_seconds: DEFAULT_REFRESH_LOCK_STALE_SECONDS,
        max_future_skew_seconds: DEFAULT_CATALOG_MAX_FUTURE_SKEW_SECONDS,
        dry_run: false,
        output_path: None,
    }
}

pub(super) fn write_refresh_lock_for_test(
    lock_path: &Path,
    request: &SubnetCatalogRefreshRequest,
    started_at_unix_ms: u64,
) {
    let lock = serde_json::json!({
        "schema_version": 1,
        "network": request.cache.network.clone(),
        "pid": 12345,
        "started_at_unix_ms": started_at_unix_ms,
        "stale_after_seconds": request.lock_stale_after_seconds,
        "target_path": subnet_catalog_path(&request.cache.cache_root, &request.cache.network)
            .display()
            .to_string(),
    });
    crate::cache_file::write_managed_text_atomically(
        &request.cache.cache_root,
        lock_path,
        &serde_json::to_string_pretty(&lock).expect("serialize lock"),
    )
    .expect("write lock");
}

///
/// FixtureRefreshSource
///
/// Controllable subnet catalog refresh source used by host tests.
///

pub(super) struct FixtureRefreshSource {
    catalog: Option<RawSubnetCatalog>,
    fail: bool,
}

impl FixtureRefreshSource {
    pub(super) const fn ok(catalog: RawSubnetCatalog) -> Self {
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
        _request: &NnsSourceRequest,
    ) -> Result<RawSubnetCatalog, SubnetCatalogHostError> {
        if self.fail {
            return Err(SubnetCatalogHostError::Catalog(CatalogError::EmptySubnets));
        }
        Ok(self.catalog.clone().expect("fixture catalog"))
    }
}

pub(super) fn fixture_catalog() -> RawSubnetCatalog {
    RawSubnetCatalog::new_mainnet_uncertified(
        123_456,
        "https://icp-api.io",
        "2026-06-04T00:00:00Z",
        "fixture",
        "test",
        vec![
            SubnetInfo {
                subnet_principal: SUBNET_A.to_string(),
                registry_subnet_type: 1,
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
                registry_subnet_type: 2,
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
        vec![
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
    )
    .expect("valid fixture catalog")
}
