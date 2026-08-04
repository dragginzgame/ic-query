use super::{fixtures::*, *};

#[test]
fn sns_canister_report_resolves_inventory_health_and_typed_gaps() {
    let report =
        build_sns_canister_report_with_source(&info_request("1"), &FixtureSnsCanisterSource)
            .expect("sns canister report");
    let text = sns_canister_report_text(&report);

    assert_eq!(report.schema_version, SNS_CANISTER_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.inventory_method, SnsCanisterMethod::ListSnsCanisters);
    assert_eq!(
        report.health_method,
        SnsCanisterMethod::GetSnsCanistersSummary
    );
    assert_eq!(report.health_call_type, SnsCanisterCallType::IngressUpdate);
    assert!(!report.health_update_canister_list);
    assert!(!report.point_in_time_guaranteed);
    assert_eq!(report.canister_count, 2);
    assert_eq!(report.health_status_count, 1);
    assert_eq!(report.reported_zero_cycles_count, 1);
    assert_eq!(report.cycles_unavailable_count, 1);
    assert_eq!(report.gap_count, 1);
    assert_eq!(report.health_query_gap, None);
    assert_eq!(report.canisters[0].role, SnsCanisterRole::Root);
    assert_eq!(
        report.canisters[0].cycle_balance_status,
        SnsCanisterCycleBalanceStatus::ReportedZero
    );
    assert_eq!(report.canisters[1].role, SnsCanisterRole::Extension);
    assert_eq!(
        report.canisters[1].cycle_balance_status,
        SnsCanisterCycleBalanceStatus::Unavailable
    );
    assert_eq!(report.gaps[0].kind, SnsCanisterGapKind::HealthUnsupported);
    assert!(text.contains("health_call_type: ingress_update"));
    assert!(text.contains("health_update_canister_list: no"));
    assert!(text.contains("reported_zero_cycles_count: 1"));
    assert!(text.contains("cycles_unavailable_count: 1"));
    assert!(text.contains("health_query_status: succeeded"));
    assert!(text.contains("1.91 MiB"));
    assert!(text.contains("health_unsupported"));
}

#[test]
fn live_sns_canister_source_rejects_non_mainnet_before_agent_construction() {
    let request = SnsSourceRequest::new(
        "local",
        "not a valid endpoint",
        "2026-07-30T00:00:00Z",
        "test",
    );
    let sns = fixture_sns_a();

    let error = LiveSnsSource
        .fetch_sns_canisters(&request, &sns)
        .expect_err("non-mainnet must fail");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn sns_canister_report_retains_inventory_when_health_is_unavailable() {
    let report = build_sns_canister_report_with_source(
        &info_request("1"),
        &MutatingSnsCanisterSource(make_health_unavailable),
    )
    .expect("partial SNS canister report");
    let text = sns_canister_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize partial SNS canister report");

    assert_eq!(report.canister_count, 2);
    assert_eq!(report.health_status_count, 0);
    assert_eq!(report.reported_zero_cycles_count, 0);
    assert_eq!(report.cycles_unavailable_count, 2);
    assert!(report.canisters.iter().all(|canister| {
        canister.cycles.is_none()
            && canister.cycle_balance_status == SnsCanisterCycleBalanceStatus::Unavailable
    }));
    assert!(
        report
            .health_query_gap
            .as_ref()
            .is_some_and(|gap| gap.reason == "health failed")
    );
    assert!(text.contains("health_query_status: failed"));
    assert!(text.contains("health_query_gap:"));
    assert!(text.contains("health failed"));
    assert_eq!(json["reported_zero_cycles_count"], 0);
    assert_eq!(json["cycles_unavailable_count"], 2);
    assert_eq!(
        json["health_query_gap"]["method"],
        "get_sns_canisters_summary"
    );
    assert_eq!(json["canisters"][0]["cycle_balance_status"], "unavailable");
}

#[test]
fn sns_canister_report_rejects_invalid_custom_source_evidence() {
    for (mutate, expected_reason) in [
        (
            wrong_inventory_method as fn(&mut MainnetSnsCanisterInventory),
            "inventory_method",
        ),
        (wrong_health_call_type, "health_call_type"),
        (
            enable_health_inventory_update,
            "health_update_canister_list",
        ),
        (invalidate_controller, "controller"),
        (invalidate_cycle_balance_status, "cycle_balance_status"),
        (add_conflicting_health_query_gap, "health_query_gap"),
        (wrong_health_query_gap_method, "health_query_gap method"),
        (empty_health_query_gap_reason, "empty reason"),
    ] {
        let error = build_sns_canister_report_with_source(
            &info_request("1"),
            &MutatingSnsCanisterSource(mutate),
        )
        .expect_err("invalid custom Root inventory must fail");

        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS Root canister inventory",
                reason,
            } if reason.contains(expected_reason)
        ));
    }
}

struct MutatingSnsCanisterSource(fn(&mut MainnetSnsCanisterInventory));

delegate_sns_discovery!(MutatingSnsCanisterSource);

impl SnsCanisterSource for MutatingSnsCanisterSource {
    fn fetch_sns_canisters(
        &self,
        request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsCanisterInventory, SnsHostError> {
        let mut inventory = FixtureSnsCanisterSource.fetch_sns_canisters(request, sns)?;
        self.0(&mut inventory);
        Ok(inventory)
    }
}

const fn wrong_inventory_method(inventory: &mut MainnetSnsCanisterInventory) {
    inventory.inventory_method = SnsCanisterMethod::GetMetrics;
}

const fn wrong_health_call_type(inventory: &mut MainnetSnsCanisterInventory) {
    inventory.health_call_type = SnsCanisterCallType::Query;
}

const fn enable_health_inventory_update(inventory: &mut MainnetSnsCanisterInventory) {
    inventory.health_update_canister_list = true;
}

fn invalidate_controller(inventory: &mut MainnetSnsCanisterInventory) {
    inventory.canisters[1].controllers = vec!["not a principal".to_string()];
}

fn invalidate_cycle_balance_status(inventory: &mut MainnetSnsCanisterInventory) {
    inventory.canisters[0].cycle_balance_status = SnsCanisterCycleBalanceStatus::ReportedNonzero;
}

fn add_conflicting_health_query_gap(inventory: &mut MainnetSnsCanisterInventory) {
    inventory.health_query_gap = Some(SnsCanisterHealthQueryGap {
        method: SnsCanisterMethod::GetSnsCanistersSummary,
        reason: "health failed".to_string(),
    });
}

fn make_health_unavailable(inventory: &mut MainnetSnsCanisterInventory) {
    for canister in &mut inventory.canisters {
        canister.status = None;
        canister.module_hash_hex = None;
        canister.cycles = None;
        canister.cycle_balance_status = SnsCanisterCycleBalanceStatus::Unavailable;
        canister.memory_size = None;
        canister.idle_cycles_burned_per_day = None;
        canister.controllers.clear();
    }
    inventory.health_query_gap = Some(SnsCanisterHealthQueryGap {
        method: SnsCanisterMethod::GetSnsCanistersSummary,
        reason: "health failed".to_string(),
    });
}

fn wrong_health_query_gap_method(inventory: &mut MainnetSnsCanisterInventory) {
    make_health_unavailable(inventory);
    inventory
        .health_query_gap
        .as_mut()
        .expect("health query gap")
        .method = SnsCanisterMethod::GetMetrics;
}

fn empty_health_query_gap_reason(inventory: &mut MainnetSnsCanisterInventory) {
    make_health_unavailable(inventory);
    inventory
        .health_query_gap
        .as_mut()
        .expect("health query gap")
        .reason = " ".to_string();
}
