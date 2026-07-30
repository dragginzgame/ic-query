#[cfg(feature = "host")]
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcCanisterSource, IcCanisterSourceData, IcHostError,
    IcSourceRequest, LiveIcSource, build_ic_canister_report, build_ic_canister_report_with_source,
};
use ic_query::ic::{
    IcCanisterReport, IcCanisterRequest, IcCanisterUpgrade, ic_canister_report_text,
};

const CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const SUBNET_ID: &str = "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe";

#[test]
fn public_ic_canister_api_is_constructible_and_renderable() {
    let request = IcCanisterRequest::new(
        "https://ic-api.internetcomputer.org/api/v3",
        1_700_000_000,
        CANISTER_ID,
    );
    let report = IcCanisterReport {
        schema_version: 1,
        network: "ic".to_string(),
        authority: "official_ic_dashboard_api".to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        fetched_by: "ic-query".to_string(),
        certified: false,
        point_in_time_guaranteed: false,
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

    assert_eq!(request.now_unix_secs, 1_700_000_000);
    assert!(text.contains("canister_id: ryjl3-tyaaa-aaaaa-aaaba-cai"));
    assert!(text.contains("authority: official_ic_dashboard_api"));
}

#[cfg(feature = "host")]
#[test]
fn public_host_api_exposes_live_and_custom_source_builders() {
    type Builder = fn(&IcCanisterRequest) -> Result<IcCanisterReport, IcHostError>;
    type CustomBuilder =
        fn(&IcCanisterRequest, &dyn IcCanisterSource) -> Result<IcCanisterReport, IcHostError>;

    let _: Builder = build_ic_canister_report;
    let _: CustomBuilder = build_ic_canister_report_with_source;
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
            source_endpoint: request.endpoint.clone(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
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
