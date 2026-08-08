use super::*;
use crate::subnet_catalog::MAINNET_NETWORK;
use std::cell::Cell;

const FETCHED_AT: &str = "2026-08-08T12:00:00Z";
const SUBNET_ID: &str = "2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe";
const OPERATOR_ID: &str = "wlnge-zyaaa-aaabw-aaaaa-cai";
const OWNER_ID: &str = "4vh3j-nyc2w-eaan4-vsl33-dguwj-7hlsb-bffh2-exinh-parof-qqlki-lae";
const ADMIN_ID: &str = "bct5z-vccu4-6q4t2-3lb6l-wm43p-ulppt-o5sqq-w6het-rthdz-qp4yn-fqe";

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
