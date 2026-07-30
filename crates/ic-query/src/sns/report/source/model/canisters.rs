//! Module: sns::report::source::model::canisters
//!
//! Responsibility: source result for SNS Root inventory and health collection.
//! Does not own: live Root transport, lookup, report assembly, or rendering.
//! Boundary: carries source provenance, joined rows, and typed gaps to builders.

use crate::sns::report::{SnsCanisterGap, SnsCanisterRole, SnsCanisterRow, SnsHostError};
use std::collections::BTreeMap;

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
