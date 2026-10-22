//! Compiled-canister coverage using Testkit-owned simulator startup and typed calls.

use super::{
    NnsGovernanceMaturityModulation, NnsGovernanceProposalId, NnsGovernanceRewardEvent,
    NnsGovernanceSourceProvenance,
    tests::sample_economics,
    wire::{
        GetMaturityModulationResponse, GetMetricsResult, GovernanceCachedMetrics, GovernanceError,
    },
};
use crate::nns::MAINNET_GOVERNANCE_CANISTER_ID;
use ic_host_artifacts::wasm::{ExportKind, InspectionLimits, inspect};
use ic_platform::{
    candid::contract::Principal,
    pocket_ic::{PocketIc, PocketIcBuilder, RejectCode, Time},
};
use ic_testkit::pic::{
    CanisterInstallExt, InstallSpec, PocketIcStartupConfig,
    fixtures::{RawReplyLimits, RawReplyMethod, ReplyMode, raw_reply_canister},
};
use serde_json::Value;
use std::{collections::BTreeSet, env, fs, time::Duration};

const REPORT_METHOD: u64 = 1;
const REQUEST_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 128,
    allocation: 2048,
    nodes: 4,
    depth: 4,
};
const RESPONSE_LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 2 * 1024 * 1024,
    allocation: 24 * 1024 * 1024,
    nodes: 4,
    depth: 4,
};

#[test]
#[ignore = "requires explicit Testkit server preparation and compiled probe; run make canister-test"]
fn compiled_governance_reports_preserve_values_provenance_and_failures() {
    let path = env::var_os("IC_QUERY_GOVERNANCE_PROBE_WASM")
        .expect("select the compiled probe with IC_QUERY_GOVERNANCE_PROBE_WASM");
    let probe_wasm = fs::read(path).expect("read compiled Governance probe");
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(60))
        .expect("explicit Testkit server")
        .build_instance(
            PocketIcBuilder::new()
                .with_nns_subnet()
                .with_application_subnet(),
        )
        .expect("start Testkit instance");
    let pic = &owned.instance;
    pic.set_time(Time::from_nanos_since_unix_epoch(1_780_531_200_000_000_000));
    let governance = Principal::from_text(MAINNET_GOVERNANCE_CANISTER_ID).unwrap();
    pic.create_canister_with_id(None, None, governance)
        .expect("fixed NNS Governance principal");
    pic.add_cycles(governance, 2_000_000_000_000);
    let mut replies = governance_replies();
    pic.install_canister(
        governance,
        governance_wasm(&replies),
        ic_platform::candid::contract::encode_args(()).unwrap(),
        None,
    );
    let probe = pic
        .try_create_and_install(InstallSpec {
            wasm: probe_wasm,
            init_bytes: ic_platform::candid::contract::encode_args(()).unwrap(),
            cycles: 2_000_000_000_000,
            install_sender: None,
            label: Some("Query Governance probe".into()),
        })
        .expect("install compiled Query probe");
    let mut trailing =
        ic_wire::native::encode_request(REPORT_METHOD, "economics", REQUEST_LIMITS).unwrap();
    trailing.push(0);
    for args in [
        b"invalid Wire".to_vec(),
        ic_wire::native::encode_request(REPORT_METHOD, &42_u64, REQUEST_LIMITS).unwrap(),
        ic_wire::native::encode_response(REPORT_METHOD, "economics", REQUEST_LIMITS).unwrap(),
        vec![0; REQUEST_LIMITS.bytes + 1],
        trailing,
    ] {
        let rejection = pic
            .update_call(probe, Principal::anonymous(), "report", args)
            .expect_err("invalid or over-budget entry arguments must trap");
        assert_eq!(rejection.reject_code, RejectCode::CanisterError);
    }
    assert_reports(pic, probe, governance);
    replies[1].1 =
        ic_platform::candid::contract::encode_one(GetMetricsResult::Err(GovernanceError {
            error_type: 7,
            error_message: "fixture governance failure".into(),
        }))
        .unwrap();
    pic.reinstall_canister(
        governance,
        governance_wasm(&replies),
        ic_platform::candid::contract::encode_args(()).unwrap(),
        None,
    )
    .unwrap();
    let reply = report(pic, probe, "metrics");
    assert_eq!(reply["status"], "error");
    let error = reply["error"].as_str().unwrap();
    assert!(error.contains("code 7"));
    assert!(error.contains("fixture governance failure"));

    replies[0].1 = b"not candid".to_vec();
    pic.reinstall_canister(
        governance,
        governance_wasm(&replies),
        ic_platform::candid::contract::encode_args(()).unwrap(),
        None,
    )
    .unwrap();
    let reply = report(pic, probe, "economics");
    assert_eq!(reply["status"], "error");
    assert!(reply["error"].as_str().unwrap().contains("decode Candid"));

    pic.uninstall_canister(governance, None).unwrap();
    let reply = report(pic, probe, "economics");
    assert_eq!(reply["status"], "error");
    assert!(
        reply["error"]
            .as_str()
            .unwrap()
            .contains("was rejected with code")
    );
}

fn assert_reports(pic: &PocketIc, probe: Principal, governance: Principal) {
    let expected_source =
        serde_json::to_value(NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
            collector_canister_id: probe.to_text(),
        })
        .unwrap();
    for kind in [
        "economics",
        "metrics",
        "reward_event",
        "maturity_modulation",
    ] {
        let reply = report(pic, probe, kind);
        assert_eq!(reply["status"], "ok", "{reply}");
        let report = &reply["report"];
        assert_eq!(report["schema_version"], 1);
        assert_eq!(report["network"], "ic");
        assert_eq!(report["governance_canister_id"], governance.to_text());
        assert_eq!(report["source"], expected_source);
        match kind {
            "economics" => assert_eq!(report["economics"]["neuron_minimum_stake_e8s"], 100_000_000),
            "metrics" => {
                assert_eq!(report["metrics"]["total_supply_icp"], 42);
                assert_eq!(
                    report["metrics"]["not_dissolving_neurons_e8s_buckets"][0]["key"],
                    7
                );
                assert_eq!(
                    report["metrics"]["not_dissolving_neurons_e8s_buckets"][0]["value"],
                    12.5
                );
            }
            "reward_event" => assert_eq!(report["reward_event"]["settled_proposals"][0]["id"], 7),
            "maturity_modulation" => assert_eq!(
                report["maturity_modulation"]["current_value_permyriad"],
                -125
            ),
            _ => unreachable!(),
        }
    }
}

fn report(pic: &PocketIc, probe: Principal, kind: &str) -> Value {
    let args = ic_wire::native::encode_request(REPORT_METHOD, kind, REQUEST_LIMITS).unwrap();
    let bytes = pic
        .update_call(probe, Principal::anonymous(), "report", args)
        .expect("execute compiled report entry point");
    let reply: String = ic_wire::native::decode_response(REPORT_METHOD, &bytes, RESPONSE_LIMITS)
        .expect("bounded Wire report reply");
    serde_json::from_str(&reply).expect("probe JSON report")
}

fn governance_replies() -> Vec<(&'static str, Vec<u8>)> {
    let metrics = GetMetricsResult::Ok(Box::new(GovernanceCachedMetrics {
        total_supply_icp: 42,
        not_dissolving_neurons_e8s_buckets: vec![(7, 12.5)],
        timestamp_seconds: 1_780_531_200,
        ..Default::default()
    }));
    let reward = NnsGovernanceRewardEvent {
        rounds_since_last_distribution: Some(1),
        day_after_genesis: 2,
        actual_timestamp_seconds: 1_780_531_200,
        total_available_e8s_equivalent: 4,
        latest_round_available_e8s_equivalent: Some(5),
        distributed_e8s_equivalent: 6,
        settled_proposals: vec![NnsGovernanceProposalId { id: 7 }],
    };
    let maturity = GetMaturityModulationResponse {
        maturity_modulation: Some(NnsGovernanceMaturityModulation {
            current_value_permyriad: Some(-125),
            updated_at_timestamp_seconds: Some(1_780_531_200),
        }),
    };
    vec![
        (
            "get_network_economics_parameters",
            ic_platform::candid::contract::encode_one(sample_economics()).unwrap(),
        ),
        (
            "get_metrics",
            ic_platform::candid::contract::encode_one(metrics).unwrap(),
        ),
        (
            "get_latest_reward_event",
            ic_platform::candid::contract::encode_one(reward).unwrap(),
        ),
        (
            "get_maturity_modulation",
            ic_platform::candid::contract::encode_one(maturity).unwrap(),
        ),
    ]
}

fn governance_wasm(replies: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let methods: Vec<_> = replies
        .iter()
        .map(|(name, reply)| RawReplyMethod {
            name,
            mode: ReplyMode::Update,
            reply,
        })
        .collect();
    let module = raw_reply_canister(
        &methods,
        RawReplyLimits {
            methods: 16,
            reply_bytes: 1024 * 1024,
            method_name_bytes: 1024,
        },
    )
    .expect("fixture Governance module");
    assert!(
        module.len() <= 1024 * 1024,
        "fixture Governance module limit"
    );
    module
}

#[test]
fn governance_fixture_exports_the_configured_update_methods() {
    let replies = governance_replies();
    let module = governance_wasm(&replies);
    let facts = inspect(
        &module,
        InspectionLimits {
            module_bytes: 1024 * 1024,
            sections: 100,
            exports: 100,
            custom_sections: 10,
        },
    )
    .unwrap();
    assert_eq!(facts.exports["memory"].kind, ExportKind::Memory);
    let actual: BTreeSet<_> = facts
        .exports
        .iter()
        .filter(|(_, export)| export.kind == ExportKind::Function)
        .map(|(name, _)| (*name).to_string())
        .collect();
    let expected = replies
        .iter()
        .map(|(name, _)| format!("canister_update {name}"))
        .collect();
    assert_eq!(actual, expected);
}
