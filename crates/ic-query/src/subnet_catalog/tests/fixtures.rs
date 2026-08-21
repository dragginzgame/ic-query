use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) const SUBNET_A: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
pub(super) const SUBNET_B: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
pub(super) const CANISTER_A: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

pub(super) fn list_request(root: &Path) -> SubnetCatalogListRequest {
    SubnetCatalogListRequest {
        cache: cache_request(root),
        read_policy: CatalogReadPolicy::RefreshMissingOrInvalid {
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
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
            source: CatalogSourceSelection::uncertified_query(
                DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
            ),
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
        source: CatalogSourceSelection::uncertified_query(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT),
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

///
/// AgreementFixtureMode
///
/// Controlled endpoint result used by agreement refresh fixtures.
///

#[derive(Clone, Copy)]
pub(super) enum AgreementFixtureMode {
    Matching,
    VersionMismatch,
    PayloadMismatch,
    EndpointFailure,
}

///
/// AgreementFixtureSource
///
/// Endpoint-aware async source used to verify bounded agreement collection.
///

pub(super) struct AgreementFixtureSource {
    mode: AgreementFixtureMode,
    differing_endpoint: String,
    calls: AtomicUsize,
}

impl AgreementFixtureSource {
    pub(super) fn new(mode: AgreementFixtureMode, differing_endpoint: &str) -> Self {
        Self {
            mode,
            differing_endpoint: differing_endpoint.to_string(),
            calls: AtomicUsize::new(0),
        }
    }

    pub(super) fn call_count(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

impl SubnetCatalogSource for AgreementFixtureSource {
    fn fetch_catalog<'a>(&'a self, request: &'a NnsSourceRequest) -> SubnetCatalogSourceFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::Relaxed);
            if request.endpoint == self.differing_endpoint
                && matches!(self.mode, AgreementFixtureMode::EndpointFailure)
            {
                return Err(SubnetCatalogHostError::Catalog(CatalogError::EmptySubnets));
            }
            let mut catalog = fixture_catalog();
            catalog.provenance.source_endpoints = vec![request.endpoint.clone()];
            catalog
                .provenance
                .fetched_at
                .clone_from(&request.fetched_at);
            catalog
                .provenance
                .fetched_by
                .clone_from(&request.fetched_by);
            if request.endpoint == self.differing_endpoint {
                match self.mode {
                    AgreementFixtureMode::VersionMismatch => {
                        catalog.provenance.registry_version += 1;
                    }
                    AgreementFixtureMode::PayloadMismatch => {
                        catalog.subnets[0].node_count = Some(35);
                    }
                    AgreementFixtureMode::Matching | AgreementFixtureMode::EndpointFailure => {}
                }
            }
            attach_complete_registry_evidence(&mut catalog, &request.endpoint);
            catalog.canonicalize_and_seal()?;
            Ok(catalog)
        })
    }
}

fn attach_complete_registry_evidence(catalog: &mut RawSubnetCatalog, endpoint: &str) {
    let registry_version = catalog.provenance.registry_version;
    let evidence = |record| SubnetCatalogRegistryRecordEvidence {
        record,
        requested_registry_version: registry_version,
        returned_registry_version: registry_version.saturating_sub(1),
        timestamp_nanoseconds: 1_780_531_200_000_000_000,
        source_endpoint: endpoint.to_string(),
        assurance: CatalogAssurance::UncertifiedQuery,
        value_encoding: SubnetCatalogRegistryValueEncoding::Inline,
    };
    let mut registry_records = vec![
        evidence(SubnetCatalogRegistryRecordSubject::keyed(
            SubnetCatalogRegistryRecordKind::SubnetList,
            crate::ic_registry::SUBNET_LIST_KEY,
        )),
        evidence(SubnetCatalogRegistryRecordSubject::keyed(
            SubnetCatalogRegistryRecordKind::RoutingTable,
            crate::ic_registry::ROUTING_TABLE_KEY,
        )),
    ];
    registry_records.extend(catalog.subnets.iter().map(|subnet| {
        let principal = candid::Principal::from_text(&subnet.subnet_principal)
            .expect("fixture Subnet principal");
        evidence(SubnetCatalogRegistryRecordSubject::subnet_record(
            crate::ic_registry::subnet_record_key(&subnet.subnet_principal),
            principal,
        ))
    }));
    catalog.provenance.registry_records = registry_records;
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
    fn fetch_catalog<'a>(&'a self, request: &'a NnsSourceRequest) -> SubnetCatalogSourceFuture<'a> {
        Box::pin(async move {
            if self.fail {
                return Err(SubnetCatalogHostError::Catalog(CatalogError::EmptySubnets));
            }
            let mut catalog = self.catalog.clone().expect("fixture catalog");
            catalog.provenance.source_endpoints = vec![request.endpoint.clone()];
            catalog.canonicalize_and_seal()?;
            Ok(catalog)
        })
    }
}

///
/// DetailedFailureSource
///
/// Fixture source that returns caller-selected typed collection failure provenance.
///

pub(super) struct DetailedFailureSource {
    registry_version: Option<u64>,
    returned_registry_value_version: Option<u64>,
    source_endpoint: Option<String>,
    assurance: Option<CatalogAssurance>,
    subject: Option<SubnetCatalogSubject>,
    message: &'static str,
}

impl DetailedFailureSource {
    pub(super) const fn new(
        registry_version: Option<u64>,
        subject: Option<SubnetCatalogSubject>,
        message: &'static str,
    ) -> Self {
        Self {
            registry_version,
            returned_registry_value_version: None,
            source_endpoint: None,
            assurance: None,
            subject,
            message,
        }
    }

    pub(super) fn with_value_response(
        mut self,
        returned_registry_value_version: u64,
        source_endpoint: &str,
    ) -> Self {
        self.returned_registry_value_version = Some(returned_registry_value_version);
        self.source_endpoint = Some(source_endpoint.to_string());
        self.assurance = Some(CatalogAssurance::UncertifiedQuery);
        self
    }

    fn source_error(&self) -> SubnetCatalogHostError {
        SubnetCatalogHostError::RegistryRefresh(
            crate::ic_registry::RegistryFetchError::ProtobufDecode {
                message: self.message,
                reason: "fixture failure".to_string(),
            },
        )
    }
}

impl SubnetCatalogSource for DetailedFailureSource {
    fn fetch_catalog<'a>(
        &'a self,
        _request: &'a NnsSourceRequest,
    ) -> SubnetCatalogSourceFuture<'a> {
        Box::pin(async move { Err(self.source_error()) })
    }

    fn fetch_catalog_detailed<'a>(
        &'a self,
        _request: &'a NnsSourceRequest,
    ) -> SubnetCatalogDetailedSourceFuture<'a> {
        Box::pin(async move {
            Err(SubnetCatalogSourceFailure::new(
                self.registry_version,
                self.subject.clone(),
                self.source_error(),
            )
            .with_registry_evidence(
                self.returned_registry_value_version,
                self.source_endpoint.clone(),
                self.assurance,
                Vec::new(),
            ))
        })
    }
}

pub(super) fn fixture_catalog() -> RawSubnetCatalog {
    RawSubnetCatalog::new_mainnet_uncertified(
        UncertifiedCatalogCollection::new(
            123_456,
            "https://icp-api.io",
            "2026-06-04T00:00:00Z",
            "fixture",
            "test",
            5,
        ),
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
