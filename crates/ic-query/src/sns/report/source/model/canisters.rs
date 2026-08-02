//! Module: sns::report::source::model::canisters
//!
//! Responsibility: source result for SNS Root inventory and health collection.
//! Does not own: live Root transport, lookup, report assembly, or rendering.
//! Boundary: carries source provenance, joined rows, and typed gaps to builders.

use crate::{
    hex::is_canonical_lowercase_hex,
    sns::report::{SnsCanisterGap, SnsCanisterRole, SnsCanisterRow, SnsHostError},
};
use candid::Principal;
use std::collections::BTreeMap;

pub(in crate::sns::report) const SNS_CANISTER_INVENTORY_METHOD: &str = "list_sns_canisters";
pub(in crate::sns::report) const SNS_CANISTER_HEALTH_METHOD: &str = "get_sns_canisters_summary";
pub(in crate::sns::report) const SNS_CANISTER_HEALTH_CALL_TYPE: &str = "ingress_update";

///
/// MainnetSnsCanisterInventory
///
/// Source-layer SNS Root inventory and health evidence for one deployed SNS.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetSnsCanisterInventory {
    /// Root query method used as the inventory authority.
    pub inventory_method: String,
    /// Root ingress method used for operational health.
    pub health_method: String,
    /// Transport kind used for the health call.
    pub health_call_type: String,
    /// Value sent in the Root health request.
    pub health_update_canister_list: bool,
    /// Whether the source can prove one point-in-time snapshot for all values.
    pub point_in_time_guaranteed: bool,
    /// Inventory rows returned by the source.
    pub canisters: Vec<SnsCanisterRow>,
    /// Explicit inventory or health relation gaps returned by the source.
    pub gaps: Vec<SnsCanisterGap>,
}

pub(in crate::sns::report) fn canonicalize_mainnet_sns_canister_inventory(
    inventory: &mut MainnetSnsCanisterInventory,
) -> Result<(), SnsHostError> {
    validate_exact(
        "inventory_method",
        SNS_CANISTER_INVENTORY_METHOD,
        &inventory.inventory_method,
    )?;
    validate_exact(
        "health_method",
        SNS_CANISTER_HEALTH_METHOD,
        &inventory.health_method,
    )?;
    validate_exact(
        "health_call_type",
        SNS_CANISTER_HEALTH_CALL_TYPE,
        &inventory.health_call_type,
    )?;
    if inventory.health_update_canister_list {
        return Err(invalid_inventory(
            "health_update_canister_list must be false for a read-only report".to_string(),
        ));
    }
    if inventory.point_in_time_guaranteed {
        return Err(invalid_inventory(
            "joined inventory and health cannot claim a point-in-time guarantee".to_string(),
        ));
    }

    for canister in &mut inventory.canisters {
        validate_canonical_principal("canister_id", &canister.canister_id)?;
        for controller in &canister.controllers {
            validate_canonical_principal("controller", controller)?;
        }
        canister.controllers.sort();
        if canister
            .controllers
            .windows(2)
            .any(|controllers| controllers[0] == controllers[1])
        {
            return Err(invalid_inventory(format!(
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
                validate_canonical_principal(field, principal)?;
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
    if canister.status.is_none() {
        if canister.module_hash_hex.is_some()
            || canister.cycles.is_some()
            || canister.memory_size.is_some()
            || canister.idle_cycles_burned_per_day.is_some()
            || !canister.controllers.is_empty()
        {
            return Err(invalid_inventory(format!(
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
            invalid_inventory(format!(
                "canister {} with status is missing {field}",
                canister.canister_id
            ))
        })?;
        if !is_canonical_decimal(value) {
            return Err(invalid_inventory(format!(
                "canister {} {field} {value:?} is not canonical unsigned decimal text",
                canister.canister_id
            )));
        }
    }
    if let Some(hash) = canister.module_hash_hex.as_deref()
        && !is_canonical_lowercase_hex(hash)
    {
        return Err(invalid_inventory(format!(
            "canister {} module_hash_hex is not lowercase even-length hexadecimal text",
            canister.canister_id
        )));
    }
    Ok(())
}

fn validate_exact(field: &'static str, expected: &str, actual: &str) -> Result<(), SnsHostError> {
    if actual != expected {
        return Err(invalid_inventory(format!(
            "{field} is {actual:?}, expected {expected:?}"
        )));
    }
    Ok(())
}

fn validate_canonical_principal(field: &'static str, value: &str) -> Result<(), SnsHostError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_inventory(format!("{field} {value:?} is invalid: {error}")))?;
    if principal.to_text() != value {
        return Err(invalid_inventory(format!(
            "{field} {value:?} is not canonical principal text"
        )));
    }
    Ok(())
}

fn is_canonical_decimal(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

const fn invalid_inventory(reason: String) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: "SNS Root canister inventory",
        reason,
    }
}
