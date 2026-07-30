//! Module: sns::report::live::convert::canisters
//!
//! Responsibility: join SNS Root inventory and health wire responses.
//! Does not own: Root transport, SNS lookup, report assembly, or rendering.
//! Boundary: retains inventory authority, native fields, canonical order, and typed gaps.

use crate::{
    hex::hex_bytes,
    sns::report::{
        MainnetSnsCanisterInventory, SnsCanisterGap, SnsCanisterGapKind, SnsCanisterRole,
        SnsCanisterRow, SnsCanisterStatus, SnsHostError,
        live::types::{
            CanisterStatusResult, CanisterStatusType, CanisterSummary,
            GetSnsCanistersSummaryResponse, ListSnsCanistersResponse,
        },
    },
};
use candid::Principal;
use std::collections::BTreeMap;

const INVENTORY_METHOD: &str = "list_sns_canisters";
const HEALTH_METHOD: &str = "get_sns_canisters_summary";
const HEALTH_CALL_TYPE: &str = "ingress_update";

pub(in crate::sns::report::live) fn mainnet_sns_canister_inventory(
    inventory: ListSnsCanistersResponse,
    health: GetSnsCanistersSummaryResponse,
) -> Result<MainnetSnsCanisterInventory, SnsHostError> {
    let mut canisters = Vec::new();
    let mut gaps = Vec::new();

    push_singleton(
        SnsCanisterRole::Root,
        inventory.root,
        health.root,
        &mut canisters,
        &mut gaps,
    );
    push_singleton(
        SnsCanisterRole::Governance,
        inventory.governance,
        health.governance,
        &mut canisters,
        &mut gaps,
    );
    push_singleton(
        SnsCanisterRole::Ledger,
        inventory.ledger,
        health.ledger,
        &mut canisters,
        &mut gaps,
    );
    push_singleton(
        SnsCanisterRole::Swap,
        inventory.swap,
        health.swap,
        &mut canisters,
        &mut gaps,
    );
    push_singleton(
        SnsCanisterRole::Index,
        inventory.index,
        health.index,
        &mut canisters,
        &mut gaps,
    );
    push_many(
        SnsCanisterRole::Archive,
        inventory.archives,
        health.archives,
        &mut canisters,
        &mut gaps,
    );
    push_many(
        SnsCanisterRole::Dapp,
        inventory.dapps,
        health.dapps,
        &mut canisters,
        &mut gaps,
    );

    for canister_id in inventory
        .extensions
        .into_iter()
        .flat_map(|extensions| extensions.extension_canister_ids)
    {
        let canister_id = canister_id.to_text();
        canisters.push(canister_row(
            SnsCanisterRole::Extension,
            canister_id.clone(),
            None,
        ));
        gaps.push(gap(
            SnsCanisterGapKind::HealthUnsupported,
            SnsCanisterRole::Extension,
            Some(canister_id),
            None,
        ));
    }

    let mut inventory = MainnetSnsCanisterInventory {
        inventory_method: INVENTORY_METHOD.to_string(),
        health_method: HEALTH_METHOD.to_string(),
        health_call_type: HEALTH_CALL_TYPE.to_string(),
        health_update_canister_list: false,
        point_in_time_guaranteed: false,
        canisters,
        gaps,
    };
    crate::sns::report::source::canonicalize_mainnet_sns_canister_inventory(&mut inventory)?;
    Ok(inventory)
}

fn push_singleton(
    role: SnsCanisterRole,
    inventory_canister_id: Option<Principal>,
    summary: Option<CanisterSummary>,
    canisters: &mut Vec<SnsCanisterRow>,
    gaps: &mut Vec<SnsCanisterGap>,
) {
    let Some(inventory_canister_id) = inventory_canister_id.map(|value| value.to_text()) else {
        gaps.push(gap(
            SnsCanisterGapKind::InventoryCanisterIdMissing,
            role,
            None,
            summary_canister_id(summary.as_ref()),
        ));
        push_unmatched_summary_gap(role, summary, gaps);
        return;
    };

    let Some(summary) = summary else {
        canisters.push(canister_row(role, inventory_canister_id.clone(), None));
        gaps.push(gap(
            SnsCanisterGapKind::SummaryMissing,
            role,
            Some(inventory_canister_id),
            None,
        ));
        return;
    };
    let Some(summary_canister_id) = summary.canister_id.map(|value| value.to_text()) else {
        canisters.push(canister_row(role, inventory_canister_id.clone(), None));
        gaps.push(gap(
            SnsCanisterGapKind::SummaryCanisterIdMissing,
            role,
            Some(inventory_canister_id),
            None,
        ));
        return;
    };
    if summary_canister_id != inventory_canister_id {
        canisters.push(canister_row(role, inventory_canister_id.clone(), None));
        gaps.push(gap(
            SnsCanisterGapKind::SummaryCanisterIdMismatch,
            role,
            Some(inventory_canister_id),
            Some(summary_canister_id),
        ));
        return;
    }

    push_matched_summary(role, inventory_canister_id, summary.status, canisters, gaps);
}

fn push_many(
    role: SnsCanisterRole,
    inventory_canister_ids: Vec<Principal>,
    summaries: Vec<CanisterSummary>,
    canisters: &mut Vec<SnsCanisterRow>,
    gaps: &mut Vec<SnsCanisterGap>,
) {
    let inventory_canister_ids = inventory_canister_ids
        .into_iter()
        .map(|value| value.to_text())
        .collect::<Vec<_>>();
    let mut summaries_by_canister = BTreeMap::<String, Vec<CanisterSummary>>::new();

    for summary in summaries {
        let Some(summary_canister_id) = summary.canister_id.map(|value| value.to_text()) else {
            gaps.push(gap(
                SnsCanisterGapKind::SummaryCanisterIdMissing,
                role,
                None,
                None,
            ));
            continue;
        };
        summaries_by_canister
            .entry(summary_canister_id)
            .or_default()
            .push(summary);
    }

    for inventory_canister_id in &inventory_canister_ids {
        let Some(mut summaries) = summaries_by_canister.remove(inventory_canister_id) else {
            canisters.push(canister_row(role, inventory_canister_id.clone(), None));
            gaps.push(gap(
                SnsCanisterGapKind::SummaryMissing,
                role,
                Some(inventory_canister_id.clone()),
                None,
            ));
            continue;
        };
        let summary = summaries.remove(0);
        push_matched_summary(
            role,
            inventory_canister_id.clone(),
            summary.status,
            canisters,
            gaps,
        );
        for _ in summaries {
            gaps.push(gap(
                SnsCanisterGapKind::DuplicateSummary,
                role,
                Some(inventory_canister_id.clone()),
                Some(inventory_canister_id.clone()),
            ));
        }
    }

    for (summary_canister_id, summaries) in summaries_by_canister {
        for _ in summaries {
            gaps.push(gap(
                SnsCanisterGapKind::SummaryNotInInventory,
                role,
                None,
                Some(summary_canister_id.clone()),
            ));
        }
    }
}

fn push_matched_summary(
    role: SnsCanisterRole,
    canister_id: String,
    status: Option<CanisterStatusResult>,
    canisters: &mut Vec<SnsCanisterRow>,
    gaps: &mut Vec<SnsCanisterGap>,
) {
    if status.is_none() {
        gaps.push(gap(
            SnsCanisterGapKind::StatusMissing,
            role,
            Some(canister_id.clone()),
            Some(canister_id.clone()),
        ));
    }
    canisters.push(canister_row(role, canister_id, status));
}

fn push_unmatched_summary_gap(
    role: SnsCanisterRole,
    summary: Option<CanisterSummary>,
    gaps: &mut Vec<SnsCanisterGap>,
) {
    let Some(summary) = summary else {
        return;
    };
    match summary.canister_id.map(|value| value.to_text()) {
        Some(summary_canister_id) => gaps.push(gap(
            SnsCanisterGapKind::SummaryNotInInventory,
            role,
            None,
            Some(summary_canister_id),
        )),
        None => gaps.push(gap(
            SnsCanisterGapKind::SummaryCanisterIdMissing,
            role,
            None,
            None,
        )),
    }
}

fn summary_canister_id(summary: Option<&CanisterSummary>) -> Option<String> {
    summary
        .and_then(|summary| summary.canister_id)
        .map(|value| value.to_text())
}

fn canister_row(
    role: SnsCanisterRole,
    canister_id: String,
    status: Option<CanisterStatusResult>,
) -> SnsCanisterRow {
    let Some(status) = status else {
        return SnsCanisterRow {
            role,
            canister_id,
            status: None,
            module_hash_hex: None,
            cycles: None,
            memory_size: None,
            idle_cycles_burned_per_day: None,
            controllers: Vec::new(),
        };
    };
    let mut controllers = status
        .settings
        .controllers
        .into_iter()
        .map(|controller| controller.to_text())
        .collect::<Vec<_>>();
    controllers.sort();

    SnsCanisterRow {
        role,
        canister_id,
        status: Some(match status.status {
            CanisterStatusType::Running => SnsCanisterStatus::Running,
            CanisterStatusType::Stopping => SnsCanisterStatus::Stopping,
            CanisterStatusType::Stopped => SnsCanisterStatus::Stopped,
        }),
        module_hash_hex: status.module_hash.map(|value| hex_bytes(&value)),
        cycles: Some(nat_decimal_text(&status.cycles)),
        memory_size: Some(nat_decimal_text(&status.memory_size)),
        idle_cycles_burned_per_day: Some(nat_decimal_text(&status.idle_cycles_burned_per_day)),
        controllers,
    }
}

fn nat_decimal_text(value: &candid::Nat) -> String {
    value.to_string().replace('_', "")
}

const fn gap(
    kind: SnsCanisterGapKind,
    role: SnsCanisterRole,
    inventory_canister_id: Option<String>,
    summary_canister_id: Option<String>,
) -> SnsCanisterGap {
    SnsCanisterGap {
        kind,
        role,
        inventory_canister_id,
        summary_canister_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sns::report::live::types::{DefiniteCanisterSettings, SnsRootExtensions};
    use candid::Nat;

    #[test]
    fn root_inventory_projection_preserves_native_health_and_canonical_roles() {
        let inventory = inventory_response();
        let health = health_response();

        let projected =
            mainnet_sns_canister_inventory(inventory, health).expect("project inventory");

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
        assert_eq!(projected.canisters[0].memory_size.as_deref(), Some("2000"));
        assert_eq!(
            projected.canisters[0].idle_cycles_burned_per_day.as_deref(),
            Some("30")
        );
        assert_eq!(projected.gaps.len(), 1);
        assert_eq!(
            projected.gaps[0].kind,
            SnsCanisterGapKind::HealthUnsupported
        );
        assert!(!projected.point_in_time_guaranteed);
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
            mainnet_sns_canister_inventory(inventory, health).expect("project inventory");
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

        let error = mainnet_sns_canister_inventory(inventory, health_response())
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
}
