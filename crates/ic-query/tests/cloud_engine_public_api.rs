#[cfg(feature = "cloud-engine-host")]
use ic_query::cloud_engine::{
    CloudEngineHostError, CloudEngineOperatorSourceData, CloudEnginePricesSourceData,
    CloudEngineSource, CloudEngineSourceRequest, build_cloud_engine_operator_report_with_source,
    build_cloud_engine_prices_report_with_source,
};
use ic_query::cloud_engine::{
    CloudEngineNodeType, CloudEngineOperatorReport, CloudEnginePriceRow, CloudEnginePricesReport,
    CloudEngineReportContext, DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT,
    MAINNET_CLOUD_ENGINE_CANISTER_ID, MAX_CLOUD_ENGINE_CYCLE_DECIMAL_DIGITS,
    MAX_CLOUD_ENGINE_DOMAINS, MAX_CLOUD_ENGINE_PRICE_ROWS, cloud_engine_operator_report_text,
    cloud_engine_prices_report_text,
};

const SUBNET_ID: &str = "2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe";
const OPERATOR_ID: &str = "wlnge-zyaaa-aaabw-aaaaa-cai";

#[test]
fn public_cloud_engine_reports_are_constructible_serializable_and_renderable() {
    let operator = CloudEngineOperatorReport {
        context: context(5),
        subnet_id: SUBNET_ID.to_string(),
        operator_binding_present: true,
        operator_canister_id: Some(OPERATOR_ID.to_string()),
        engine_owner: Some("aaaaa-aa".to_string()),
        platform_admin: None,
        caffeine_enabled: Some(true),
        claimed_domain_count: Some(1),
        claimed_domains: Some(vec!["example.com".to_string()]),
    };
    let prices = CloudEnginePricesReport {
        context: context(2),
        network_fee: 0.25,
        price_count: 1,
        prices: vec![price_row()],
    };

    assert!(cloud_engine_operator_report_text(&operator).contains("example.com"));
    assert!(cloud_engine_prices_report_text(&prices).contains("Marketplace prices"));
    let operator_json = serde_json::to_value(operator).expect("serialize operator report");
    let prices_json = serde_json::to_value(prices).expect("serialize prices report");
    assert_eq!(operator_json["certified"], false);
    assert_eq!(operator_json["point_in_time_guaranteed"], false);
    assert_eq!(
        prices_json["prices"][0]["net_cycles_per_month"],
        "1000000000000"
    );
    assert_eq!(MAX_CLOUD_ENGINE_DOMAINS, 100);
    assert_eq!(MAX_CLOUD_ENGINE_CYCLE_DECIMAL_DIGITS, 256);
    assert_eq!(MAX_CLOUD_ENGINE_PRICE_ROWS, 1_000);
}

#[cfg(feature = "cloud-engine-host")]
#[test]
fn public_cloud_engine_host_api_accepts_a_custom_source() {
    let request = CloudEngineSourceRequest::from_unix_secs(
        "ic",
        DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT,
        1_700_000_000,
        "fixture",
    );

    let operator = build_cloud_engine_operator_report_with_source(&request, SUBNET_ID, &Fixture)
        .expect("custom operator source");
    let prices = build_cloud_engine_prices_report_with_source(&request, &Fixture)
        .expect("custom prices source");

    assert_eq!(operator.context.fetched_at, "2023-11-14T22:13:20Z");
    assert_eq!(operator.context.query_call_count, 5);
    assert_eq!(prices.context.query_call_count, 2);
}

#[cfg(feature = "cloud-engine-host")]
struct Fixture;

#[cfg(feature = "cloud-engine-host")]
impl CloudEngineSource for Fixture {
    fn fetch_operator(
        &self,
        request: &CloudEngineSourceRequest,
        subnet_id: &str,
    ) -> Result<CloudEngineOperatorSourceData, CloudEngineHostError> {
        Ok(CloudEngineOperatorSourceData {
            source: request.clone(),
            subnet_id: subnet_id.to_string(),
            operator_canister_id: Some(OPERATOR_ID.to_string()),
            engine_owner: Some("aaaaa-aa".to_string()),
            platform_admin: None,
            caffeine_enabled: Some(true),
            claimed_domains: Some(vec!["example.com".to_string()]),
            query_call_count: 5,
        })
    }

    fn fetch_prices(
        &self,
        request: &CloudEngineSourceRequest,
    ) -> Result<CloudEnginePricesSourceData, CloudEngineHostError> {
        Ok(CloudEnginePricesSourceData {
            source: request.clone(),
            network_fee: 0.25,
            prices: vec![price_row()],
            query_call_count: 2,
        })
    }
}

fn context(query_call_count: usize) -> CloudEngineReportContext {
    CloudEngineReportContext {
        schema_version: 1,
        network: "ic".to_string(),
        authority: "cloud_engine_control_plane_canister".to_string(),
        engine_canister_id: MAINNET_CLOUD_ENGINE_CANISTER_ID.to_string(),
        fetched_at: "2026-08-08T12:00:00Z".to_string(),
        source_endpoint: DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT.to_string(),
        fetched_by: "fixture".to_string(),
        certified: false,
        point_in_time_guaranteed: false,
        query_call_count,
    }
}

fn price_row() -> CloudEnginePriceRow {
    CloudEnginePriceRow {
        key: "type4.1".to_string(),
        node_type: CloudEngineNodeType::Type4_1,
        data_center_id: None,
        provider_id: None,
        net_cycles_per_month: "1000000000000".to_string(),
        gross_cycles_per_month: "1250000000000".to_string(),
        updated_at_unix_nanos: 1_785_946_128_242_156_275,
    }
}
