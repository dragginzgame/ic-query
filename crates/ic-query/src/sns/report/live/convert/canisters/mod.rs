//! Module: sns::report::live::convert::canisters
//!
//! Responsibility: join SNS Root inventory and health wire responses.
//! Does not own: Root transport, SNS lookup, report assembly, or rendering.
//! Boundary: retains inventory authority, native fields, canonical order, and typed gaps.

use crate::{
    hex::hex_bytes,
    icrc::ledger::nat_text,
    sns::report::{
        MainnetSnsCanisterInventory, SnsCanisterCycleBalanceStatus, SnsCanisterGap,
        SnsCanisterGapKind, SnsCanisterHealthQueryGap, SnsCanisterMethod, SnsCanisterRole,
        SnsCanisterRow, SnsCanisterStatus, SnsHostError,
        live::types::{
            CanisterStatusResult, CanisterStatusType, CanisterSummary,
            GetSnsCanistersSummaryResponse, ListSnsCanistersResponse,
        },
        source::SNS_CANISTER_HEALTH_CALL_TYPE,
    },
};
use ic_platform::candid::contract::Principal;
use std::collections::BTreeMap;

pub(in crate::sns::report::live) fn mainnet_sns_canister_inventory(
    inventory: ListSnsCanistersResponse,
    health: Result<GetSnsCanistersSummaryResponse, String>,
) -> Result<MainnetSnsCanisterInventory, SnsHostError> {
    let mut canisters = Vec::new();
    let mut gaps = Vec::new();
    let (mut health, health_query_gap) = match health {
        Ok(health) => (Some(health), None),
        Err(reason) => (
            None,
            Some(SnsCanisterHealthQueryGap {
                method: SnsCanisterMethod::GetSnsCanistersSummary,
                reason,
            }),
        ),
    };
    for (role, canister_id, summary) in [
        (
            SnsCanisterRole::Root,
            inventory.root,
            health.as_mut().and_then(|health| health.root.take()),
        ),
        (
            SnsCanisterRole::Governance,
            inventory.governance,
            health.as_mut().and_then(|health| health.governance.take()),
        ),
        (
            SnsCanisterRole::Ledger,
            inventory.ledger,
            health.as_mut().and_then(|health| health.ledger.take()),
        ),
        (
            SnsCanisterRole::Swap,
            inventory.swap,
            health.as_mut().and_then(|health| health.swap.take()),
        ),
        (
            SnsCanisterRole::Index,
            inventory.index,
            health.as_mut().and_then(|health| health.index.take()),
        ),
    ] {
        if health.is_some() {
            push_singleton(role, canister_id, summary, &mut canisters, &mut gaps);
        } else {
            push_inventory_only_singleton(role, canister_id, &mut canisters, &mut gaps);
        }
    }
    for (role, ids, summaries) in [
        (
            SnsCanisterRole::Archive,
            inventory.archives,
            health
                .as_mut()
                .map(|health| std::mem::take(&mut health.archives)),
        ),
        (
            SnsCanisterRole::Dapp,
            inventory.dapps,
            health
                .as_mut()
                .map(|health| std::mem::take(&mut health.dapps)),
        ),
    ] {
        if let Some(summaries) = summaries {
            push_many(role, ids, summaries, &mut canisters, &mut gaps);
        } else {
            push_inventory_only_many(role, ids, &mut canisters);
        }
    }
    push_extensions(
        inventory
            .extensions
            .into_iter()
            .flat_map(|extensions| extensions.extension_canister_ids)
            .collect(),
        &mut canisters,
        &mut gaps,
    );
    let mut inventory = MainnetSnsCanisterInventory {
        inventory_method: SnsCanisterMethod::ListSnsCanisters,
        health_method: SnsCanisterMethod::GetSnsCanistersSummary,
        health_call_type: SNS_CANISTER_HEALTH_CALL_TYPE,
        health_update_canister_list: false,
        point_in_time_guaranteed: false,
        canisters,
        health_query_gap,
        gaps,
    };
    crate::sns::report::source::canonicalize_mainnet_sns_canister_inventory(&mut inventory)?;
    Ok(inventory)
}

fn push_inventory_only_singleton(
    role: SnsCanisterRole,
    canister_id: Option<Principal>,
    canisters: &mut Vec<SnsCanisterRow>,
    gaps: &mut Vec<SnsCanisterGap>,
) {
    let Some(canister_id) = canister_id else {
        gaps.push(gap(
            SnsCanisterGapKind::InventoryCanisterIdMissing,
            role,
            None,
            None,
        ));
        return;
    };
    canisters.push(canister_row(role, canister_id.to_text(), None));
}

fn push_inventory_only_many(
    role: SnsCanisterRole,
    canister_ids: Vec<Principal>,
    canisters: &mut Vec<SnsCanisterRow>,
) {
    canisters.extend(
        canister_ids
            .into_iter()
            .map(|canister_id| canister_row(role, canister_id.to_text(), None)),
    );
}

fn push_extensions(
    extension_canister_ids: Vec<Principal>,
    canisters: &mut Vec<SnsCanisterRow>,
    gaps: &mut Vec<SnsCanisterGap>,
) {
    for canister_id in extension_canister_ids {
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

    let matched = match summary {
        None => Err((SnsCanisterGapKind::SummaryMissing, None)),
        Some(summary) => match summary_canister_id(Some(&summary)) {
            None => Err((SnsCanisterGapKind::SummaryCanisterIdMissing, None)),
            Some(id) if id != inventory_canister_id => {
                Err((SnsCanisterGapKind::SummaryCanisterIdMismatch, Some(id)))
            }
            Some(_) => Ok(summary.status),
        },
    };
    match matched {
        Ok(status) => push_matched_summary(role, inventory_canister_id, status, canisters, gaps),
        Err((kind, summary_id)) => {
            gaps.push(gap(
                kind,
                role,
                Some(inventory_canister_id.clone()),
                summary_id,
            ));
            canisters.push(canister_row(role, inventory_canister_id, None));
        }
    }
}

fn push_many(
    role: SnsCanisterRole,
    inventory_canister_ids: Vec<Principal>,
    summaries: Vec<CanisterSummary>,
    canisters: &mut Vec<SnsCanisterRow>,
    gaps: &mut Vec<SnsCanisterGap>,
) {
    let mut summaries_by_canister =
        BTreeMap::<String, (usize, Option<CanisterStatusResult>)>::new();

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
            .and_modify(|(count, status)| {
                *count += 1;
                *status = None;
            })
            .or_insert((1, summary.status));
    }

    for inventory_canister_id in inventory_canister_ids
        .into_iter()
        .map(|value| value.to_text())
    {
        let Some((count, status)) = summaries_by_canister.remove(&inventory_canister_id) else {
            canisters.push(canister_row(role, inventory_canister_id.clone(), None));
            gaps.push(gap(
                SnsCanisterGapKind::SummaryMissing,
                role,
                Some(inventory_canister_id),
                None,
            ));
            continue;
        };
        if count == 1 {
            push_matched_summary(role, inventory_canister_id, status, canisters, gaps);
            continue;
        }

        canisters.push(canister_row(role, inventory_canister_id.clone(), None));
        for _ in 1..count {
            gaps.push(gap(
                SnsCanisterGapKind::DuplicateSummary,
                role,
                Some(inventory_canister_id.clone()),
                Some(inventory_canister_id.clone()),
            ));
        }
    }

    for (summary_canister_id, (count, _)) in summaries_by_canister {
        for _ in 0..count {
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
            cycle_balance_status: SnsCanisterCycleBalanceStatus::Unavailable,
            memory_size: None,
            idle_cycles_burned_per_day: None,
            controllers: Vec::new(),
        };
    };
    let controllers = status
        .settings
        .controllers
        .into_iter()
        .map(|controller| controller.to_text())
        .collect::<Vec<_>>();
    let cycles = nat_text(&status.cycles);
    let cycle_balance_status = if cycles == "0" {
        SnsCanisterCycleBalanceStatus::ReportedZero
    } else {
        SnsCanisterCycleBalanceStatus::ReportedNonzero
    };
    SnsCanisterRow {
        role,
        canister_id,
        status: Some(match status.status {
            CanisterStatusType::Running => SnsCanisterStatus::Running,
            CanisterStatusType::Stopping => SnsCanisterStatus::Stopping,
            CanisterStatusType::Stopped => SnsCanisterStatus::Stopped,
        }),
        module_hash_hex: status.module_hash.map(|value| hex_bytes(&value)),
        cycles: Some(cycles),
        cycle_balance_status,
        memory_size: Some(nat_text(&status.memory_size)),
        idle_cycles_burned_per_day: Some(nat_text(&status.idle_cycles_burned_per_day)),
        controllers,
    }
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
mod tests;
