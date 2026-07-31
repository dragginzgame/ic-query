use super::*;
use std::cell::{Cell, RefCell};

const CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const CONTROLLER_ID: &str = "r7inp-6aaaa-aaaaa-aaabq-cai";
const SUBNET_ID: &str = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
const MODULE_HASH: &str = "51f4be010f23064137defacd627ffbec024c5133210c68ca3b80ab8f257101d6";
const OLDER_MODULE_HASH: &str = "324bfd929805a930cbf6b5f29b4a858ea84452bb551771df7303c769aecb1439";
const FETCHED_AT: &str = "2023-11-14T22:13:20Z";
const PAGE_CANISTER_ID: &str = "2223e-iaaaa-aaaac-awyra-cai";
const PAGE_NEXT_CANISTER_ID: &str = "2223u-yaaaa-aaaal-qutrq-cai";
const METRIC_START: u64 = 1_699_996_400;
const METRIC_END: u64 = 1_700_000_000;

#[test]
fn metric_report_preserves_raw_values_bounds_and_dashboard_provenance() {
    let source = MetricFixture::default();
    let request = metric_request();

    let report = build_ic_metric_report_with_source(&request, &source)
        .expect("bounded Dashboard metric report");
    let text = ic_metric_report_text(&report);

    assert_eq!(source.calls.get(), 1);
    assert_eq!(report.query, request.query);
    assert_eq!(report.returned_series_count, 1);
    assert_eq!(report.returned_observation_count, 2);
    assert_eq!(report.series[0].name, "instruction_rate");
    assert_eq!(report.series[0].observations[0].value, "21089992048.10834");
    assert_eq!(report.provenance.authority, "official_ic_dashboard_api");
    assert!(!report.provenance.certified);
    assert!(!report.provenance.point_in_time_guaranteed);
    assert!(text.contains("metric: instruction-rate"));
    assert!(text.contains("1699996400  21089992048.10834"));
}

#[test]
fn metric_request_bounds_are_validated_before_source_calls() {
    let source = MetricFixture::default();
    let mut request = metric_request();
    request.query.step_secs = 1;

    let error = build_ic_metric_report_with_source(&request, &source)
        .expect_err("oversized metric window must fail");

    assert!(matches!(
        error,
        IcHostError::InvalidRequest { field: "query", .. }
    ));
    assert_eq!(source.calls.get(), 0);
}

#[test]
fn metric_custom_source_must_echo_query_and_valid_series() {
    for mutation in [
        MetricMutation::WrongQuery,
        MetricMutation::WrongSeries,
        MetricMutation::TooManyObservations,
        MetricMutation::EmptyValue,
    ] {
        let source = MetricFixture {
            mutation: RefCell::new(Some(mutation)),
            ..MetricFixture::default()
        };
        let error = build_ic_metric_report_with_source(&metric_request(), &source)
            .expect_err("invalid metric source data must fail");

        assert!(matches!(error, IcHostError::InvalidSourceData { .. }));
    }
}

#[test]
fn metric_kind_uses_exact_official_path_names() {
    for (name, kind) in [
        ("instruction-rate", IcMetricKind::InstructionRate),
        ("message-execution-rate", IcMetricKind::MessageExecutionRate),
        ("cycle-burn-rate", IcMetricKind::CycleBurnRate),
        ("block-rate", IcMetricKind::BlockRate),
        ("ic-node-count", IcMetricKind::IcNodeCount),
        ("ic-subnet-total", IcMetricKind::IcSubnetTotal),
        (
            "registered-canisters-count",
            IcMetricKind::RegisteredCanistersCount,
        ),
        (
            "total-ic-energy-consumption-rate-kwh",
            IcMetricKind::TotalIcEnergyConsumptionRateKwh,
        ),
        ("boundary-nodes-count", IcMetricKind::BoundaryNodesCount),
    ] {
        assert_eq!(kind.to_string(), name);
        assert_eq!(name.parse::<IcMetricKind>(), Ok(kind));
        assert_eq!(serde_json::to_value(kind).expect("metric JSON"), name);
    }
}

#[test]
fn canister_report_preserves_dashboard_values_and_explicit_provenance() {
    let report = build_ic_canister_report_with_source(&request(), &FixtureSource::default())
        .expect("Dashboard canister report");
    let text = ic_canister_report_text(&report);

    assert_eq!(report.provenance.schema_version, 1);
    assert_eq!(report.provenance.network, "ic");
    assert_eq!(report.provenance.authority, "official_ic_dashboard_api");
    assert_eq!(
        report.provenance.source_endpoint,
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT
    );
    assert_eq!(report.provenance.fetched_at, FETCHED_AT);
    assert_eq!(report.canister_id, CANISTER_ID);
    assert_eq!(report.canister_type.as_deref(), Some("ledger"));
    assert_eq!(report.controllers, [CONTROLLER_ID]);
    assert_eq!(report.upgrade_count, Some(2));
    assert_eq!(
        report.upgrades.as_ref().expect("history")[0].proposal_id,
        138_271
    );
    assert!(!report.provenance.certified);
    assert!(!report.provenance.point_in_time_guaranteed);
    assert!(text.contains("authority: official_ic_dashboard_api"));
    assert!(text.contains("latest_upgrade:"));
    assert!(text.contains("certified: no"));
}

#[test]
fn canister_report_preserves_null_upgrade_history() {
    let source = MutatingSource(|data| data.upgrades = None);
    let report =
        build_ic_canister_report_with_source(&request(), &source).expect("nullable history");

    assert_eq!(report.upgrades, None);
    assert_eq!(report.upgrade_count, None);
    assert!(ic_canister_report_text(&report).contains("upgrade_history_available: no"));
}

#[test]
fn request_principal_is_validated_before_calling_source() {
    let source = FixtureSource::default();
    let request = IcCanisterRequest::new(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        1_700_000_000,
        "not a principal",
    );

    let error = build_ic_canister_report_with_source(&request, &source)
        .expect_err("invalid principal must fail");

    assert!(matches!(
        error,
        IcHostError::InvalidPrincipal {
            field: "canister_id",
            ..
        }
    ));
    assert_eq!(source.calls.get(), 0);
}

#[test]
fn custom_source_identity_and_provenance_are_validated() {
    for (mutate, expected_reason) in [
        (
            wrong_canister as fn(&mut IcCanisterSourceData),
            "canister_id",
        ),
        (wrong_endpoint, "source_endpoint"),
        (duplicate_controller, "duplicate controller"),
        (invalid_module_hash, "module_hash"),
        (duplicate_upgrade, "duplicate upgrade"),
    ] {
        let error = build_ic_canister_report_with_source(&request(), &MutatingSource(mutate))
            .expect_err("invalid custom source data must fail");

        assert!(matches!(
            error,
            IcHostError::InvalidSourceData { reason } if reason.contains(expected_reason)
        ));
    }
}

#[test]
fn projection_canonically_orders_controllers_and_upgrade_history() {
    let source = MutatingSource(|data| {
        data.controllers = vec![
            "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
            CONTROLLER_ID.to_string(),
        ];
        data.upgrades.as_mut().expect("history").reverse();
    });

    let report =
        build_ic_canister_report_with_source(&request(), &source).expect("canonical report");

    assert_eq!(
        report.controllers,
        ["qaa6y-5yaaa-aaaaa-aaafa-cai", CONTROLLER_ID]
    );
    assert_eq!(
        report.upgrades.as_ref().expect("history")[0].proposal_id,
        138_271
    );
}

#[test]
fn live_source_rejects_invalid_endpoint_before_http_request() {
    let request = IcSourceRequest::new("not a URL", FETCHED_AT, "test");

    let error = LiveIcSource
        .fetch_canister(&request, CANISTER_ID)
        .expect_err("invalid endpoint must fail");

    assert!(matches!(
        error,
        IcHostError::InvalidEndpoint { endpoint, .. } if endpoint == "not a URL"
    ));
}

#[test]
fn live_metric_source_validates_bounds_before_endpoint_or_http_request() {
    let request = IcSourceRequest::new("not a URL", FETCHED_AT, "test");
    let query = IcMetricQuery::new(
        IcMetricKind::InstructionRate,
        MIN_IC_METRIC_TIMESTAMP - 1,
        MIN_IC_METRIC_TIMESTAMP,
        DEFAULT_IC_METRIC_STEP_SECS,
    );

    let error = LiveIcSource
        .fetch_metric(&request, &query)
        .expect_err("invalid metric bounds must fail first");

    assert!(matches!(
        error,
        IcHostError::InvalidRequest {
            field: "query.start_unix_secs",
            ..
        }
    ));
}

#[test]
fn live_source_rejects_invalid_principal_before_endpoint_or_http_request() {
    let request = IcSourceRequest::new("not a URL", FETCHED_AT, "test");

    let error = LiveIcSource
        .fetch_canister(&request, "not a principal")
        .expect_err("invalid principal must fail first");

    assert!(matches!(
        error,
        IcHostError::InvalidPrincipal {
            field: "canister_id",
            ..
        }
    ));
}

#[test]
fn live_collection_source_enforces_bounds_before_endpoint_or_http_request() {
    let source_request = IcSourceRequest::new("not a URL", FETCHED_AT, "test");

    let count_error = LiveIcSource
        .fetch_canister_count(
            &source_request,
            &IcCanisterFilters {
                query: Some("x".to_string()),
                ..IcCanisterFilters::default()
            },
        )
        .expect_err("invalid count filter must fail first");
    let page_error = LiveIcSource
        .fetch_canister_page(
            &source_request,
            &IcCanisterFilters::default(),
            MAX_IC_CANISTER_PAGE_LIMIT + 1,
            None,
            None,
        )
        .expect_err("invalid page limit must fail first");

    assert!(matches!(
        count_error,
        IcHostError::InvalidRequest {
            field: "filters.query",
            ..
        }
    ));
    assert!(matches!(
        page_error,
        IcHostError::InvalidRequest { field: "limit", .. }
    ));
}

#[test]
fn canister_count_normalizes_filters_and_preserves_bounded_provenance() {
    let source = CollectionFixture::default();
    let filters = IcCanisterFilters {
        has_name: Some(true),
        subnet_id: Some(SUBNET_ID.to_string()),
        controller_id: None,
        languages: vec!["rust".to_string(), "motoko".to_string()],
        canister_types: vec!["ledger".to_string()],
        query: Some("ICP Ledger".to_string()),
    };
    let request = IcCanisterCountRequest::new(
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
        1_700_000_000,
    )
    .with_filters(filters);

    let report = build_ic_canister_count_report_with_source(&request, &source)
        .expect("bounded Dashboard count");

    assert_eq!(source.count_calls.get(), 1);
    assert_eq!(source.page_calls.get(), 0);
    assert_eq!(report.total, 649);
    assert_eq!(report.filters.languages, ["motoko", "rust"]);
    assert!(!report.provenance.certified);
    assert!(!report.provenance.point_in_time_guaranteed);
    assert!(ic_canister_count_report_text(&report).contains("total: 649"));
}

#[test]
fn canister_page_returns_one_validated_slice_without_following_cursor() {
    let source = CollectionFixture::default();
    let request = IcCanisterPageRequest::new(
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
        1_700_000_000,
    )
    .with_limit(2);

    let report = build_ic_canister_page_report_with_source(&request, &source)
        .expect("bounded Dashboard page");

    assert_eq!(source.page_calls.get(), 1);
    assert_eq!(source.count_calls.get(), 0);
    assert_eq!(report.requested_limit, 2);
    assert_eq!(report.returned_count, 2);
    assert_eq!(report.rows[0].canister_id, PAGE_CANISTER_ID);
    assert_eq!(report.next_cursor.as_deref(), Some(PAGE_NEXT_CANISTER_ID));
    assert_eq!(
        report.rows[0].controllers[0].raw_metadata.as_deref(),
        Some("")
    );
    assert!(ic_canister_page_report_text(&report).contains("returned_count: 2"));
}

#[test]
fn canister_page_validates_bound_and_cursor_contract_before_source_call() {
    for mutate in [
        zero_page_limit as fn(&mut IcCanisterPageRequest),
        excessive_page_limit,
        both_page_cursors,
        invalid_page_cursor,
    ] {
        let source = CollectionFixture::default();
        let mut request = IcCanisterPageRequest::new(
            DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
            1_700_000_000,
        );
        mutate(&mut request);

        let error = build_ic_canister_page_report_with_source(&request, &source)
            .expect_err("invalid page request must fail");

        assert!(matches!(
            error,
            IcHostError::InvalidRequest { .. } | IcHostError::InvalidPrincipal { .. }
        ));
        assert_eq!(source.page_calls.get(), 0);
    }
}

#[test]
fn canister_collection_filters_are_validated_before_source_calls() {
    let source = CollectionFixture::default();
    let request = IcCanisterCountRequest::new(
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
        1_700_000_000,
    )
    .with_filters(IcCanisterFilters {
        query: Some("x".to_string()),
        ..IcCanisterFilters::default()
    });

    let error = build_ic_canister_count_report_with_source(&request, &source)
        .expect_err("short Dashboard query must fail");

    assert!(matches!(
        error,
        IcHostError::InvalidRequest {
            field: "filters.query",
            ..
        }
    ));
    assert_eq!(source.count_calls.get(), 0);
}

#[test]
fn canister_page_rejects_invalid_custom_source_order_and_provenance() {
    for mutation in [
        PageSourceMutation::ReverseRows,
        PageSourceMutation::WrongFilters,
        PageSourceMutation::WrongLimit,
    ] {
        let source = CollectionFixture {
            page_mutation: RefCell::new(Some(mutation)),
            ..CollectionFixture::default()
        };
        let request = IcCanisterPageRequest::new(
            DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
            1_700_000_000,
        )
        .with_limit(2);

        let error = build_ic_canister_page_report_with_source(&request, &source)
            .expect_err("invalid source page must fail");

        assert!(matches!(error, IcHostError::InvalidSourceData { .. }));
    }
}

fn request() -> IcCanisterRequest {
    IcCanisterRequest::new(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        1_700_000_000,
        CANISTER_ID,
    )
}

fn metric_request() -> IcMetricRequest {
    IcMetricRequest::new(
        DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT,
        METRIC_END,
        IcMetricQuery::new(
            IcMetricKind::InstructionRate,
            METRIC_START,
            METRIC_END,
            DEFAULT_IC_METRIC_STEP_SECS,
        ),
    )
}

#[derive(Clone, Copy)]
enum MetricMutation {
    WrongQuery,
    WrongSeries,
    TooManyObservations,
    EmptyValue,
}

#[derive(Default)]
struct MetricFixture {
    calls: Cell<usize>,
    mutation: RefCell<Option<MetricMutation>>,
}

impl IcMetricSource for MetricFixture {
    fn fetch_metric(
        &self,
        request: &IcSourceRequest,
        query: &IcMetricQuery,
    ) -> Result<IcMetricSourceData, IcHostError> {
        self.calls.set(self.calls.get() + 1);
        let mut data = IcMetricSourceData {
            source: request.clone(),
            query: query.clone(),
            series: vec![IcMetricSeries {
                name: "instruction_rate".to_string(),
                observations: vec![
                    IcMetricObservation {
                        timestamp_unix_secs: METRIC_START,
                        value: "21089992048.10834".to_string(),
                    },
                    IcMetricObservation {
                        timestamp_unix_secs: METRIC_END,
                        value: "21100000000".to_string(),
                    },
                ],
            }],
        };
        match self.mutation.borrow_mut().take() {
            Some(MetricMutation::WrongQuery) => data.query.step_secs += 1,
            Some(MetricMutation::WrongSeries) => data.series[0].name = "block_rate".to_string(),
            Some(MetricMutation::TooManyObservations) => {
                data.series[0].observations = (0..14)
                    .map(|offset| IcMetricObservation {
                        timestamp_unix_secs: METRIC_START + offset,
                        value: "1".to_string(),
                    })
                    .collect();
            }
            Some(MetricMutation::EmptyValue) => {
                data.series[0].observations[0].value.clear();
            }
            None => {}
        }
        Ok(data)
    }
}

#[derive(Default)]
struct FixtureSource {
    calls: Cell<usize>,
}

impl IcCanisterSource for FixtureSource {
    fn fetch_canister(
        &self,
        request: &IcSourceRequest,
        canister_id: &str,
    ) -> Result<IcCanisterSourceData, IcHostError> {
        self.calls.set(self.calls.get() + 1);
        Ok(source_data(request, canister_id))
    }
}

struct MutatingSource(fn(&mut IcCanisterSourceData));

impl IcCanisterSource for MutatingSource {
    fn fetch_canister(
        &self,
        request: &IcSourceRequest,
        canister_id: &str,
    ) -> Result<IcCanisterSourceData, IcHostError> {
        let mut data = source_data(request, canister_id);
        self.0(&mut data);
        Ok(data)
    }
}

#[derive(Clone, Copy)]
enum PageSourceMutation {
    ReverseRows,
    WrongFilters,
    WrongLimit,
}

#[derive(Default)]
struct CollectionFixture {
    count_calls: Cell<usize>,
    page_calls: Cell<usize>,
    page_mutation: RefCell<Option<PageSourceMutation>>,
}

impl IcCanisterCollectionSource for CollectionFixture {
    fn fetch_canister_count(
        &self,
        request: &IcSourceRequest,
        filters: &IcCanisterFilters,
    ) -> Result<IcCanisterCountSourceData, IcHostError> {
        self.count_calls.set(self.count_calls.get() + 1);
        Ok(IcCanisterCountSourceData {
            source: request.clone(),
            filters: filters.clone(),
            total: 649,
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
        self.page_calls.set(self.page_calls.get() + 1);
        let mut data = page_source_data(request, filters, limit, after, before);
        match self.page_mutation.borrow_mut().take() {
            Some(PageSourceMutation::ReverseRows) => data.rows.reverse(),
            Some(PageSourceMutation::WrongFilters) => data.filters.has_name = Some(true),
            Some(PageSourceMutation::WrongLimit) => data.requested_limit += 1,
            None => {}
        }
        Ok(data)
    }
}

fn page_source_data(
    request: &IcSourceRequest,
    filters: &IcCanisterFilters,
    requested_limit: u16,
    after: Option<&str>,
    before: Option<&str>,
) -> IcCanisterPageSourceData {
    IcCanisterPageSourceData {
        source: request.clone(),
        filters: filters.clone(),
        requested_limit,
        after: after.map(str::to_string),
        before: before.map(str::to_string),
        previous_cursor: None,
        next_cursor: Some(PAGE_NEXT_CANISTER_ID.to_string()),
        rows: vec![
            page_row(PAGE_CANISTER_ID, 918_419, Some("")),
            page_row(PAGE_NEXT_CANISTER_ID, 1_091_549, None),
        ],
    }
}

fn page_row(canister_id: &str, dashboard_id: u64, raw_metadata: Option<&str>) -> IcCanisterPageRow {
    IcCanisterPageRow {
        canister_id: canister_id.to_string(),
        dashboard_id,
        canister_type: None,
        name: String::new(),
        subnet_id: SUBNET_ID.to_string(),
        controllers: vec![IcCanisterPageController {
            principal_id: CONTROLLER_ID.to_string(),
            raw_metadata: raw_metadata.map(str::to_string),
        }],
        language: String::new(),
        module_hash: String::new(),
        dashboard_updated_at: "2026-07-31T05:13:38.882316".to_string(),
    }
}

const fn zero_page_limit(request: &mut IcCanisterPageRequest) {
    request.limit = 0;
}

const fn excessive_page_limit(request: &mut IcCanisterPageRequest) {
    request.limit = MAX_IC_CANISTER_PAGE_LIMIT + 1;
}

fn both_page_cursors(request: &mut IcCanisterPageRequest) {
    request.after = Some(PAGE_CANISTER_ID.to_string());
    request.before = Some(PAGE_NEXT_CANISTER_ID.to_string());
}

fn invalid_page_cursor(request: &mut IcCanisterPageRequest) {
    request.after = Some("not a principal".to_string());
}

fn source_data(request: &IcSourceRequest, canister_id: &str) -> IcCanisterSourceData {
    IcCanisterSourceData {
        source: request.clone(),
        canister_id: canister_id.to_string(),
        dashboard_id: 226_973,
        canister_type: Some("ledger".to_string()),
        name: "ICP Ledger".to_string(),
        subnet_id: SUBNET_ID.to_string(),
        controllers: vec![CONTROLLER_ID.to_string()],
        language: String::new(),
        module_hash: MODULE_HASH.to_string(),
        dashboard_updated_at: "2026-07-30T17:47:41.745647".to_string(),
        upgrades: Some(vec![
            IcCanisterUpgrade {
                executed_timestamp_seconds: 1_756_730_623,
                module_hash: MODULE_HASH.to_string(),
                proposal_id: 138_271,
            },
            IcCanisterUpgrade {
                executed_timestamp_seconds: 1_755_509_658,
                module_hash: OLDER_MODULE_HASH.to_string(),
                proposal_id: 137_924,
            },
        ]),
    }
}

fn wrong_canister(data: &mut IcCanisterSourceData) {
    data.canister_id = "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string();
}

fn wrong_endpoint(data: &mut IcCanisterSourceData) {
    data.source.endpoint = "https://example.com/api/v3".to_string();
}

fn duplicate_controller(data: &mut IcCanisterSourceData) {
    data.controllers.push(CONTROLLER_ID.to_string());
}

fn invalid_module_hash(data: &mut IcCanisterSourceData) {
    data.module_hash = "not a hash".to_string();
}

fn duplicate_upgrade(data: &mut IcCanisterSourceData) {
    let duplicate = data.upgrades.as_ref().expect("history")[0].clone();
    data.upgrades.as_mut().expect("history").push(duplicate);
}
