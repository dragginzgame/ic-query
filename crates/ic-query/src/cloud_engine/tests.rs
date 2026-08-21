use super::*;
use crate::subnet_catalog::{
    CacheDisposition, CatalogAssurance, ClassificationSource, GeographicScope, MAINNET_NETWORK,
    MAINNET_REGISTRY_CANISTER_ID, SubnetCatalogListReport, SubnetCatalogSubnetRow, SubnetKind,
    SubnetSpecialization,
};
use std::cell::Cell;

const FETCHED_AT: &str = "2026-08-08T12:00:00Z";
const SUBNET_ID: &str = "2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe";
const OPERATOR_ID: &str = "wlnge-zyaaa-aaabw-aaaaa-cai";
const OWNER_ID: &str = "4vh3j-nyc2w-eaan4-vsl33-dguwj-7hlsb-bffh2-exinh-parof-qqlki-lae";
const ADMIN_ID: &str = "bct5z-vccu4-6q4t2-3lb6l-wm43p-ulppt-o5sqq-w6het-rthdz-qp4yn-fqe";
const SUBNET_B: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
const SUBNET_C: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";

struct FixtureSource {
    calls: Cell<usize>,
    operator: CloudEngineOperatorSourceData,
    prices: CloudEnginePricesSourceData,
}

impl FixtureSource {
    fn new(request: &CloudEngineSourceRequest) -> Self {
        Self {
            calls: Cell::new(0),
            operator: operator_source(request),
            prices: prices_source(request),
        }
    }
}

impl CloudEngineSource for FixtureSource {
    fn fetch_operator(
        &self,
        _request: &CloudEngineSourceRequest,
        _subnet_id: &str,
    ) -> Result<CloudEngineOperatorSourceData, CloudEngineHostError> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.operator.clone())
    }

    fn fetch_prices(
        &self,
        _request: &CloudEngineSourceRequest,
    ) -> Result<CloudEnginePricesSourceData, CloudEngineHostError> {
        self.calls.set(self.calls.get() + 1);
        Ok(self.prices.clone())
    }
}

struct BindingFixtureSource {
    calls: Cell<usize>,
    request: CloudEngineSourceRequest,
    query_call_count: usize,
}

impl CloudEngineOperatorBindingSource for BindingFixtureSource {
    fn fetch_operator_binding(
        &self,
        _request: &CloudEngineSourceRequest,
        subnet_id: &str,
    ) -> Result<CloudEngineOperatorBindingSourceData, CloudEngineHostError> {
        self.calls.set(self.calls.get() + 1);
        if subnet_id == SUBNET_C {
            return Err(CloudEngineHostError::AgentCall {
                method: "getEngineOperatorBySubnet",
                reason: "fixture unavailable".to_string(),
            });
        }
        Ok(CloudEngineOperatorBindingSourceData {
            source: self.request.clone(),
            subnet_id: subnet_id.to_string(),
            operator_canister_id: (subnet_id == SUBNET_ID).then(|| OPERATOR_ID.to_string()),
            query_call_count: self.query_call_count,
        })
    }
}

#[test]
fn operator_report_preserves_public_settings_and_provenance() {
    let request = request(MAINNET_NETWORK);
    let source = FixtureSource::new(&request);
    let report = build_cloud_engine_operator_report_with_source(&request, SUBNET_ID, &source)
        .expect("fixture CloudEngine operator report");

    assert_eq!(source.calls.get(), 1);
    assert_eq!(report.context.schema_version, 1);
    assert_eq!(
        report.context.authority,
        "cloud_engine_control_plane_canister"
    );
    assert_eq!(
        report.context.engine_canister_id,
        MAINNET_CLOUD_ENGINE_CANISTER_ID
    );
    assert!(!report.context.certified);
    assert!(!report.context.point_in_time_guaranteed);
    assert_eq!(report.context.query_call_count, 5);
    assert_eq!(report.subnet_id, SUBNET_ID);
    assert!(report.operator_binding_present);
    assert_eq!(report.operator_canister_id.as_deref(), Some(OPERATOR_ID));
    assert_eq!(report.engine_owner.as_deref(), Some(OWNER_ID));
    assert_eq!(report.platform_admin.as_deref(), Some(ADMIN_ID));
    assert_eq!(report.caffeine_enabled, Some(true));
    assert_eq!(report.claimed_domain_count, Some(2));
    assert_eq!(
        report.claimed_domains,
        Some(vec!["a.example".to_string(), "z.example".to_string()])
    );

    let text = cloud_engine_operator_report_text(&report);
    assert!(text.contains("operator_binding_present: yes"));
    assert!(text.contains("\n\nClaimed domains\n"));
    let json = serde_json::to_value(&report).expect("serialize operator report");
    assert_eq!(json["query_call_count"], 5);
    assert_eq!(json["operator_binding_present"], true);
    assert_eq!(json["claimed_domains"][0], "a.example");
}

#[test]
fn absent_operator_binding_is_a_visible_one_call_result() {
    let request = request(MAINNET_NETWORK);
    let mut source = FixtureSource::new(&request);
    source.operator = CloudEngineOperatorSourceData {
        source: request.clone(),
        subnet_id: SUBNET_ID.to_string(),
        operator_canister_id: None,
        engine_owner: None,
        platform_admin: None,
        caffeine_enabled: None,
        claimed_domains: None,
        query_call_count: 1,
    };
    let report = build_cloud_engine_operator_report_with_source(&request, SUBNET_ID, &source)
        .expect("absent operator binding is reportable");

    assert!(!report.operator_binding_present);
    assert_eq!(report.context.query_call_count, 1);
    assert_eq!(report.claimed_domain_count, None);
}

#[test]
fn price_report_sorts_rows_and_preserves_raw_cycle_text() {
    let request = request(MAINNET_NETWORK);
    let source = FixtureSource::new(&request);
    let report = build_cloud_engine_prices_report_with_source(&request, &source)
        .expect("fixture CloudEngine prices report");

    assert_eq!(source.calls.get(), 1);
    assert_eq!(report.context.query_call_count, 2);
    assert!((report.network_fee - 0.25).abs() < f64::EPSILON);
    assert_eq!(report.price_count, 3);
    assert_eq!(report.prices[0].key, "aaaaa-aa,type4.3");
    assert_eq!(report.prices[1].key, "type4.1");
    assert_eq!(report.prices[2].key, "type4.2,de-fr-001");
    assert_eq!(report.prices[1].net_cycles_per_month, "471065106452");

    let text = cloud_engine_prices_report_text(&report);
    assert!(text.contains("\n\nMarketplace prices\n"));
    assert!(text.contains("471.07 B"));
    let json = serde_json::to_value(&report).expect("serialize prices report");
    assert_eq!(json["prices"][1]["net_cycles_per_month"], "471065106452");
    assert_eq!(
        json["prices"][1]["updated_at_unix_nanos"],
        1_785_946_128_242_156_275_i64
    );
}

#[test]
fn list_report_keeps_registry_inventory_and_binding_outcomes_separate() {
    let request = request(MAINNET_NETWORK);
    let source = BindingFixtureSource {
        calls: Cell::new(0),
        request: request.clone(),
        query_call_count: 1,
    };
    let report = super::list::build_cloud_engine_list_report_from_catalog_with_source(
        &request,
        list_catalog_report(&[SUBNET_B, SUBNET_C, SUBNET_ID]),
        &source,
    )
    .expect("fixture CloudEngine list report");

    assert_eq!(source.calls.get(), 3);
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.registry_authority, "nns_registry");
    assert_eq!(report.registry_version, 123_456);
    assert_eq!(
        report.control_plane_authority,
        "cloud_engine_control_plane_canister"
    );
    assert!(!report.control_plane_certified);
    assert!(!report.control_plane_point_in_time_guaranteed);
    assert_eq!(report.control_plane_lookup_attempt_count, 3);
    assert_eq!(report.registry_cloud_engine_subnet_count, 3);
    assert_eq!(report.operator_binding_count, 1);
    assert_eq!(report.missing_operator_binding_count, 1);
    assert_eq!(report.operator_lookup_failure_count, 1);
    assert_eq!(
        report
            .cloud_engines
            .iter()
            .map(|row| row.subnet_id.as_str())
            .collect::<Vec<_>>(),
        vec![SUBNET_ID, SUBNET_C, SUBNET_B]
    );

    let resolved = report
        .cloud_engines
        .iter()
        .find(|row| row.subnet_id == SUBNET_ID)
        .expect("resolved row");
    assert_eq!(
        resolved.operator_lookup_status,
        CloudEngineOperatorLookupStatus::Resolved
    );
    assert_eq!(resolved.operator_canister_id.as_deref(), Some(OPERATOR_ID));

    let absent = report
        .cloud_engines
        .iter()
        .find(|row| row.subnet_id == SUBNET_B)
        .expect("absent row");
    assert_eq!(
        absent.operator_lookup_status,
        CloudEngineOperatorLookupStatus::Absent
    );
    assert_eq!(absent.operator_lookup_error, None);

    let failed = report
        .cloud_engines
        .iter()
        .find(|row| row.subnet_id == SUBNET_C)
        .expect("failed row");
    assert_eq!(
        failed.operator_lookup_status,
        CloudEngineOperatorLookupStatus::Failed
    );
    assert!(
        failed
            .operator_lookup_error
            .as_deref()
            .is_some_and(|error| error.contains("fixture unavailable"))
    );

    let text = cloud_engine_list_report_text(&report);
    assert!(text.contains("\n\ncontrol_plane_authority:"));
    assert!(text.contains("\n\nCloudEngine subnets\n"));
    assert!(text.contains("\n\nOperator lookup failures\n"));
    let json = serde_json::to_value(&report).expect("serialize list report");
    assert_eq!(json["registry_cloud_engine_subnet_count"], 3);
    assert_eq!(
        json["cloud_engines"][0]["operator_lookup_status"],
        "resolved"
    );
}

#[test]
fn list_report_enforces_fanout_and_successful_source_contracts() {
    let request = request(MAINNET_NETWORK);
    let source = BindingFixtureSource {
        calls: Cell::new(0),
        request: request.clone(),
        query_call_count: 1,
    };
    let excessive = vec![SUBNET_ID; MAX_CLOUD_ENGINE_LIST_ROWS + 1];
    let error = super::list::build_cloud_engine_list_report_from_catalog_with_source(
        &request,
        list_catalog_report(&excessive),
        &source,
    )
    .expect_err("fanout above the hard bound must fail before lookups");
    assert!(matches!(
        error,
        CloudEngineHostError::InvalidSourceData { reason }
            if reason.contains("maximum is 100")
    ));
    assert_eq!(source.calls.get(), 0);

    let source = BindingFixtureSource {
        calls: Cell::new(0),
        request: request.clone(),
        query_call_count: 2,
    };
    let error = super::list::build_cloud_engine_list_report_from_catalog_with_source(
        &request,
        list_catalog_report(&[SUBNET_ID]),
        &source,
    )
    .expect_err("a successful binding source must report exactly one call");
    assert!(matches!(
        error,
        CloudEngineHostError::InvalidSourceData { reason }
            if reason.contains("exactly one query call")
    ));
    assert_eq!(source.calls.get(), 1);
}

#[test]
fn builders_reject_targets_before_invoking_custom_sources() {
    let local_request = request("local");
    let source = FixtureSource::new(&local_request);
    let error = build_cloud_engine_prices_report_with_source(&local_request, &source)
        .expect_err("non-mainnet CloudEngine report must fail");
    assert!(matches!(
        error,
        CloudEngineHostError::UnsupportedNetwork { network } if network == "local"
    ));
    assert_eq!(source.calls.get(), 0);

    let request = request(MAINNET_NETWORK);
    let source = FixtureSource::new(&request);
    let error =
        build_cloud_engine_operator_report_with_source(&request, "not-a-principal", &source)
            .expect_err("invalid Subnet must fail before source invocation");
    assert!(matches!(
        error,
        CloudEngineHostError::InvalidPrincipal { .. }
    ));
    assert_eq!(source.calls.get(), 0);
}

#[test]
fn builders_reject_inconsistent_source_evidence() {
    let request = request(MAINNET_NETWORK);
    let mut source = FixtureSource::new(&request);
    source.operator.query_call_count = 4;
    let error = build_cloud_engine_operator_report_with_source(&request, SUBNET_ID, &source)
        .expect_err("present operator binding call count must be exact");
    assert!(matches!(
        error,
        CloudEngineHostError::InvalidSourceData { reason } if reason.contains("five query calls")
    ));

    let mut source = FixtureSource::new(&request);
    source.prices.prices[0].key = "wrong".to_string();
    let error = build_cloud_engine_prices_report_with_source(&request, &source)
        .expect_err("marketplace identity must match the key");
    assert!(matches!(
        error,
        CloudEngineHostError::InvalidSourceData { reason } if reason.contains("does not match")
    ));

    let mut source = FixtureSource::new(&request);
    source.prices.prices[0].net_cycles_per_month =
        "1".repeat(MAX_CLOUD_ENGINE_CYCLE_DECIMAL_DIGITS + 1);
    let error = build_cloud_engine_prices_report_with_source(&request, &source)
        .expect_err("cycle amounts beyond the public digit bound must fail");
    assert!(matches!(
        error,
        CloudEngineHostError::InvalidSourceData { reason }
            if reason.contains("canonical non-negative decimal")
    ));
}

#[test]
fn live_source_rejects_network_before_constructing_an_agent() {
    let request = CloudEngineSourceRequest::new("local", "://invalid", FETCHED_AT, "test");
    let error = LiveCloudEngineSource
        .fetch_prices(&request)
        .expect_err("network is validated before endpoint parsing");
    assert!(matches!(
        error,
        CloudEngineHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn source_request_formats_unix_collection_time() {
    let request = CloudEngineSourceRequest::from_unix_secs(
        MAINNET_NETWORK,
        DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT,
        0,
        "test",
    );
    assert_eq!(request.fetched_at, "1970-01-01T00:00:00Z");
}

fn request(network: &str) -> CloudEngineSourceRequest {
    CloudEngineSourceRequest::new(
        network,
        DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT,
        FETCHED_AT,
        "test",
    )
}

fn operator_source(request: &CloudEngineSourceRequest) -> CloudEngineOperatorSourceData {
    CloudEngineOperatorSourceData {
        source: request.clone(),
        subnet_id: SUBNET_ID.to_string(),
        operator_canister_id: Some(OPERATOR_ID.to_string()),
        engine_owner: Some(OWNER_ID.to_string()),
        platform_admin: Some(ADMIN_ID.to_string()),
        caffeine_enabled: Some(true),
        claimed_domains: Some(vec!["z.example".to_string(), "a.example".to_string()]),
        query_call_count: 5,
    }
}

fn prices_source(request: &CloudEngineSourceRequest) -> CloudEnginePricesSourceData {
    CloudEnginePricesSourceData {
        source: request.clone(),
        network_fee: 0.25,
        prices: vec![
            price_row(
                "type4.2,de-fr-001",
                CloudEngineNodeType::Type4_2,
                Some("de-fr-001"),
                None,
                "500000000000",
                "625000000000",
            ),
            price_row(
                "aaaaa-aa,type4.3",
                CloudEngineNodeType::Type4_3,
                None,
                Some("aaaaa-aa"),
                "600000000000",
                "750000000000",
            ),
            price_row(
                "type4.1",
                CloudEngineNodeType::Type4_1,
                None,
                None,
                "471065106452",
                "588831383065",
            ),
        ],
        query_call_count: 2,
    }
}

fn price_row(
    key: &str,
    node_type: CloudEngineNodeType,
    data_center_id: Option<&str>,
    provider_id: Option<&str>,
    net: &str,
    gross: &str,
) -> CloudEnginePriceRow {
    CloudEnginePriceRow {
        key: key.to_string(),
        node_type,
        data_center_id: data_center_id.map(str::to_string),
        provider_id: provider_id.map(str::to_string),
        net_cycles_per_month: net.to_string(),
        gross_cycles_per_month: gross.to_string(),
        updated_at_unix_nanos: 1_785_946_128_242_156_275,
    }
}

fn list_catalog_report(subnet_ids: &[&str]) -> SubnetCatalogListReport {
    SubnetCatalogListReport {
        schema_version: 1,
        network: MAINNET_NETWORK.to_string(),
        catalog_path: "/tmp/subnet-catalog.json".to_string(),
        catalog_schema_version: 1,
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version: 123_456,
        assurance: CatalogAssurance::UncertifiedQuery,
        source_endpoints: vec!["https://icp-api.io".to_string()],
        agreement_digest: None,
        registry_query_call_count: 5,
        routing_source: crate::subnet_catalog::SubnetCatalogRoutingSource::LegacyRoutingTable,
        registry_records: Vec::new(),
        catalog_digest: "0".repeat(64),
        cache_disposition: CacheDisposition::CacheHit,
        fetched_at: FETCHED_AT.to_string(),
        catalog_stale: false,
        stale_reason: "fresh".to_string(),
        resolver_backend: "registry_routing_table".to_string(),
        collector_version: "test".to_string(),
        classification_schema_version: 1,
        classification_policy_digest: "1".repeat(64),
        resolver_schema_version: 1,
        subnets: subnet_ids
            .iter()
            .enumerate()
            .map(|(index, subnet_id)| SubnetCatalogSubnetRow {
                subnet_principal: (*subnet_id).to_string(),
                registry_subnet_type: 5,
                subnet_kind: SubnetKind::CloudEngine,
                subnet_kind_source: ClassificationSource::Registry,
                subnet_specialization: SubnetSpecialization::None,
                subnet_specialization_source: ClassificationSource::Curated,
                geographic_scope: GeographicScope::Global,
                geographic_scope_source: ClassificationSource::Curated,
                subnet_label: format!("cloud-engine-{}", index + 1),
                subnet_label_source: ClassificationSource::Curated,
                node_count: Some(13),
                charges_apply_by_default: true,
                range_count: 1,
                ranges_shown: 0,
                range_offset: 0,
                range_limit: 50,
                ranges: Vec::new(),
            })
            .collect(),
    }
}
