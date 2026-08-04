//! Module: sns::report::source::model::canisters
//!
//! Responsibility: source result for SNS Root inventory and health collection.
//! Does not own: live Root transport, lookup, report assembly, or rendering.
//! Boundary: carries source provenance, joined rows, and typed gaps to builders.

use super::validation::SnsSourceValidator;
use crate::{
    hex::is_canonical_lowercase_hex,
    sns::report::{
        SnsCanisterCallType, SnsCanisterCycleBalanceStatus, SnsCanisterGap, SnsCanisterGapKind,
        SnsCanisterHealthQueryGap, SnsCanisterMethod, SnsCanisterRole, SnsCanisterRow,
        SnsHostError,
    },
};
use std::collections::BTreeMap;

pub(in crate::sns::report) const SNS_CANISTER_HEALTH_CALL_TYPE: SnsCanisterCallType =
    SnsCanisterCallType::IngressUpdate;
const VALIDATOR: SnsSourceValidator = SnsSourceValidator::new("SNS Root canister inventory");

///
/// MainnetSnsCanisterInventory
///
/// Source-layer SNS Root inventory and health evidence for one deployed SNS.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsCanisterInventory {
    /// Root query method used as the inventory authority.
    pub inventory_method: SnsCanisterMethod,
    /// Root ingress method used for operational health.
    pub health_method: SnsCanisterMethod,
    /// Transport kind used for the health call.
    pub health_call_type: SnsCanisterCallType,
    /// Value sent in the Root health request.
    pub health_update_canister_list: bool,
    /// Whether the source can prove one point-in-time snapshot for all values.
    pub point_in_time_guaranteed: bool,
    /// Inventory rows returned by the source.
    pub canisters: Vec<SnsCanisterRow>,
    /// Root health ingress failure retained after successful inventory collection.
    pub health_query_gap: Option<SnsCanisterHealthQueryGap>,
    /// Explicit inventory or health relation gaps returned by the source.
    pub gaps: Vec<SnsCanisterGap>,
}

pub(in crate::sns::report) fn canonicalize_mainnet_sns_canister_inventory(
    inventory: &mut MainnetSnsCanisterInventory,
) -> Result<(), SnsHostError> {
    VALIDATOR.exact(
        "inventory_method",
        SnsCanisterMethod::ListSnsCanisters.as_str(),
        inventory.inventory_method.as_str(),
    )?;
    VALIDATOR.exact(
        "health_method",
        SnsCanisterMethod::GetSnsCanistersSummary.as_str(),
        inventory.health_method.as_str(),
    )?;
    VALIDATOR.exact(
        "health_call_type",
        SNS_CANISTER_HEALTH_CALL_TYPE.as_str(),
        inventory.health_call_type.as_str(),
    )?;
    if inventory.health_update_canister_list {
        return Err(VALIDATOR.invalid(
            "health_update_canister_list must be false for a read-only report".to_string(),
        ));
    }
    if inventory.point_in_time_guaranteed {
        return Err(VALIDATOR.invalid(
            "joined inventory and health cannot claim a point-in-time guarantee".to_string(),
        ));
    }

    validate_health_query_gap(inventory)?;

    for canister in &mut inventory.canisters {
        VALIDATOR.canonical_principal("canister_id", &canister.canister_id)?;
        for controller in &canister.controllers {
            VALIDATOR.canonical_principal("controller", controller)?;
        }
        canister.controllers.sort();
        if canister
            .controllers
            .windows(2)
            .any(|controllers| controllers[0] == controllers[1])
        {
            return Err(VALIDATOR.invalid(format!(
                "canister {} contains duplicate controllers",
                canister.canister_id
            )));
        }
        validate_canister_health(canister)?;
    }
    for gap in &inventory.gaps {
        for (field, principal) in [
            (
                "gap inventory_canister_id",
                gap.inventory_canister_id.as_deref(),
            ),
            (
                "gap summary_canister_id",
                gap.summary_canister_id.as_deref(),
            ),
        ] {
            if let Some(principal) = principal {
                VALIDATOR.canonical_principal(field, principal)?;
            }
        }
    }

    inventory.canisters.sort_by(|left, right| {
        (left.role, left.canister_id.as_str()).cmp(&(right.role, right.canister_id.as_str()))
    });
    inventory.gaps.sort_by(|left, right| {
        (
            left.role,
            left.inventory_canister_id.as_deref(),
            left.summary_canister_id.as_deref(),
            left.kind,
        )
            .cmp(&(
                right.role,
                right.inventory_canister_id.as_deref(),
                right.summary_canister_id.as_deref(),
                right.kind,
            ))
    });

    let mut seen = BTreeMap::<&str, SnsCanisterRole>::new();
    for canister in &inventory.canisters {
        if let Some(first_role) = seen.insert(&canister.canister_id, canister.role) {
            return Err(SnsHostError::DuplicateCanisterId {
                canister_id: canister.canister_id.clone(),
                first_role: first_role.as_str().to_string(),
                duplicate_role: canister.role.as_str().to_string(),
            });
        }
    }
    Ok(())
}

fn validate_canister_health(canister: &SnsCanisterRow) -> Result<(), SnsHostError> {
    let expected_cycle_balance_status = match canister.cycles.as_deref() {
        Some("0") => SnsCanisterCycleBalanceStatus::ReportedZero,
        Some(_) => SnsCanisterCycleBalanceStatus::ReportedNonzero,
        None => SnsCanisterCycleBalanceStatus::Unavailable,
    };
    if canister.cycle_balance_status != expected_cycle_balance_status {
        return Err(VALIDATOR.invalid(format!(
            "canister {} cycle_balance_status is {:?}, expected {:?} for cycles {:?}",
            canister.canister_id,
            canister.cycle_balance_status.as_str(),
            expected_cycle_balance_status.as_str(),
            canister.cycles
        )));
    }
    if canister.status.is_none() {
        if canister.module_hash_hex.is_some()
            || canister.cycles.is_some()
            || canister.memory_size.is_some()
            || canister.idle_cycles_burned_per_day.is_some()
            || !canister.controllers.is_empty()
        {
            return Err(VALIDATOR.invalid(format!(
                "canister {} has operational fields without status",
                canister.canister_id
            )));
        }
        return Ok(());
    }

    for (field, value) in [
        ("cycles", canister.cycles.as_deref()),
        ("memory_size", canister.memory_size.as_deref()),
        (
            "idle_cycles_burned_per_day",
            canister.idle_cycles_burned_per_day.as_deref(),
        ),
    ] {
        let value = value.ok_or_else(|| {
            VALIDATOR.invalid(format!(
                "canister {} with status is missing {field}",
                canister.canister_id
            ))
        })?;
        if !is_canonical_decimal(value) {
            return Err(VALIDATOR.invalid(format!(
                "canister {} {field} {value:?} is not canonical unsigned decimal text",
                canister.canister_id
            )));
        }
    }
    if let Some(hash) = canister.module_hash_hex.as_deref()
        && !is_canonical_lowercase_hex(hash)
    {
        return Err(VALIDATOR.invalid(format!(
            "canister {} module_hash_hex is not lowercase even-length hexadecimal text",
            canister.canister_id
        )));
    }
    Ok(())
}

fn validate_health_query_gap(inventory: &MainnetSnsCanisterInventory) -> Result<(), SnsHostError> {
    let Some(gap) = &inventory.health_query_gap else {
        return Ok(());
    };
    VALIDATOR.exact(
        "health_query_gap method",
        SnsCanisterMethod::GetSnsCanistersSummary.as_str(),
        gap.method.as_str(),
    )?;
    if gap.reason.trim().is_empty() {
        return Err(VALIDATOR.invalid("health_query_gap has an empty reason".to_string()));
    }
    if inventory
        .canisters
        .iter()
        .any(|canister| canister.status.is_some())
    {
        return Err(VALIDATOR
            .invalid("health_query_gap cannot coexist with returned canister status".to_string()));
    }
    if inventory.gaps.iter().any(|gap| {
        !matches!(
            gap.kind,
            SnsCanisterGapKind::InventoryCanisterIdMissing | SnsCanisterGapKind::HealthUnsupported
        )
    }) {
        return Err(VALIDATOR.invalid(
            "health_query_gap cannot coexist with health-response relation gaps".to_string(),
        ));
    }
    Ok(())
}

fn is_canonical_decimal(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}
