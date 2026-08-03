//! Module: sns::report::live::convert::upgrade
//!
//! Responsibility: convert native SNS version wire values into report DTOs.
//! Does not own: live calls, source validation, lookup, or rendering.
//! Boundary: preserves all six native role hashes and raw pending-upgrade fields.

use crate::{
    hex::hex_bytes,
    sns::report::{
        SnsPendingUpgrade, SnsRunningVersionResponse, SnsVersion,
        live::types::{GetRunningSnsVersionResponse, PendingSnsVersion, SnsVersionWire},
    },
};

pub(in crate::sns::report::live) fn sns_running_version_response(
    response: GetRunningSnsVersionResponse,
) -> SnsRunningVersionResponse {
    SnsRunningVersionResponse {
        deployed_version: response.deployed_version.map(sns_version),
        pending_version: response.pending_version.map(sns_pending_upgrade),
    }
}

pub(in crate::sns::report::live) fn sns_version(version: SnsVersionWire) -> SnsVersion {
    SnsVersion {
        archive_wasm_hash_hex: hex_bytes(&version.archive_wasm_hash),
        root_wasm_hash_hex: hex_bytes(&version.root_wasm_hash),
        swap_wasm_hash_hex: hex_bytes(&version.swap_wasm_hash),
        ledger_wasm_hash_hex: hex_bytes(&version.ledger_wasm_hash),
        governance_wasm_hash_hex: hex_bytes(&version.governance_wasm_hash),
        index_wasm_hash_hex: hex_bytes(&version.index_wasm_hash),
    }
}

pub(in crate::sns::report::live) fn sns_pending_upgrade(
    pending: PendingSnsVersion,
) -> SnsPendingUpgrade {
    SnsPendingUpgrade {
        mark_failed_at_seconds: pending.mark_failed_at_seconds,
        checking_upgrade_lock: pending.checking_upgrade_lock,
        proposal_id: pending.proposal_id,
        target_version: pending.target_version.map(sns_version),
    }
}
