use super::*;
use std::cell::Cell;

const CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const CONTROLLER_ID: &str = "r7inp-6aaaa-aaaaa-aaabq-cai";
const SUBNET_ID: &str = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";
const MODULE_HASH: &str = "51f4be010f23064137defacd627ffbec024c5133210c68ca3b80ab8f257101d6";
const OLDER_MODULE_HASH: &str = "324bfd929805a930cbf6b5f29b4a858ea84452bb551771df7303c769aecb1439";
const FETCHED_AT: &str = "2023-11-14T22:13:20Z";

#[test]
fn canister_report_preserves_dashboard_values_and_explicit_provenance() {
    let report = build_ic_canister_report_with_source(&request(), &FixtureSource::default())
        .expect("Dashboard canister report");
    let text = ic_canister_report_text(&report);

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, "ic");
    assert_eq!(report.authority, "official_ic_dashboard_api");
    assert_eq!(report.source_endpoint, DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT);
    assert_eq!(report.fetched_at, FETCHED_AT);
    assert_eq!(report.canister_id, CANISTER_ID);
    assert_eq!(report.canister_type.as_deref(), Some("ledger"));
    assert_eq!(report.controllers, [CONTROLLER_ID]);
    assert_eq!(report.upgrade_count, Some(2));
    assert_eq!(
        report.upgrades.as_ref().expect("history")[0].proposal_id,
        138_271
    );
    assert!(!report.certified);
    assert!(!report.point_in_time_guaranteed);
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

fn request() -> IcCanisterRequest {
    IcCanisterRequest::new(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        1_700_000_000,
        CANISTER_ID,
    )
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

fn source_data(request: &IcSourceRequest, canister_id: &str) -> IcCanisterSourceData {
    IcCanisterSourceData {
        source_endpoint: request.endpoint.clone(),
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
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
    data.source_endpoint = "https://example.com/api/v3".to_string();
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
