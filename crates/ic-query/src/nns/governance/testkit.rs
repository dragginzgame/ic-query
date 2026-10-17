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
use candid::Principal;
use ic_testkit::pic::{
    CandidCallExt, CanisterInstallExt, InstallSpec, PocketIc, PocketIcBuilder,
    PocketIcStartupConfig,
};
use ic_testkit::pocket_ic::Time;
use ic_testkit_candid::DecodeLimits;
use serde_json::Value;
use std::{env, fmt::Write, fs, time::Duration};

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
        candid::encode_args(()).unwrap(),
        None,
    );
    let probe = pic
        .try_create_and_install(
            InstallSpec::new(
                probe_wasm,
                candid::encode_args(()).unwrap(),
                2_000_000_000_000,
            )
            .label("Query Governance probe"),
        )
        .expect("install compiled Query probe");
    assert_reports(pic, probe, governance);
    replies[1].1 = candid::encode_one(GetMetricsResult::Err(GovernanceError {
        error_type: 7,
        error_message: "fixture governance failure".into(),
    }))
    .unwrap();
    pic.reinstall_canister(
        governance,
        governance_wasm(&replies),
        candid::encode_args(()).unwrap(),
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
        candid::encode_args(()).unwrap(),
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
    let reply: String = pic
        .update_candid(
            probe,
            "report",
            (kind.to_string(),),
            DecodeLimits {
                reply_bytes: 2 * 1024 * 1024,
                decoding_quota: 64_000_000,
                skipping_quota: 100_000,
                type_entries: 4_096,
                header_bytes: 64 * 1024,
            },
        )
        .expect("execute compiled report entry point");
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
            candid::encode_one(sample_economics()).unwrap(),
        ),
        ("get_metrics", candid::encode_one(metrics).unwrap()),
        (
            "get_latest_reward_event",
            candid::encode_one(reward).unwrap(),
        ),
        (
            "get_maturity_modulation",
            candid::encode_one(maturity).unwrap(),
        ),
    ]
}

fn governance_wasm(replies: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let bytes = replies.iter().map(|(_, bytes)| bytes.len()).sum::<usize>();
    let mut module = format!(
        r#"(module
        (import "ic0" "msg_reply_data_append" (func $append (param i32 i32)))
        (import "ic0" "msg_reply" (func $reply))
        (memory (export "memory") {})"#,
        bytes.div_ceil(65_536).max(1)
    );
    let mut offset = 0;
    for (method, reply) in replies {
        write!(module, "(data (i32.const {offset}) \"").unwrap();
        for byte in reply {
            write!(module, "\\{byte:02x}").unwrap();
        }
        write!(module, "\") (func (export \"canister_update {method}\") i32.const {offset} i32.const {} call $append call $reply)", reply.len()).unwrap();
        offset += reply.len();
    }
    module.push(')');
    wat::parse_str(module).expect("fixture Governance module")
}
