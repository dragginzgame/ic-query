use ic_query::ic::{
    DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT, DEFAULT_IC_CANISTER_PAGE_LIMIT,
    DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
    DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT, DEFAULT_IC_METRIC_STEP_SECS,
    IcBoundaryNodeDataCenterRow, IcBoundaryNodeDataCentersReport, IcBoundaryNodeDataCentersRequest,
    IcCanisterCountReport, IcCanisterCountRequest, IcCanisterFilters, IcCanisterPageController,
    IcCanisterPageReport, IcCanisterPageRequest, IcCanisterPageRow, IcCanisterReport,
    IcCanisterRequest, IcCanisterUpgrade, IcDashboardReportProvenance, IcMetricKind,
    IcMetricObservation, IcMetricQuery, IcMetricReport, IcMetricRequest, IcMetricSeries,
    MAX_IC_CANISTER_PAGE_LIMIT, ic_boundary_node_data_centers_report_text,
    ic_canister_count_report_text, ic_canister_page_report_text, ic_canister_report_text,
    ic_metric_report_text,
};
#[cfg(feature = "host")]
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcBoundaryNodeDataCentersSourceData,
    IcCanisterCollectionSource, IcCanisterCountSourceData, IcCanisterPageSourceData,
    IcCanisterSource, IcCanisterSourceData, IcHostError, IcMetricSource, IcMetricSourceData,
    IcNetworkSource, IcSourceRequest, LiveIcSource, build_ic_boundary_node_data_centers_report,
    build_ic_boundary_node_data_centers_report_with_source, build_ic_canister_count_report,
    build_ic_canister_count_report_with_source, build_ic_canister_page_report,
    build_ic_canister_page_report_with_source, build_ic_canister_report,
    build_ic_canister_report_with_source, build_ic_metric_report,
    build_ic_metric_report_with_source,
};

const CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const SUBNET_ID: &str = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";

#[test]
fn public_ic_boundary_node_api_is_constructible_serializable_and_renderable() {
    let request = IcBoundaryNodeDataCentersRequest::new(
        DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let report = IcBoundaryNodeDataCentersReport {
        provenance: public_provenance(request.source_endpoint),
        data_center_count: 1,
        total_node_count: 2,
        rows: vec![public_boundary_node_data_center()],
    };

    let text = ic_boundary_node_data_centers_report_text(&report);
    let json = serde_json::to_value(&report).expect("serializable boundary-node report");

    assert!(text.contains("data_center_count: 1"));
    assert_eq!(json["rows"][0]["dc_id"], "da11");
    assert_eq!(json["rows"][0]["total_nodes"], "2");
    assert_eq!(json["certified"], false);
    assert!(json.get("provenance").is_none());
}

#[test]
fn public_ic_metric_api_is_constructible_serializable_and_renderable() {
    let query = IcMetricQuery::new(
        IcMetricKind::InstructionRate,
        1_699_996_400,
        1_700_000_000,
        DEFAULT_IC_METRIC_STEP_SECS,
    );
    let request = IcMetricRequest::new(
        DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT,
        1_700_000_000,
        query.clone(),
    );
    let report = IcMetricReport {
        provenance: public_provenance(request.source_endpoint),
        query,
        returned_series_count: 1,
        returned_observation_count: 1,
        series: vec![IcMetricSeries {
            name: "instruction_rate".to_string(),
            observations: vec![IcMetricObservation {
                timestamp_unix_secs: 1_700_000_000,
                value: "21089992048.10834".to_string(),
            }],
        }],
    };

    let text = ic_metric_report_text(&report);
    let json = serde_json::to_value(&report).expect("serializable metric report");

    assert!(text.contains("metric: instruction-rate"));
    assert_eq!(json["metric"], "instruction-rate");
    assert_eq!(
        json["series"][0]["observations"][0]["value"],
        "21089992048.10834"
    );
    assert!(json.get("provenance").is_none());
    assert!(json.get("query").is_none());
}

#[test]
fn public_ic_canister_api_is_constructible_and_renderable() {
    let request = IcCanisterRequest::new(
        "https://ic-api.internetcomputer.org/api/v3",
        1_700_000_000,
        CANISTER_ID,
    );
    let report = IcCanisterReport {
        provenance: public_provenance(request.source_endpoint.clone()),
        canister_id: request.canister_id.clone(),
        dashboard_id: 226_973,
        canister_type: Some("ledger".to_string()),
        name: "ICP Ledger".to_string(),
        subnet_id: SUBNET_ID.to_string(),
        controllers: vec!["r7inp-6aaaa-aaaaa-aaabq-cai".to_string()],
        language: String::new(),
        module_hash: String::new(),
        dashboard_updated_at: "2026-07-30T17:47:41.745647".to_string(),
        upgrade_count: Some(1),
        upgrades: Some(vec![IcCanisterUpgrade {
            executed_timestamp_seconds: 1_756_730_623,
            module_hash: "51f4be010f23064137defacd627ffbec024c5133210c68ca3b80ab8f257101d6"
                .to_string(),
            proposal_id: 138_271,
        }]),
    };

    let text = ic_canister_report_text(&report);
    let json = serde_json::to_value(&report).expect("serializable canister report");

    assert_eq!(request.now_unix_secs, 1_700_000_000);
    assert!(text.contains("canister_id: ryjl3-tyaaa-aaaaa-aaaba-cai"));
    assert!(text.contains("authority: official_ic_dashboard_api"));
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["source_endpoint"], request.source_endpoint);
    assert!(json.get("provenance").is_none());
}

#[test]
fn public_ic_canister_collection_api_is_constructible_and_renderable() {
    let filters = IcCanisterFilters {
        has_name: Some(true),
        query: Some("ICP Ledger".to_string()),
        ..IcCanisterFilters::default()
    };
    let count_request = IcCanisterCountRequest::new(
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
        1_700_000_000,
    )
    .with_filters(filters.clone());
    let page_request = IcCanisterPageRequest::new(
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
        1_700_000_000,
    )
    .with_filters(filters.clone())
    .with_limit(25)
    .with_after(CANISTER_ID);
    let count = IcCanisterCountReport {
        provenance: public_provenance(count_request.source_endpoint),
        filters: filters.clone(),
        total: 649,
    };
    let page = IcCanisterPageReport {
        provenance: public_provenance(page_request.source_endpoint),
        filters,
        requested_limit: page_request.limit,
        returned_count: 1,
        after: page_request.after,
        before: None,
        previous_cursor: Some(CANISTER_ID.to_string()),
        next_cursor: Some(CANISTER_ID.to_string()),
        rows: vec![public_page_row()],
    };

    assert_eq!(DEFAULT_IC_CANISTER_PAGE_LIMIT, 50);
    assert_eq!(MAX_IC_CANISTER_PAGE_LIMIT, 100);
    assert!(ic_canister_count_report_text(&count).contains("total: 649"));
    assert!(ic_canister_page_report_text(&page).contains("returned_count: 1"));
    let page_json = serde_json::to_value(page).expect("serializable page report");
    assert_eq!(page_json["authority"], "official_ic_dashboard_api");
    assert!(page_json.get("provenance").is_none());
}

#[cfg(feature = "host")]
#[test]
fn public_host_api_exposes_live_and_custom_source_builders() {
    type Builder = fn(&IcCanisterRequest) -> Result<IcCanisterReport, IcHostError>;
    type CustomBuilder =
        fn(&IcCanisterRequest, &dyn IcCanisterSource) -> Result<IcCanisterReport, IcHostError>;

    let _: Builder = build_ic_canister_report;
    let _: CustomBuilder = build_ic_canister_report_with_source;
    let _: fn(&IcCanisterCountRequest) -> Result<IcCanisterCountReport, IcHostError> =
        build_ic_canister_count_report;
    let _: fn(
        &IcCanisterCountRequest,
        &dyn IcCanisterCollectionSource,
    ) -> Result<IcCanisterCountReport, IcHostError> = build_ic_canister_count_report_with_source;
    let _: fn(&IcCanisterPageRequest) -> Result<IcCanisterPageReport, IcHostError> =
        build_ic_canister_page_report;
    let _: fn(
        &IcCanisterPageRequest,
        &dyn IcCanisterCollectionSource,
    ) -> Result<IcCanisterPageReport, IcHostError> = build_ic_canister_page_report_with_source;
    let _: fn(&IcMetricRequest) -> Result<IcMetricReport, IcHostError> = build_ic_metric_report;
    let _: fn(&IcMetricRequest, &dyn IcMetricSource) -> Result<IcMetricReport, IcHostError> =
        build_ic_metric_report_with_source;
    let _: fn(
        &IcBoundaryNodeDataCentersRequest,
    ) -> Result<IcBoundaryNodeDataCentersReport, IcHostError> =
        build_ic_boundary_node_data_centers_report;
    let _: fn(
        &IcBoundaryNodeDataCentersRequest,
        &dyn IcNetworkSource,
    ) -> Result<IcBoundaryNodeDataCentersReport, IcHostError> =
        build_ic_boundary_node_data_centers_report_with_source;
    let _: LiveIcSource = LiveIcSource;
    assert_eq!(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        "https://ic-api.internetcomputer.org/api/v3"
    );

    let source = FixtureSource;
    let request = IcCanisterRequest::new(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        1_700_000_000,
        CANISTER_ID,
    );
    let report =
        build_ic_canister_report_with_source(&request, &source).expect("custom source report");

    assert_eq!(report.canister_id, CANISTER_ID);

    let count_request = IcCanisterCountRequest::new(
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let count = build_ic_canister_count_report_with_source(&count_request, &source)
        .expect("custom count source report");
    assert_eq!(count.total, 1);

    let page_request = IcCanisterPageRequest::new(
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
        1_700_000_000,
    )
    .with_limit(1);
    let page = build_ic_canister_page_report_with_source(&page_request, &source)
        .expect("custom page source report");
    assert_eq!(page.returned_count, 1);

    let metric_request = IcMetricRequest::new(
        DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT,
        1_700_000_000,
        IcMetricQuery::new(
            IcMetricKind::InstructionRate,
            1_699_996_400,
            1_700_000_000,
            DEFAULT_IC_METRIC_STEP_SECS,
        ),
    );
    let metric = build_ic_metric_report_with_source(&metric_request, &source)
        .expect("custom metric source report");
    assert_eq!(metric.returned_observation_count, 1);

    let network_request = IcBoundaryNodeDataCentersRequest::new(
        DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let network = build_ic_boundary_node_data_centers_report_with_source(&network_request, &source)
        .expect("custom network source report");
    assert_eq!(network.data_center_count, 1);
}

#[cfg(feature = "host")]
struct FixtureSource;

#[cfg(feature = "host")]
impl IcCanisterSource for FixtureSource {
    fn fetch_canister(
        &self,
        request: &IcSourceRequest,
        canister_id: &str,
    ) -> Result<IcCanisterSourceData, IcHostError> {
        Ok(IcCanisterSourceData {
            source: request.clone(),
            canister_id: canister_id.to_string(),
            dashboard_id: 1,
            canister_type: None,
            name: String::new(),
            subnet_id: SUBNET_ID.to_string(),
            controllers: Vec::new(),
            language: String::new(),
            module_hash: String::new(),
            dashboard_updated_at: "2026-07-30T17:47:41.745647".to_string(),
            upgrades: None,
        })
    }
}

#[cfg(feature = "host")]
impl IcCanisterCollectionSource for FixtureSource {
    fn fetch_canister_count(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
    ) -> Result<IcCanisterCountSourceData, IcHostError> {
        Ok(IcCanisterCountSourceData {
            source: request.clone(),
            filters: filters.clone(),
            total: 1,
        })
    }

    fn fetch_canister_page(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
        limit: u16,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<IcCanisterPageSourceData, IcHostError> {
        Ok(IcCanisterPageSourceData {
            source: request.clone(),
            filters: filters.clone(),
            requested_limit: limit,
            after: after.map(str::to_string),
            before: before.map(str::to_string),
            previous_cursor: None,
            next_cursor: Some(CANISTER_ID.to_string()),
            rows: vec![public_page_row()],
        })
    }
}

#[cfg(feature = "host")]
impl IcMetricSource for FixtureSource {
    fn fetch_metric(
        &self,
        request: &IcSourceRequest,
        query: &IcMetricQuery,
    ) -> Result<IcMetricSourceData, IcHostError> {
        Ok(IcMetricSourceData {
            source: request.clone(),
            query: query.clone(),
            series: vec![IcMetricSeries {
                name: "instruction_rate".to_string(),
                observations: vec![IcMetricObservation {
                    timestamp_unix_secs: query.end_unix_secs,
                    value: "21089992048.10834".to_string(),
                }],
            }],
        })
    }
}

#[cfg(feature = "host")]
impl IcNetworkSource for FixtureSource {
    fn fetch_boundary_node_data_centers(
        &self,
        request: &IcSourceRequest,
    ) -> Result<IcBoundaryNodeDataCentersSourceData, IcHostError> {
        Ok(IcBoundaryNodeDataCentersSourceData {
            source: request.clone(),
            rows: vec![public_boundary_node_data_center()],
        })
    }
}

fn public_provenance(source_endpoint: impl Into<String>) -> IcDashboardReportProvenance {
    IcDashboardReportProvenance {
        schema_version: 1,
        network: "ic".to_string(),
        authority: "official_ic_dashboard_api".to_string(),
        source_endpoint: source_endpoint.into(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        fetched_by: "ic-query".to_string(),
        certified: false,
        point_in_time_guaranteed: false,
    }
}

fn public_page_row() -> IcCanisterPageRow {
    IcCanisterPageRow {
        canister_id: CANISTER_ID.to_string(),
        dashboard_id: 226_973,
        canister_type: Some("ledger".to_string()),
        name: "ICP Ledger".to_string(),
        subnet_id: SUBNET_ID.to_string(),
        controllers: vec![IcCanisterPageController {
            principal_id: "r7inp-6aaaa-aaaaa-aaabq-cai".to_string(),
            raw_metadata: Some("NNS".to_string()),
        }],
        language: String::new(),
        module_hash: String::new(),
        dashboard_updated_at: "2026-07-30T17:47:41.745647".to_string(),
    }
}

fn public_boundary_node_data_center() -> IcBoundaryNodeDataCenterRow {
    IcBoundaryNodeDataCenterRow {
        dc_id: "da11".to_string(),
        name: "Dallas".to_string(),
        owner: "Equinix Metal".to_string(),
        region: "North America,US,Texas".to_string(),
        latitude: "32.7767".to_string(),
        longitude: "-96.797".to_string(),
        total_nodes: "2".to_string(),
    }
}
