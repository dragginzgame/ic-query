#[cfg(feature = "cloud-engine-host")]
use ic_query::cloud_engine::{
    CloudEngineHostError, CloudEngineOperatorBindingSource, CloudEngineOperatorBindingSourceData,
    CloudEngineOperatorSourceData, CloudEnginePricesSourceData, CloudEngineSource,
    CloudEngineSourceRequest, build_cloud_engine_operator_report_with_source,
    build_cloud_engine_prices_report_with_source,
};
use ic_query::cloud_engine::{
    CloudEngineNodeType, CloudEngineOperatorReport, CloudEnginePriceRow, CloudEnginePricesReport,
    CloudEngineProviderInfoReport, CloudEngineProviderInfoRequest, CloudEngineProviderListReport,
    CloudEngineProviderListRequest, CloudEngineProviderLocation, CloudEngineProviderRow,
    CloudEngineReportContext, DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
    DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT, MAINNET_CLOUD_ENGINE_CANISTER_ID,
    MAX_CLOUD_ENGINE_CYCLE_DECIMAL_DIGITS, MAX_CLOUD_ENGINE_DOMAINS, MAX_CLOUD_ENGINE_PRICE_ROWS,
    MAX_CLOUD_ENGINE_PROVIDER_LOCATIONS, MAX_CLOUD_ENGINE_PROVIDER_SOURCE_ROWS,
    cloud_engine_operator_report_text, cloud_engine_prices_report_text,
    cloud_engine_provider_info_report_text, cloud_engine_provider_list_report_text,
};
use ic_query::ic::IcDashboardReportProvenance;
#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
use ic_query::{
    cloud_engine::{
        CloudEngineListReport, MAX_CLOUD_ENGINE_LIST_ROWS, build_cloud_engine_list_report,
        build_cloud_engine_list_report_with_sources, cloud_engine_list_report_text,
    },
    subnet_catalog::SubnetCatalogListRequest,
};
#[cfg(feature = "dashboard-host")]
use ic_query::{
    cloud_engine::{
        CloudEngineProviderInfoSourceData, CloudEngineProviderListSourceData,
        CloudEngineProviderSource, build_cloud_engine_provider_info_report,
        build_cloud_engine_provider_info_report_with_source,
        build_cloud_engine_provider_list_report,
        build_cloud_engine_provider_list_report_with_source,
    },
    ic::{IcHostError, IcSourceRequest},
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

#[test]
fn public_cloud_engine_provider_reports_preserve_raw_dashboard_evidence() {
    let list_request = CloudEngineProviderListRequest::new(
        "ic",
        DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
        1_800_000_000,
    );
    let info_request = CloudEngineProviderInfoRequest::new(
        "ic",
        DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
        1_800_000_000,
        provider_row().principal_id,
    );
    let provider = provider_row();
    let list = CloudEngineProviderListReport {
        provenance: dashboard_provenance(),
        source_node_provider_count: 2,
        cloud_engine_provider_count: 1,
        providers: vec![provider.clone()],
    };
    let info = CloudEngineProviderInfoReport {
        provenance: dashboard_provenance(),
        cloud_engine_evidence_present: true,
        provider,
    };

    assert!(cloud_engine_provider_list_report_text(&list).contains("CloudEngine providers"));
    assert!(cloud_engine_provider_info_report_text(&info).contains("CloudEngine locations"));
    let json = serde_json::to_value(&info).expect("serialize provider report");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["certified"], false);
    assert_eq!(json["point_in_time_guaranteed"], false);
    assert_eq!(json["provider"]["total_cloud_engine_nodes"], 5);
    assert_eq!(json["provider"]["website"], "example.com");
    assert_eq!(MAX_CLOUD_ENGINE_PROVIDER_SOURCE_ROWS, 1_000);
    assert_eq!(MAX_CLOUD_ENGINE_PROVIDER_LOCATIONS, 100);
    assert_eq!(list_request.network, "ic");
    assert_eq!(info_request.node_provider_id, info.provider.principal_id);
}

#[cfg(feature = "dashboard-host")]
#[test]
fn public_cloud_engine_provider_host_api_accepts_a_dashboard_source() {
    let list_request = CloudEngineProviderListRequest::new(
        "ic",
        DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let info_request = CloudEngineProviderInfoRequest::new(
        "ic",
        DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT,
        1_700_000_000,
        provider_row().principal_id,
    );
    let list = build_cloud_engine_provider_list_report_with_source(&list_request, &ProviderFixture)
        .expect("custom provider list source");
    let info = build_cloud_engine_provider_info_report_with_source(&info_request, &ProviderFixture)
        .expect("custom provider info source");

    assert_eq!(list.source_node_provider_count, 1);
    assert_eq!(list.cloud_engine_provider_count, 1);
    assert!(info.cloud_engine_evidence_present);
    let _: fn(
        &CloudEngineProviderListRequest,
    ) -> Result<CloudEngineProviderListReport, IcHostError> =
        build_cloud_engine_provider_list_report;
    let _: fn(
        &CloudEngineProviderInfoRequest,
    ) -> Result<CloudEngineProviderInfoReport, IcHostError> =
        build_cloud_engine_provider_info_report;
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

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
#[test]
fn public_cloud_engine_list_api_exposes_bounded_registry_join() {
    let _: fn(
        &SubnetCatalogListRequest,
        &CloudEngineSourceRequest,
    ) -> Result<CloudEngineListReport, CloudEngineHostError> = build_cloud_engine_list_report;
    assert_eq!(
        std::mem::size_of_val(&build_cloud_engine_list_report_with_sources),
        0
    );
    let _: fn(&CloudEngineListReport) -> String = cloud_engine_list_report_text;

    assert_eq!(MAX_CLOUD_ENGINE_LIST_ROWS, 100);
}

#[cfg(feature = "cloud-engine-host")]
struct Fixture;

#[cfg(feature = "dashboard-host")]
struct ProviderFixture;

#[cfg(feature = "dashboard-host")]
impl CloudEngineProviderSource for ProviderFixture {
    fn fetch_cloud_engine_provider_list(
        &self,
        request: &IcSourceRequest,
    ) -> Result<CloudEngineProviderListSourceData, IcHostError> {
        Ok(CloudEngineProviderListSourceData {
            source: request.clone(),
            providers: vec![provider_row()],
        })
    }

    fn fetch_cloud_engine_provider_info(
        &self,
        request: &IcSourceRequest,
        node_provider_id: &str,
    ) -> Result<CloudEngineProviderInfoSourceData, IcHostError> {
        let mut provider = provider_row();
        provider.principal_id = node_provider_id.to_string();
        Ok(CloudEngineProviderInfoSourceData {
            source: request.clone(),
            provider,
        })
    }
}

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

#[cfg(feature = "cloud-engine-host")]
impl CloudEngineOperatorBindingSource for Fixture {
    fn fetch_operator_binding(
        &self,
        request: &CloudEngineSourceRequest,
        subnet_id: &str,
    ) -> Result<CloudEngineOperatorBindingSourceData, CloudEngineHostError> {
        Ok(CloudEngineOperatorBindingSourceData {
            source: request.clone(),
            subnet_id: subnet_id.to_string(),
            operator_canister_id: Some(OPERATOR_ID.to_string()),
            query_call_count: 1,
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

fn dashboard_provenance() -> IcDashboardReportProvenance {
    IcDashboardReportProvenance {
        schema_version: 1,
        network: "ic".to_string(),
        authority: "official_ic_dashboard_api".to_string(),
        source_endpoint: DEFAULT_CLOUD_ENGINE_PROVIDER_SOURCE_ENDPOINT.to_string(),
        fetched_at: "2026-08-08T12:00:00Z".to_string(),
        fetched_by: "fixture".to_string(),
        certified: false,
        point_in_time_guaranteed: false,
    }
}

fn provider_row() -> CloudEngineProviderRow {
    let location = CloudEngineProviderLocation {
        dc_key: "br1".to_string(),
        display_name: "Brussels".to_string(),
        latitude: 50.8386,
        longitude: 4.3475,
        owner: "Digital Realty".to_string(),
        region: "Europe,BE,Brussels Capital".to_string(),
    };
    CloudEngineProviderRow {
        principal_id: "rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae".to_string(),
        display_name: "Provider".to_string(),
        website: Some("example.com".to_string()),
        logo_url: None,
        location_count: 1,
        locations: vec![location.clone()],
        cloud_engine_location_count: 1,
        cloud_engine_locations: vec![location],
        total_cloud_engine_nodes: 5,
        total_cloud_engine_unassigned_nodes: 4,
        total_cloud_engines: 1,
        total_node_allowance: 7,
        total_nodes: 8,
        total_rewardable_nodes: 6,
        total_subnets: 2,
        total_unassigned_nodes: 3,
    }
}
