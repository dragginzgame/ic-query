use super::*;
use crate::sns::report::live::types::{DefiniteCanisterSettings, SnsRootExtensions};
use ic_platform::candid::contract::Nat;

#[test]
fn root_inventory_projection_preserves_native_health_and_canonical_roles() {
    let inventory = inventory_response();
    let health = health_response();

    let projected =
        mainnet_sns_canister_inventory(inventory, Ok(health)).expect("project inventory");

    assert_eq!(
        projected
            .canisters
            .iter()
            .map(|canister| canister.role)
            .collect::<Vec<_>>(),
        vec![
            SnsCanisterRole::Root,
            SnsCanisterRole::Governance,
            SnsCanisterRole::Ledger,
            SnsCanisterRole::Swap,
            SnsCanisterRole::Index,
            SnsCanisterRole::Archive,
            SnsCanisterRole::Dapp,
            SnsCanisterRole::Extension,
        ]
    );
    assert_eq!(
        projected.canisters[0].status,
        Some(SnsCanisterStatus::Running)
    );
    assert_eq!(
        projected.canisters[0].module_hash_hex.as_deref(),
        Some("0102")
    );
    assert_eq!(projected.canisters[0].cycles.as_deref(), Some("1000"));
    assert_eq!(
        projected.canisters[0].cycle_balance_status,
        SnsCanisterCycleBalanceStatus::ReportedNonzero
    );
    assert_eq!(projected.canisters[0].memory_size.as_deref(), Some("2000"));
    assert_eq!(
        projected.canisters[0].idle_cycles_burned_per_day.as_deref(),
        Some("30")
    );
    assert_eq!(projected.gaps.len(), 1);
    assert_eq!(projected.health_query_gap, None);
    assert_eq!(
        projected.gaps[0].kind,
        SnsCanisterGapKind::HealthUnsupported
    );
    assert!(!projected.point_in_time_guaranteed);
}

#[test]
fn failed_health_query_preserves_inventory_with_unavailable_cycle_evidence() {
    let projected = mainnet_sns_canister_inventory(
        inventory_response(),
        Err("SNS ingress method get_sns_canisters_summary failed: reject".to_string()),
    )
    .expect("project inventory without health");

    assert_eq!(projected.canisters.len(), 8);
    assert!(projected.canisters.iter().all(|canister| {
        canister.status.is_none()
            && canister.cycles.is_none()
            && canister.cycle_balance_status == SnsCanisterCycleBalanceStatus::Unavailable
    }));
    assert_eq!(
        projected.health_query_gap.as_ref().map(|gap| gap.method),
        Some(SnsCanisterMethod::GetSnsCanistersSummary)
    );
    assert!(
        projected
            .health_query_gap
            .as_ref()
            .is_some_and(|gap| gap.reason.contains("reject"))
    );
    assert_eq!(projected.gaps.len(), 1);
    assert_eq!(
        projected.gaps[0].kind,
        SnsCanisterGapKind::HealthUnsupported
    );
}

#[test]
fn cycle_balance_classification_preserves_raw_naturals() {
    const LARGE: &str = "18446744073709551616000";
    for (cycles, expected_status) in [
        ("0", SnsCanisterCycleBalanceStatus::ReportedZero),
        (LARGE, SnsCanisterCycleBalanceStatus::ReportedNonzero),
    ] {
        let mut canister_status = status(CanisterStatusType::Stopped);
        canister_status.cycles = cycles.parse().unwrap();
        canister_status.memory_size = LARGE.parse().unwrap();
        canister_status.idle_cycles_burned_per_day = LARGE.parse().unwrap();

        let row = canister_row(
            SnsCanisterRole::Root,
            principal(1).to_text(),
            Some(canister_status),
        );

        assert_eq!(row.cycles.as_deref(), Some(cycles));
        assert_eq!(row.cycle_balance_status, expected_status);
        assert_eq!(row.memory_size.as_deref(), Some(LARGE));
        assert_eq!(row.idle_cycles_burned_per_day.as_deref(), Some(LARGE));
    }
}

#[test]
fn root_inventory_projection_retains_relation_specific_gaps() {
    let mut inventory = inventory_response();
    inventory.governance = None;
    inventory.dapps = vec![principal(11)];
    let mut health = health_response();
    health.root = Some(summary(
        principal(9),
        Some(status(CanisterStatusType::Running)),
    ));
    health.governance = Some(summary(
        principal(2),
        Some(status(CanisterStatusType::Running)),
    ));
    health.dapps = vec![
        CanisterSummary {
            canister_id: None,
            status: None,
        },
        summary(principal(10), Some(status(CanisterStatusType::Stopped))),
    ];

    let projected =
        mainnet_sns_canister_inventory(inventory, Ok(health)).expect("project inventory");
    let kinds = projected
        .gaps
        .iter()
        .map(|gap| gap.kind)
        .collect::<Vec<_>>();

    assert!(kinds.contains(&SnsCanisterGapKind::SummaryCanisterIdMismatch));
    assert!(kinds.contains(&SnsCanisterGapKind::InventoryCanisterIdMissing));
    assert!(kinds.contains(&SnsCanisterGapKind::SummaryNotInInventory));
    assert!(kinds.contains(&SnsCanisterGapKind::SummaryCanisterIdMissing));
    assert!(kinds.contains(&SnsCanisterGapKind::SummaryMissing));
}

#[test]
fn root_inventory_projection_rejects_duplicate_inventory_canister_ids() {
    let mut inventory = inventory_response();
    inventory.dapps = vec![principal(1)];

    let error = mainnet_sns_canister_inventory(inventory, Ok(health_response()))
        .expect_err("duplicate inventory canister must fail");

    assert!(matches!(
        error,
        SnsHostError::DuplicateCanisterId {
            first_role,
            duplicate_role,
            ..
        } if first_role == "root" && duplicate_role == "dapp"
    ));
}

#[test]
fn duplicate_health_summaries_do_not_select_response_order_status() {
    let inventory = inventory_response();
    let mut health = health_response();
    health.archives = vec![
        summary(principal(6), Some(status(CanisterStatusType::Running))),
        summary(principal(6), Some(status(CanisterStatusType::Stopped))),
    ];

    let projected =
        mainnet_sns_canister_inventory(inventory, Ok(health)).expect("project inventory");
    let archive = projected
        .canisters
        .iter()
        .find(|canister| canister.role == SnsCanisterRole::Archive)
        .expect("archive inventory row");

    assert_eq!(archive.status, None);
    assert_eq!(
        projected
            .gaps
            .iter()
            .filter(|gap| {
                gap.role == SnsCanisterRole::Archive
                    && gap.kind == SnsCanisterGapKind::DuplicateSummary
            })
            .count(),
        1
    );
    assert!(!projected.gaps.iter().any(|gap| {
        gap.role == SnsCanisterRole::Archive && gap.kind == SnsCanisterGapKind::StatusMissing
    }));
}

fn inventory_response() -> ListSnsCanistersResponse {
    ListSnsCanistersResponse {
        root: Some(principal(1)),
        governance: Some(principal(2)),
        ledger: Some(principal(3)),
        swap: Some(principal(4)),
        index: Some(principal(5)),
        dapps: vec![principal(7)],
        archives: vec![principal(6)],
        extensions: Some(SnsRootExtensions {
            extension_canister_ids: vec![principal(8)],
        }),
    }
}

fn health_response() -> GetSnsCanistersSummaryResponse {
    GetSnsCanistersSummaryResponse {
        root: Some(summary(
            principal(1),
            Some(status(CanisterStatusType::Running)),
        )),
        governance: Some(summary(
            principal(2),
            Some(status(CanisterStatusType::Running)),
        )),
        ledger: Some(summary(
            principal(3),
            Some(status(CanisterStatusType::Running)),
        )),
        swap: Some(summary(
            principal(4),
            Some(status(CanisterStatusType::Stopping)),
        )),
        index: Some(summary(
            principal(5),
            Some(status(CanisterStatusType::Running)),
        )),
        dapps: vec![summary(
            principal(7),
            Some(status(CanisterStatusType::Stopped)),
        )],
        archives: vec![summary(
            principal(6),
            Some(status(CanisterStatusType::Running)),
        )],
    }
}

fn summary(canister_id: Principal, status: Option<CanisterStatusResult>) -> CanisterSummary {
    CanisterSummary {
        canister_id: Some(canister_id),
        status,
    }
}

fn status(status: CanisterStatusType) -> CanisterStatusResult {
    CanisterStatusResult {
        status,
        memory_size: Nat::from(2_000_u64),
        cycles: Nat::from(1_000_u64),
        settings: DefiniteCanisterSettings {
            controllers: vec![principal(20), principal(19)],
        },
        idle_cycles_burned_per_day: Nat::from(30_u64),
        module_hash: Some(vec![1, 2]),
    }
}

fn principal(value: u8) -> Principal {
    Principal::self_authenticating([value])
}
